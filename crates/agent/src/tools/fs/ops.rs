use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool to delete a file safely within workspace boundaries.
#[derive(Default)]
pub struct DeleteFileTool {
    workspace_root: Option<PathBuf>,
}

impl DeleteFileTool {
    pub fn new() -> Self {
        Self { workspace_root: None }
    }

    pub fn with_workspace(mut self, root: PathBuf) -> Self {
        self.workspace_root = Some(root);
        self
    }
}

impl Tool for DeleteFileTool {
    fn name(&self) -> &str {
        "delete_file"
    }

    fn description(&self) -> &str {
        "Delete a file from the filesystem."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to the file to remove"
                }
            },
            "required": ["path"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Destructive
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'path'".to_string()))?;

        let trimmed = path_str.trim();
        if trimmed == "." || trimmed == "./" || trimmed == ".." || trimmed.is_empty() || trimmed == "/" {
            return Err(ToolError::ExecutionFailed("Refusing to delete project root or parent directory".to_string()));
        }

        let rel_norm = trimmed.replace('\\', "/");
        if rel_norm.contains(".zenthree_sessions") || rel_norm.contains(".zenthree_state.json") {
            return Err(ToolError::ExecutionFailed(
                "Access denied: Cannot delete internal IDE runtime files.".to_string(),
            ));
        }

        let resolved = if let Some(ref ws) = self.workspace_root {
            if Path::new(trimmed).is_absolute() {
                PathBuf::from(trimmed)
            } else {
                ws.join(trimmed)
            }
        } else {
            PathBuf::from(trimmed)
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
            return Err(ToolError::ExecutionFailed(format!("File does not exist: {}", path_str)));
        }

        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to delete directory '{}': {}", path_str, e)))?;
            Ok(Value::String(format!("Successfully deleted directory '{}'", path_str)))
        } else {
            fs::remove_file(path)
                .map_err(|e| ToolError::ExecutionFailed(format!("Failed to delete file '{}': {}", path_str, e)))?;
            Ok(Value::String(format!("Successfully deleted file '{}'", path_str)))
        }
    }
}

/// Tool to rename or move a file safely within workspace boundaries.
#[derive(Default)]
pub struct RenameFileTool {
    workspace_root: Option<PathBuf>,
}

impl RenameFileTool {
    pub fn new() -> Self {
        Self { workspace_root: None }
    }

    pub fn with_workspace(mut self, root: PathBuf) -> Self {
        self.workspace_root = Some(root);
        self
    }
}

impl Tool for RenameFileTool {
    fn name(&self) -> &str {
        "rename_file"
    }

    fn description(&self) -> &str {
        "Rename or move a file from old_path to new_path."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "old_path": {
                    "type": "string",
                    "description": "Current file path"
                },
                "new_path": {
                    "type": "string",
                    "description": "Destination file path"
                }
            },
            "required": ["old_path", "new_path"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Mutating
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let old_path = args.get("old_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'old_path'".to_string()))?;

        let new_path = args.get("new_path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'new_path'".to_string()))?;

        let resolve_path = |p: &str| -> PathBuf {
            if let Some(ref ws) = self.workspace_root {
                if Path::new(p).is_absolute() {
                    PathBuf::from(p)
                } else {
                    ws.join(p)
                }
            } else {
                PathBuf::from(p)
            }
        };

        let old_res = resolve_path(old_path);
        let new_res = resolve_path(new_path);

        if let Some(ref ws) = self.workspace_root {
            let canon_ws = ws.canonicalize().unwrap_or_else(|_| ws.clone());
            let canon_old = old_res.canonicalize().unwrap_or_else(|_| old_res.clone());
            if !canon_old.starts_with(&canon_ws) {
                return Err(ToolError::ExecutionFailed(format!(
                    "Access denied: Source path '{}' is outside the project workspace boundary ('{}').",
                    old_path,
                    ws.display()
                )));
            }

            let check_dir = if let Some(parent) = new_res.parent() {
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
                    "Access denied: Destination path '{}' is outside the project workspace boundary ('{}').",
                    new_path,
                    ws.display()
                )));
            }
        }

        let old = &old_res;
        if !old.exists() {
            return Err(ToolError::ExecutionFailed(format!("Source path '{}' does not exist", old_path)));
        }

        let new = &new_res;
        if let Some(parent) = new.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent)
                    .map_err(|e| ToolError::ExecutionFailed(format!("Failed to create destination directories for '{}': {}", new_path, e)))?;
            }
        }

        fs::rename(old, new)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to rename '{}' to '{}': {}", old_path, new_path, e)))?;

        Ok(Value::String(format!("Successfully renamed '{}' to '{}'", old_path, new_path)))
    }
}
