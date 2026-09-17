use std::path::{Path, PathBuf};
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct SymbolLocation {
    pub file_path: PathBuf,
    pub relative_path: String,
    pub name: String,
    pub kind: String, // "fn", "struct", "enum", "type", "variable"
    pub line_number: usize,
    pub snippet: String,
}

#[derive(Debug, Clone)]
pub struct FileMatch {
    pub path: PathBuf,
    pub relative_path: String,
    pub score: usize,
    pub matched_keywords: Vec<String>,
    pub snippet_lines: Vec<(usize, String)>,
}

#[derive(Debug, Clone)]
pub struct ProjectContextSummary {
    pub matched_files: Vec<FileMatch>,
    pub symbol_locations: Vec<SymbolLocation>,
    pub formatted_prompt_context: String,
}

/// Extracts meaningful search keywords, symbol names, and identifiers from user prompt
pub fn extract_prompt_keywords(prompt: &str) -> Vec<String> {
    let stopwords: HashSet<&str> = [
        "the", "a", "an", "and", "or", "in", "on", "at", "to", "for", "with",
        "about", "is", "are", "was", "were", "be", "been", "being", "have", "has",
        "had", "do", "does", "did", "can", "could", "will", "would", "should",
        "please", "help", "me", "how", "what", "where", "why", "who", "which",
        "this", "that", "these", "those", "from", "into", "out", "code", "file",
        "files", "project", "folder", "directory", "show", "tell", "explain",
        "modify", "change", "update", "add", "remove", "delete", "fix", "write",
        "need", "want", "like", "make", "create", "also", "now", "here", "there",
        "you", "your", "it", "its"
    ].iter().cloned().collect();

    let mut keywords = Vec::new();
    let mut seen = HashSet::new();

    let cleaned: String = prompt.chars().map(|c| {
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
            c
        } else {
            ' '
        }
    }).collect();

    for token in cleaned.split_whitespace() {
        let trimmed = token.trim_matches('.').trim_matches('-').trim();
        let lower = trimmed.to_lowercase();
        if trimmed.len() >= 2 && !stopwords.contains(lower.as_str()) {
            if seen.insert(lower.clone()) {
                keywords.push(lower);
            }
        }
    }

    keywords
}

pub fn build_project_context(
    workspace_root: &Path,
    prompt: &str,
    max_files: usize,
) -> ProjectContextSummary {
    let keywords = extract_prompt_keywords(prompt);
    let mut matched_files = Vec::new();

    fn scan_dir(
        dir: &Path,
        root: &Path,
        keywords: &[String],
        matched_files: &mut Vec<FileMatch>,
        max_files: usize,
    ) {
        if matched_files.len() >= max_files {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            if matched_files.len() >= max_files {
                break;
            }

            let path = entry.path();
            let fname = entry.file_name().to_string_lossy().to_string();

            if fname.starts_with('.') || fname == "node_modules" || fname == "target" || fname == "dist" || fname == "build" {
                continue;
            }

            if path.is_dir() {
                scan_dir(&path, root, keywords, matched_files, max_files);
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let is_code = matches!(
                    ext,
                    "rs" | "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "json" | "toml" | "html"
                        | "css" | "py" | "go" | "c" | "cpp" | "h" | "md"
                );

                if !is_code {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let mut score = 0;
                    let mut matched_kw = Vec::new();
                    let mut snippet_lines = Vec::new();

                    for kw in keywords {
                        if fname.to_lowercase().contains(kw) {
                            score += 5;
                            if !matched_kw.contains(kw) {
                                matched_kw.push(kw.clone());
                            }
                        }
                    }

                    for (line_idx, line) in content.lines().enumerate() {
                        let lower_line = line.to_lowercase();
                        for kw in keywords {
                            if lower_line.contains(kw) {
                                score += 1;
                                if !matched_kw.contains(kw) {
                                    matched_kw.push(kw.clone());
                                }
                                if snippet_lines.len() < 3 {
                                    snippet_lines.push((line_idx + 1, line.trim().to_string()));
                                }
                            }
                        }
                    }

                    if score > 0 {
                        let rel_path = path.strip_prefix(root).unwrap_or(&path).to_string_lossy().to_string();
                        matched_files.push(FileMatch {
                            path,
                            relative_path: rel_path,
                            score,
                            matched_keywords: matched_kw,
                            snippet_lines,
                        });
                    }
                }
            }
        }
    }

    scan_dir(workspace_root, workspace_root, &keywords, &mut matched_files, max_files);
    matched_files.sort_by(|a, b| b.score.cmp(&a.score));
    matched_files.truncate(max_files);

    let mut formatted = String::new();
    if !matched_files.is_empty() {
        formatted.push_str("### Relevant Workspace Context:\n");
        for f in &matched_files {
            formatted.push_str(&format!("- `{}`:\n", f.relative_path));
            for (line_no, snip) in &f.snippet_lines {
                formatted.push_str(&format!("  {:4} | {}\n", line_no, snip));
            }
        }
        formatted.push('\n');
    }

    ProjectContextSummary {
        matched_files,
        symbol_locations: Vec::new(),
        formatted_prompt_context: formatted,
    }
}
