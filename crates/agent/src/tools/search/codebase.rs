use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for searching text/symbols across the codebase.
pub struct SearchCodebaseTool;

impl Tool for SearchCodebaseTool {
    fn name(&self) -> &str {
        "search_codebase"
    }

    fn description(&self) -> &str {
        "Search for a keyword or symbol query across files in the codebase or specific directory."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Text pattern, symbol, or keyword to search for"
                },
                "path": {
                    "type": "string",
                    "description": "Root directory to start search from (defaults to '.')"
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Whether to perform case-sensitive search (defaults to false)"
                }
            },
            "required": ["query"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'query'".to_string()))?;

        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let case_sensitive = args.get("case_sensitive")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let query_lower = if case_sensitive { String::new() } else { query.to_lowercase() };

        let root = Path::new(path_str);
        if !root.exists() {
            return Err(ToolError::ExecutionFailed(format!("Path does not exist: {}", path_str)));
        }

        let mut matches = Vec::new();
        const MAX_MATCHES: usize = 100;

        let mut stack = vec![root.to_path_buf()];
        while let Some(current) = stack.pop() {
            if matches.len() >= MAX_MATCHES {
                break;
            }

            if current.is_file() {
                if let Ok(content) = fs::read_to_string(&current) {
                    for (idx, line) in content.lines().enumerate() {
                        let matched = if case_sensitive {
                            line.contains(query)
                        } else {
                            line.to_lowercase().contains(&query_lower)
                        };

                        if matched {
                            matches.push(format!(
                                "{}:{}: {}",
                                current.display(),
                                idx + 1,
                                line.trim()
                            ));
                            if matches.len() >= MAX_MATCHES {
                                break;
                            }
                        }
                    }
                }
            } else if current.is_dir() {
                let name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" {
                    continue;
                }

                if let Ok(dir) = fs::read_dir(&current) {
                    for entry in dir.flatten() {
                        stack.push(entry.path());
                    }
                }
            }
        }

        if matches.is_empty() {
            Ok(Value::String(format!("No matches found for '{}' in '{}'", query, path_str)))
        } else {
            let count = matches.len();
            let mut output = matches.join("\n");
            if count >= MAX_MATCHES {
                output.push_str("\n... (results capped at 100 matches)");
            }
            Ok(Value::String(output))
        }
    }
}
