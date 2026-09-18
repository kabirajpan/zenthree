use serde::{Deserialize, Serialize};
use std::path::Path;

/// Input parameters for the `find_path` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindPathInput {
    pub glob: String,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

/// Output returned by the `find_path` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FindPathOutput {
    pub glob: String,
    pub matches: Vec<String>,
    pub total_matches: usize,
    pub offset: usize,
    pub has_more: bool,
}

pub const DEFAULT_FIND_LIMIT: usize = 50;

/// Finds project files matching a glob or substring pattern.
pub fn execute_find_path(
    workspace_root: &Path,
    glob: &str,
    offset: Option<usize>,
    limit: Option<usize>,
) -> Result<FindPathOutput, String> {
    let clean_glob = glob.trim();
    if clean_glob.is_empty() {
        return Err("Glob search pattern cannot be empty".to_string());
    }

    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(DEFAULT_FIND_LIMIT);

    let mut all_files = Vec::new();
    collect_files_recursive(workspace_root, workspace_root, &mut all_files);

    // Filter matching paths
    let mut matching_paths: Vec<String> = all_files
        .into_iter()
        .filter(|p| matches_path_glob(p, clean_glob))
        .collect();

    matching_paths.sort();

    let total_matches = matching_paths.len();
    let has_more = offset_val + limit_val < total_matches;

    let paged_matches = if offset_val < total_matches {
        matching_paths
            .into_iter()
            .skip(offset_val)
            .take(limit_val)
            .collect()
    } else {
        Vec::new()
    };

    Ok(FindPathOutput {
        glob: clean_glob.to_string(),
        matches: paged_matches,
        total_matches,
        offset: offset_val,
        has_more,
    })
}

fn collect_files_recursive(base: &Path, current: &Path, out: &mut Vec<String>) {
    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor" {
            continue;
        }

        if path.is_dir() {
            collect_files_recursive(base, &path, out);
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                out.push(rel.to_string_lossy().to_string());
            }
        }
    }
}

fn matches_path_glob(path: &str, glob: &str) -> bool {
    let lower_path = path.to_ascii_lowercase();
    let lower_glob = glob.to_ascii_lowercase();

    if lower_glob == "*" || lower_glob == "**/*" {
        return true;
    }

    // Exact suffix match (e.g. *.rs or *.json)
    if let Some(suffix) = lower_glob.strip_prefix('*') {
        if let Some(ext) = suffix.strip_prefix('.') {
            return lower_path.ends_with(&format!(".{}", ext));
        }
    }

    if let Some(suffix) = lower_glob.strip_prefix("**/*") {
        return lower_path.ends_with(suffix);
    }

    // Substring match
    let pattern = lower_glob.trim_matches('*');
    lower_path.contains(pattern)
}

/// Returns the JSON Schema specification for `find_path`.
pub fn find_path_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "find_path",
        "description": "Finds file paths matching a given name pattern or glob across the project.\n\nUse this tool to locate files when you do not know the exact directory location.\n\n<example>\n{\"tool\": \"find_path\", \"glob\": \"*secret_scrubber*\"}\n</example>",
        "parameters": {
            "type": "object",
            "properties": {
                "glob": {
                    "type": "string",
                    "description": "Glob or substring pattern to match against project relative paths (e.g. '*test*.rs', 'crates/**/lib.rs')."
                },
                "offset": {
                    "type": "integer",
                    "description": "0-based pagination offset (default 0)."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of file paths to return (default 50)."
                }
            },
            "required": ["glob"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_execute_find_path_matches_glob() {
        let dir = tempdir().unwrap();
        let ws = dir.path();

        let sub1 = ws.join("crates").join("agent").join("src");
        fs::create_dir_all(&sub1).unwrap();
        fs::write(sub1.join("lib.rs"), "").unwrap();
        fs::write(sub1.join("parser.rs"), "").unwrap();

        let sub2 = ws.join("crates").join("editor").join("src");
        fs::create_dir_all(&sub2).unwrap();
        fs::write(sub2.join("buffer.rs"), "").unwrap();

        let res = execute_find_path(ws, "*parser*", None, None).unwrap();
        assert_eq!(res.matches.len(), 1);
        assert_eq!(res.matches[0], "crates/agent/src/parser.rs");

        let res_all_rs = execute_find_path(ws, "*.rs", None, None).unwrap();
        assert_eq!(res_all_rs.matches.len(), 3);
    }
}
