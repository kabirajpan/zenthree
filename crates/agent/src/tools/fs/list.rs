use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for listing directory contents safely to explore project structure.
/// Defaults to compact output (capping large folders and ignoring build artifacts),
/// with `show_all` or `max_items` available when full inspection is desired.
pub struct ListDirectoryTool;

impl Tool for ListDirectoryTool {
    fn name(&self) -> &str {
        "list_directory"
    }

    fn description(&self) -> &str {
        "List files and subdirectories in a directory path. Compact by default (ignores target/, node_modules/, .git/ and caps output to avoid context overflow). Use show_all: true to list everything."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Path to directory (defaults to current directory if omitted)"
                },
                "max_items": {
                    "type": "integer",
                    "description": "Maximum entries to display before capping (defaults to 50)"
                },
                "show_all": {
                    "type": "boolean",
                    "description": "If true, displays all entries without capping (defaults to false)"
                },
                "show_hidden": {
                    "type": "boolean",
                    "description": "If true, includes hidden/build directories like .git, target, node_modules (defaults to false)"
                }
            }
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .unwrap_or(".");

        let max_items = args.get("max_items")
            .and_then(|v| v.as_u64())
            .unwrap_or(50) as usize;

        let show_all = args.get("show_all")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let show_hidden = args.get("show_hidden")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!("Directory not found: {}", path_str)));
        }
        if !path.is_dir() {
            return Err(ToolError::ExecutionFailed(format!("Path is not a directory: {}", path_str)));
        }

        let read_dir = fs::read_dir(path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read directory '{}': {}", path_str, e)))?;

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !show_hidden && (name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" || name == ".turbo") {
                continue;
            }

            let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);

            if is_dir {
                dirs.push(format!("[DIR]  {}/", name));
            } else {
                let size_str = if size > 1024 * 1024 {
                    format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
                } else if size > 1024 {
                    format!("{:.1} KB", size as f64 / 1024.0)
                } else {
                    format!("{} B", size)
                };
                files.push(format!("[FILE] {} ({})", name, size_str));
            }
        }

        dirs.sort();
        files.sort();

        let total_dirs = dirs.len();
        let total_files = files.len();
        let total_entries = total_dirs + total_files;

        let mut all_lines = Vec::with_capacity(total_entries);
        all_lines.extend(dirs);
        all_lines.extend(files);

        if all_lines.is_empty() {
            return Ok(Value::String(format!("Directory '{}' is empty (or contains only hidden/ignored files).", path_str)));
        }

        let limit = if show_all { total_entries } else { max_items.min(total_entries) };
        let mut output_lines: Vec<String> = all_lines.into_iter().take(limit).collect();

        if !show_all && total_entries > limit {
            output_lines.push(format!("... ({} more items capped. Use show_all: true or increase max_items to view all)", total_entries - limit));
        }

        output_lines.push(format!("Total: {} directories, {} files", total_dirs, total_files));
        Ok(Value::String(output_lines.join("\n")))
    }
}
