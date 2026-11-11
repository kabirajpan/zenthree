use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for applying targeted patches/edits to files safely within workspace boundaries.
#[derive(Default)]
pub struct EditFileTool {
    workspace_root: Option<PathBuf>,
}

impl EditFileTool {
    pub fn new() -> Self {
        Self { workspace_root: None }
    }

    pub fn with_workspace(mut self, root: PathBuf) -> Self {
        self.workspace_root = Some(root);
        self
    }
}

impl Tool for EditFileTool {
    fn name(&self) -> &str {
        "edit_file"
    }

    fn description(&self) -> &str {
        "Apply a targeted patch to a file by replacing unique target text with replacement text. Safer and cheaper than rewriting the whole file."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to edit"
                },
                "target": {
                    "type": "string",
                    "description": "Unique text snippet in the file to replace (include surrounding context if needed to ensure uniqueness)"
                },
                "replacement": {
                    "type": "string",
                    "description": "New text to substitute in place of target"
                }
            },
            "required": ["path", "target", "replacement"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Mutating
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let path_str = args.get("TargetFile")
            .or_else(|| args.get("path"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'TargetFile' or 'path'".to_string()))?;

        let target = args.get("TargetContent")
            .or_else(|| args.get("target"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'TargetContent' or 'target'".to_string()))?;

        let replacement = args.get("ReplacementContent")
            .or_else(|| args.get("replacement"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'ReplacementContent' or 'replacement'".to_string()))?;

        let start_line = args.get("StartLine")
            .or_else(|| args.get("start_line"))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let end_line = args.get("EndLine")
            .or_else(|| args.get("end_line"))
            .and_then(|v| v.as_u64())
            .map(|v| v as usize);

        let allow_multiple = args.get("AllowMultiple")
            .or_else(|| args.get("allow_multiple"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let rel_norm = path_str.replace('\\', "/");
        if rel_norm.contains(".zenthree_sessions") || rel_norm.contains(".zenthree_state.json") {
            return Err(ToolError::ExecutionFailed(
                "Access denied: Cannot edit internal IDE runtime files.".to_string(),
            ));
        }

        let resolved = if let Some(ref ws) = self.workspace_root {
            if Path::new(path_str).is_absolute() {
                PathBuf::from(path_str)
            } else {
                ws.join(path_str)
            }
        } else {
            PathBuf::from(path_str)
        };

        if let Some(ref ws) = self.workspace_root {
            let canon_ws = ws.canonicalize().unwrap_or_else(|_| ws.clone());
            let canon_res = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
            if !canon_res.starts_with(&canon_ws) {
                return Err(ToolError::ExecutionFailed(format!(
                    "Access denied: Path '{}' is outside the project workspace boundary ('{}'). You are strictly forbidden from modifying files in parent directories.",
                    path_str,
                    ws.display()
                )));
            }
        }

        let path = &resolved;
        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!("File not found: {}", path_str)));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read '{}': {}", path_str, e)))?;

        // If line range is specified, scope replacement to those exact lines
        let patched = if let (Some(start), Some(end)) = (start_line, end_line) {
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();
            if start == 0 || start > end || end > total_lines {
                return Err(ToolError::InvalidArgs(format!(
                    "Invalid line range [{}, {}] for file with {} lines",
                    start, end, total_lines
                )));
            }

            let slice = lines[start - 1..end].join("\n");
            if !slice.contains(target) {
                return Err(ToolError::ExecutionFailed(format!(
                    "TargetContent not found in lines {}-{} of '{}'. Ensure whitespace and lines match exactly.",
                    start, end, path_str
                )));
            }

            let occurrences = slice.matches(target).count();
            if !allow_multiple && occurrences > 1 {
                return Err(ToolError::ExecutionFailed(format!(
                    "TargetContent matched {} times in lines {}-{}. Narrow the line range or provide unique surrounding context.",
                    occurrences, start, end
                )));
            }

            let patched_slice = if allow_multiple {
                slice.replace(target, replacement)
            } else {
                slice.replacen(target, replacement, 1)
            };

            let mut working: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
            let new_lines: Vec<String> = patched_slice.lines().map(|s| s.to_string()).collect();
            working.splice(start - 1..end, new_lines);

            let mut res = working.join("\n");
            if content.ends_with('\n') {
                res.push('\n');
            }
            res
        } else {
            if !content.contains(target) {
                return Err(ToolError::ExecutionFailed(format!(
                    "Target snippet not found in '{}'. Ensure whitespace and lines match exactly.",
                    path_str
                )));
            }

            let occurrences = content.matches(target).count();
            if !allow_multiple && occurrences > 1 {
                return Err(ToolError::ExecutionFailed(format!(
                    "Target snippet matched {} times in '{}'. Provide additional surrounding context to make it unique.",
                    occurrences, path_str
                )));
            }

            if allow_multiple {
                content.replace(target, replacement)
            } else {
                content.replacen(target, replacement, 1)
            }
        };

        fs::write(path, patched)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write patched file '{}': {}", path_str, e)))?;

        Ok(Value::String(format!("Successfully patched '{}'", path_str)))
    }
}
