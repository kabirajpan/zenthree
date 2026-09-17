use std::path::{Path, PathBuf};
use super::AgentToolResult;

pub fn execute_list_directory(workspace: &Path, rel_path: Option<&str>) -> AgentToolResult {
    let target_dir = match rel_path {
        Some(p) if !p.trim().is_empty() && p.trim() != "." => {
            let cleaned = p.trim().trim_matches('"').trim_matches('\'').trim();
            if Path::new(cleaned).is_absolute() {
                PathBuf::from(cleaned)
            } else {
                workspace.join(cleaned)
            }
        }
        _ => workspace.to_path_buf(),
    };

    let entries = match std::fs::read_dir(&target_dir) {
        Ok(e) => e,
        Err(err) => {
            return AgentToolResult {
                tool_name: "list_directory".to_string(),
                ok: false,
                output: format!("Failed to list directory `{}`: {}", target_dir.display(), err),
            };
        }
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" || name == "dist" || name == "build" {
            continue;
        }

        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                dirs.push(format!("{}/", name));
            } else {
                let size_str = entry
                    .metadata()
                    .map(|m| {
                        let bytes = m.len();
                        if bytes < 1024 {
                            format!("{} B", bytes)
                        } else if bytes < 1024 * 1024 {
                            format!("{:.1} KB", bytes as f64 / 1024.0)
                        } else {
                            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                        }
                    })
                    .unwrap_or_default();
                files.push(format!("{} ({})", name, size_str));
            }
        }
    }

    dirs.sort();
    files.sort();

    let display_path = target_dir.strip_prefix(workspace).unwrap_or(&target_dir).to_string_lossy();
    let label = if display_path.is_empty() || display_path == "." {
        "workspace root".to_string()
    } else {
        format!("`{}`", display_path)
    };

    let mut out = format!("Listing of {}:\n", label);
    for d in dirs {
        out.push_str(&format!("  📁 {}\n", d));
    }
    for f in files {
        out.push_str(&format!("  📄 {}\n", f));
    }

    AgentToolResult {
        tool_name: "list_directory".to_string(),
        ok: true,
        output: out.trim_end().to_string(),
    }
}
