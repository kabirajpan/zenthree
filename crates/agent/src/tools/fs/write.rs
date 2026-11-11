use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for creating or overwriting a file safely within workspace boundaries.
#[derive(Default)]
pub struct WriteFileTool {
    workspace_root: Option<PathBuf>,
}

impl WriteFileTool {
    pub fn new() -> Self {
        Self { workspace_root: None }
    }

    pub fn with_workspace(mut self, root: PathBuf) -> Self {
        self.workspace_root = Some(root);
        self
    }
}

impl Tool for WriteFileTool {
    fn name(&self) -> &str {
        "write_file"
    }

    fn description(&self) -> &str {
        "Create a new file or completely overwrite an existing file with content. Automatically creates parent directories."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to create or overwrite"
                },
                "content": {
                    "type": "string",
                    "description": "Full file content"
                }
            },
            "required": ["path", "content"]
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

        let content = args.get("CodeContent")
            .or_else(|| args.get("content"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'CodeContent' or 'content'".to_string()))?;

        let overwrite = args.get("Overwrite")
            .or_else(|| args.get("overwrite"))
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let rel_norm = path_str.replace('\\', "/");
        if rel_norm.contains(".zenthree_sessions") || rel_norm.contains(".zenthree_state.json") {
            return Err(ToolError::ExecutionFailed(
                "Access denied: Cannot write to internal IDE runtime files.".to_string(),
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
            let check_dir = if let Some(parent) = resolved.parent() {
                if parent.exists() {
                    parent.canonicalize().unwrap_or_else(|_| parent.to_path_buf())
                } else {
                    canon_ws.clone()
                }
            } else {
                canon_ws.clone()
            };
            if !check_dir.starts_with(&canon_ws) {
                return Err(ToolError::ExecutionFailed(format!(
                    "Access denied: Path '{}' is outside the project workspace boundary ('{}'). You are strictly forbidden from modifying files in parent directories.",
                    path_str,
                    ws.display()
                )));
            }
        }

        let path = &resolved;

        if !overwrite && path.exists() {
            return Err(ToolError::ExecutionFailed(format!(
                "File '{}' already exists. Pass 'Overwrite: true' to overwrite it.",
                path_str
            )));
        }

        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create directories for '{}': {}", path_str, e)))?;
            }
        }

        fs::write(path, content)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write to '{}': {}", path_str, e)))?;

        Ok(Value::String(format!("Successfully wrote {} bytes to '{}'", content.len(), path_str)))
    }
}
