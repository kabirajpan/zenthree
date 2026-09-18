use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Input parameters for the `grep` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrepInput {
    pub regex: String,
    #[serde(default)]
    pub include_pattern: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub case_sensitive: Option<bool>,
}

/// A single matched line in a file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepMatch {
    pub path: String,
    pub line_number: usize,
    pub line_content: String,
}

/// Output returned by the `grep` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrepOutput {
    pub query: String,
    pub matches: Vec<GrepMatch>,
    pub total_matches: usize,
    pub offset: usize,
    pub has_more: bool,
}

pub const DEFAULT_GREP_LIMIT: usize = 50;

/// Executes a regex content search across workspace files with glob filtering and pagination.
pub fn execute_grep(
    workspace_root: &Path,
    query: &str,
    include_pattern: Option<&str>,
    offset: Option<usize>,
    limit: Option<usize>,
    case_sensitive: Option<bool>,
) -> Result<GrepOutput, String> {
    let re = RegexBuilder::new(query)
        .case_insensitive(!case_sensitive.unwrap_or(false))
        .build()
        .map_err(|e| format!("Invalid regex pattern `{}`: {}", query, e))?;

    let include_glob = include_pattern.map(|p| p.trim()).filter(|p| !p.is_empty());
    let offset_val = offset.unwrap_or(0);
    let limit_val = limit.unwrap_or(DEFAULT_GREP_LIMIT);

    let mut all_matches = Vec::new();
    scan_dir_for_regex(workspace_root, workspace_root, &re, include_glob, &mut all_matches)?;

    let total_matches = all_matches.len();
    let has_more = offset_val + limit_val < total_matches;

    let paged_matches = if offset_val < total_matches {
        all_matches
            .into_iter()
            .skip(offset_val)
            .take(limit_val)
            .collect()
    } else {
        Vec::new()
    };

    Ok(GrepOutput {
        query: query.to_string(),
        matches: paged_matches,
        total_matches,
        offset: offset_val,
        has_more,
    })
}

fn scan_dir_for_regex(
    base: &Path,
    current: &Path,
    re: &Regex,
    include_glob: Option<&str>,
    out: &mut Vec<GrepMatch>,
) -> Result<(), String> {
    let entries = match std::fs::read_dir(current) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        // Skip common noisy/large directories
        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor" {
            continue;
        }

        if path.is_dir() {
            scan_dir_for_regex(base, &path, re, include_glob, out)?;
        } else if path.is_file() {
            let rel_path = match path.strip_prefix(base) {
                Ok(p) => p.to_string_lossy().to_string(),
                Err(_) => path.to_string_lossy().to_string(),
            };

            // Check include glob if specified
            if let Some(glob_pat) = include_glob {
                if !matches_simple_glob(&rel_path, glob_pat) {
                    continue;
                }
            }

            // Read file and search lines
            if let Ok(content) = std::fs::read_to_string(&path) {
                for (idx, line) in content.lines().enumerate() {
                    if re.is_match(line) {
                        out.push(GrepMatch {
                            path: rel_path.clone(),
                            line_number: idx + 1,
                            line_content: line.to_string(),
                        });
                    }
                }
            }
        }
    }

    Ok(())
}

fn matches_simple_glob(path: &str, glob: &str) -> bool {
    let clean_glob = glob.trim();
    if clean_glob == "*" || clean_glob == "**/*" {
        return true;
    }

    if let Some(suffix) = clean_glob.strip_prefix('*') {
        if let Some(ext) = suffix.strip_prefix('.') {
            return path.ends_with(&format!(".{}", ext));
        }
    }

    if let Some(suffix) = clean_glob.strip_prefix("**/*") {
        return path.ends_with(suffix);
    }

    path.contains(clean_glob.trim_matches('*'))
}

/// Returns the JSON Schema specification for `grep`.
pub fn grep_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "grep",
        "description": "Searches the contents of files across the project using regular expressions.\n\nUse this tool to find definitions, function calls, structs, or text patterns without knowing the exact file path.\n\n<example>\n{\"tool\": \"grep\", \"regex\": \"fn execute_read_file\", \"include_pattern\": \"**/*.rs\"}\n</example>",
        "parameters": {
            "type": "object",
            "properties": {
                "regex": {
                    "type": "string",
                    "description": "A regular expression pattern to search for in file contents."
                },
                "include_pattern": {
                    "type": "string",
                    "description": "Optional glob pattern to restrict searches (e.g. '**/*.rs', 'crates/agent/**/*.rs')."
                },
                "offset": {
                    "type": "integer",
                    "description": "0-based pagination offset (default 0)."
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results to return (default 50)."
                },
                "case_sensitive": {
                    "type": "boolean",
                    "description": "Whether to enforce case sensitivity (default false)."
                }
            },
            "required": ["regex"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_execute_grep_finds_symbols() {
        let dir = tempdir().unwrap();
        let ws = dir.path();

        let src = ws.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("lib.rs"), "pub fn start_agent() {}\npub fn stop_agent() {}\n").unwrap();
        fs::write(src.join("ignore.txt"), "start_agent in text file").unwrap();

        let res = execute_grep(ws, "start_agent", Some("**/*.rs"), None, None, None).unwrap();
        assert_eq!(res.matches.len(), 1);
        assert_eq!(res.matches[0].path, "src/lib.rs");
        assert_eq!(res.matches[0].line_number, 1);
        assert!(res.matches[0].line_content.contains("pub fn start_agent()"));
    }

    #[test]
    fn test_execute_grep_pagination() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        let mut content = String::new();
        for i in 1..=20 {
            content.push_str(&format!("item_{}\n", i));
        }
        fs::write(ws.join("items.rs"), content).unwrap();

        let res = execute_grep(ws, "item_", None, Some(5), Some(5), None).unwrap();
        assert_eq!(res.total_matches, 20);
        assert_eq!(res.matches.len(), 5);
        assert_eq!(res.matches[0].line_number, 6);
        assert!(res.has_more);
    }
}
