use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for generating a safe, compact tree structure of the workspace.
/// Folds directories with counts and caps items to prevent context blowouts.
pub struct GetProjectStructureTool;

impl Tool for GetProjectStructureTool {
    fn name(&self) -> &str {
        "get_project_structure"
    }

    fn description(&self) -> &str {
        "Get a compact, high-level directory tree of the workspace. Automatically folds directories with item counts and ignores build artifacts (target/, node_modules/, .git/) to keep context lean."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Root path to inspect (defaults to current directory '.')"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum tree depth to expand (defaults to 2)"
                }
            }
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let root_str = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
        let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(2) as usize;

        let root_path = Path::new(root_str);
        if !root_path.exists() {
            return Err(ToolError::ExecutionFailed(format!("Path does not exist: {}", root_str)));
        }

        let mut output = String::new();
        output.push_str(&format!("{}/\n", root_str));
        render_tree(root_path, "", 1, max_depth, &mut output);

        Ok(Value::String(output))
    }
}

fn is_ignored(name: &str) -> bool {
    name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" || name == ".turbo"
}

fn count_children(dir: &Path) -> (usize, usize) {
    let mut files = 0;
    let mut dirs = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_ignored(&name) {
                continue;
            }
            if let Ok(ft) = entry.file_type() {
                if ft.is_dir() {
                    dirs += 1;
                } else {
                    files += 1;
                }
            }
        }
    }
    (dirs, files)
}

fn render_tree(dir: &Path, prefix: &str, current_depth: usize, max_depth: usize, output: &mut String) {
    let mut entries: Vec<PathBuf> = Vec::new();
    if let Ok(read_dir) = fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if is_ignored(&name) {
                continue;
            }
            entries.push(entry.path());
        }
    }

    // Sort: directories first, then files alphabetically
    entries.sort_by(|a, b| {
        let a_is_dir = a.is_dir();
        let b_is_dir = b.is_dir();
        if a_is_dir == b_is_dir {
            a.file_name().cmp(&b.file_name())
        } else if a_is_dir {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });

    let total = entries.len();
    const MAX_DISPLAY_PER_DIR: usize = 12;
    let display_count = total.min(MAX_DISPLAY_PER_DIR);

    for (idx, path) in entries.iter().take(display_count).enumerate() {
        let is_last = idx == display_count - 1 && total <= MAX_DISPLAY_PER_DIR;
        let branch = if is_last { "└── " } else { "├── " };
        let child_prefix = if is_last { "    " } else { "│   " };
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        if path.is_dir() {
            let (child_dirs, child_files) = count_children(path);
            let summary = match (child_dirs, child_files) {
                (d, f) if d > 0 && f > 0 => format!("{} subdirs, {} files", d, f),
                (d, 0) if d > 0 => format!("{} subdirs", d),
                (0, f) if f > 0 => format!("{} files", f),
                _ => "empty".to_string(),
            };

            if current_depth < max_depth && (child_dirs > 0 || child_files > 0) {
                output.push_str(&format!("{}{}{}/ ({})\n", prefix, branch, name, summary));
                let next_prefix = format!("{}{}", prefix, child_prefix);
                render_tree(path, &next_prefix, current_depth + 1, max_depth, output);
            } else {
                output.push_str(&format!("{}{}{}/ [{}]\n", prefix, branch, name, summary));
            }
        } else {
            let size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
            let size_str = if size > 1024 * 1024 {
                format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
            } else if size > 1024 {
                format!("{:.1} KB", size as f64 / 1024.0)
            } else {
                format!("{} B", size)
            };
            output.push_str(&format!("{}{}{} ({})\n", prefix, branch, name, size_str));
        }
    }

    if total > MAX_DISPLAY_PER_DIR {
        let remaining = total - MAX_DISPLAY_PER_DIR;
        output.push_str(&format!("{}└── ... ({} more items)\n", prefix, remaining));
    }
}
