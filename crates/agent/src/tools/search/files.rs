use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for finding files across the repository by pattern, name, or extension.
pub struct FindFilesTool;

impl Tool for FindFilesTool {
    fn name(&self) -> &str {
        "find_files"
    }

    fn description(&self) -> &str {
        "Find file paths matching a filename or wildcard pattern (e.g. '*.rs', 'Cargo.toml', 'auth.*', '*.scm') across the project."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "pattern": {
                    "type": "string",
                    "description": "Filename or wildcard pattern to search for (e.g. '*.rs', 'ai_panel.rs')"
                },
                "path": {
                    "type": "string",
                    "description": "Root directory to start search from (defaults to '.')"
                }
            },
            "required": ["pattern"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'pattern'".to_string()))?;

        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let root = Path::new(path_str);
        if !root.exists() {
            return Err(ToolError::ExecutionFailed(format!("Path does not exist: {}", path_str)));
        }

        let mut matches = Vec::new();
        const MAX_MATCHES: usize = 50;

        let pattern_lower = pattern.to_lowercase();
        let is_wildcard = pattern.starts_with('*');
        let clean_ext = if is_wildcard { &pattern_lower[1..] } else { &pattern_lower };

        let mut stack = vec![root.to_path_buf()];
        while let Some(current) = stack.pop() {
            if matches.len() >= MAX_MATCHES {
                break;
            }

            if current.is_file() {
                let file_name = current.file_name().and_then(|n| n.to_str()).unwrap_or("").to_lowercase();
                let matched = if is_wildcard {
                    file_name.ends_with(clean_ext)
                } else if pattern_lower.contains('*') {
                    // Simple wildcard check
                    let parts: Vec<&str> = pattern_lower.split('*').collect();
                    if parts.len() == 2 {
                        file_name.starts_with(parts[0]) && file_name.ends_with(parts[1])
                    } else {
                        file_name.contains(&pattern_lower)
                    }
                } else {
                    file_name.contains(&pattern_lower)
                };

                if matched {
                    matches.push(current.display().to_string());
                }
            } else if current.is_dir() {
                let dir_name = current.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if dir_name.starts_with('.') || dir_name == "target" || dir_name == "node_modules" || dir_name == "dist" {
                    continue;
                }

                if let Ok(read_dir) = fs::read_dir(&current) {
                    for entry in read_dir.flatten() {
                        stack.push(entry.path());
                    }
                }
            }
        }

        matches.sort();
        if matches.is_empty() {
            Ok(Value::String(format!("No files found matching pattern '{}' in '{}'", pattern, path_str)))
        } else {
            let total = matches.len();
            let mut res = matches.join("\n");
            if total >= MAX_MATCHES {
                res.push_str(&format!("\n... (capped at {} results)", MAX_MATCHES));
            }
            Ok(Value::String(res))
        }
    }
}
