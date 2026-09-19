use std::path::Path;
use super::AgentToolResult;

pub fn execute_search_code(workspace: &Path, query: &str) -> AgentToolResult {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return AgentToolResult {
            tool_name: "search_code".to_string(),
            ok: false,
            output: "Search query cannot be empty".to_string(),
        };
    }

    let mut matches = Vec::new();
    let lower_query = trimmed.to_lowercase();

    fn walk_dir(
        dir: &Path,
        workspace: &Path,
        lower_query: &str,
        matches: &mut Vec<String>,
        max_matches: usize,
    ) {
        if matches.len() >= max_matches {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            if matches.len() >= max_matches {
                break;
            }

            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str == ".git"
                || name_str == ".zenthree"
                || name_str == ".svn"
                || name_str == ".hg"
                || name_str == ".cache"
                || name_str == ".idea"
                || name_str == ".DS_Store"
                || name_str == "node_modules"
                || name_str == "target"
                || name_str == "dist"
                || name_str == "build"
                || name_str == "vendor"
                || name_str == ".gemini"
                || name_str == "references"
                || name_str == ".next"
                || name_str == ".turbo"
                || name_str == ".svelte-kit"
                || name_str == "out"
            {
                continue;
            }

            if path.is_dir() {
                walk_dir(&path, workspace, lower_query, matches, max_matches);
            } else if path.is_file() {
                let lower_name = name_str.to_lowercase();

                // Exclude noisy lockfiles, large data dictionaries, and minified bundles
                if lower_name == "package-lock.json"
                    || lower_name == "pnpm-lock.yaml"
                    || lower_name == "yarn.lock"
                    || lower_name == "cargo.lock"
                    || lower_name == "tokens.json"
                    || lower_name == "canonical_models.json"
                    || lower_name.ends_with(".old.json")
                    || lower_name.ends_with(".min.js")
                    || lower_name.ends_with(".min.css")
                    || lower_name.ends_with(".map")
                {
                    continue;
                }

                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let ext_lower = ext.to_lowercase();

                let is_config_file = lower_name == "dockerfile"
                    || lower_name.starts_with("dockerfile.")
                    || lower_name == ".dockerignore"
                    || lower_name == ".gitignore"
                    || lower_name == ".gitattributes"
                    || lower_name == ".editorconfig"
                    || lower_name == "makefile"
                    || lower_name == "justfile"
                    || lower_name == "procfile"
                    || lower_name.starts_with(".env");

                let is_searchable_ext = matches!(
                    ext_lower.as_str(),
                    "rs" | "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "json" | "toml" | "html"
                        | "css" | "scss" | "less" | "py" | "go" | "c" | "cpp" | "h" | "hpp" | "md"
                        | "yaml" | "yml" | "sh" | "bash" | "zsh" | "sql" | "xml" | "zig" | "java"
                        | "kt" | "swift" | "php" | "rb" | "dart" | "ex" | "exs" | "conf" | "cfg" | "ini"
                );

                if !is_searchable_ext && !is_config_file {
                    continue;
                }

                if let Ok(meta) = path.metadata() {
                    if meta.len() > 5_000_000 {
                        continue;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let rel_path = path.strip_prefix(workspace).unwrap_or(&path).to_string_lossy();
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(lower_query) {
                            let clean_line = line.trim();
                            let truncated = if clean_line.len() > 140 {
                                format!("{}...", &clean_line[..140])
                            } else {
                                clean_line.to_string()
                            };
                            matches.push(format!("{}:{}: {}", rel_path, line_idx + 1, truncated));
                            if matches.len() >= max_matches {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    walk_dir(workspace, workspace, &lower_query, &mut matches, 30);

    if matches.is_empty() {
        AgentToolResult {
            tool_name: "search_code".to_string(),
            ok: true,
            output: format!("No matches found for `{}` across workspace source files.", trimmed),
        }
    } else {
        AgentToolResult {
            tool_name: "search_code".to_string(),
            ok: true,
            output: format!(
                "Found {} match(es) for `{}`:\n{}",
                matches.len(),
                trimmed,
                matches.join("\n")
            ),
        }
    }
}
