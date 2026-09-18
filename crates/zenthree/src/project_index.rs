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
    // Only filter grammatical noise particles; preserve ALL potential identifiers, verbs, and filenames
    let grammatical_noise: HashSet<&str> = [
        "the", "a", "an", "and", "or", "in", "on", "at", "to", "for", "with",
        "about", "is", "are", "was", "were", "be", "been", "being",
        "of", "by", "as", "from", "into"
    ].iter().cloned().collect();

    let mut keywords = Vec::new();
    let mut seen = HashSet::new();

    // Extract alphanumeric / identifier tokens
    let cleaned: String = prompt.chars().map(|c| {
        if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/' {
            c
        } else {
            ' '
        }
    }).collect();

    for token in cleaned.split_whitespace() {
        let trimmed = token.trim_matches('.').trim_matches('-').trim();
        let lower = trimmed.to_lowercase();
        if trimmed.len() >= 2 && !grammatical_noise.contains(lower.as_str()) {
            if seen.insert(lower.clone()) {
                keywords.push(trimmed.to_string());
            }
        }
    }

    keywords
}

/// Scans the workspace directory to find files, directory structure, and symbol references
pub fn search_project_context(
    workspace_root: &Path,
    prompt: &str,
    max_files: usize,
) -> ProjectContextSummary {
    let keywords = extract_prompt_keywords(prompt);

    let mut matched_files: Vec<FileMatch> = Vec::new();
    let mut symbol_locations: Vec<SymbolLocation> = Vec::new();

    let mut candidates = Vec::new();
    collect_source_files(workspace_root, &mut candidates, 0);

    if !keywords.is_empty() {
        for path in &candidates {
            let rel_path = path.strip_prefix(workspace_root)
                .unwrap_or(path)
                .to_string_lossy()
                .to_string();

            if let Ok(content) = std::fs::read_to_string(path) {
                let mut file_score = 0;
                let mut matched_keys = Vec::new();
                let mut snippets = Vec::new();

                for kw in &keywords {
                    let kw_lower = kw.to_lowercase();
                    let mut found_count = 0;
                    for (line_idx, line) in content.lines().enumerate() {
                        let line_lower = line.to_lowercase();
                        if line_lower.contains(&kw_lower) {
                            found_count += 1;
                            if snippets.len() < 4 {
                                snippets.push((line_idx + 1, line.trim().to_string()));
                            }

                            // Inspect syntax definitions (fn, struct, enum, let, def)
                            let trimmed_line = line.trim();
                            if trimmed_line.starts_with("fn ")
                                || trimmed_line.starts_with("pub fn ")
                                || trimmed_line.starts_with("struct ")
                                || trimmed_line.starts_with("pub struct ")
                                || trimmed_line.starts_with("enum ")
                                || trimmed_line.starts_with("pub enum ")
                                || trimmed_line.starts_with("def ")
                            {
                                symbol_locations.push(SymbolLocation {
                                    file_path: path.clone(),
                                    relative_path: rel_path.clone(),
                                    name: kw.clone(),
                                    kind: if trimmed_line.contains("fn ") || trimmed_line.contains("def ") {
                                        "fn".to_string()
                                    } else if trimmed_line.contains("struct ") {
                                        "struct".to_string()
                                    } else {
                                        "type".to_string()
                                    },
                                    line_number: line_idx + 1,
                                    snippet: trimmed_line.to_string(),
                                });
                            }
                        }
                    }

                    if found_count > 0 {
                        file_score += found_count;
                        matched_keys.push(kw.clone());
                    }
                }

                if file_score > 0 {
                    matched_files.push(FileMatch {
                        path: path.clone(),
                        relative_path: rel_path,
                        score: file_score,
                        matched_keywords: matched_keys,
                        snippet_lines: snippets,
                    });
                }
            }
        }

        // Sort matching files by relevance score (highest matches first)
        matched_files.sort_by(|a, b| b.score.cmp(&a.score));
        matched_files.truncate(max_files);
    }

    // Build synthesized prompt context string
    let mut out = String::new();

    // 1. Always provide the full workspace file tree structure so the AI understands project layout & conventions
    let tree_overview = generate_project_tree_overview(workspace_root, 4);
    if !tree_overview.is_empty() {
        out.push_str("### Project Workspace File Tree:\n```text\n");
        out.push_str(&tree_overview);
        out.push_str("\n```\n\n");
    }

    if !matched_files.is_empty() || !symbol_locations.is_empty() {
        out.push_str("### Project Workspace Code Context\n\n");

        if !symbol_locations.is_empty() {
            out.push_str("**Symbol Definitions & Syntax Tracing:**\n");
            for sym in symbol_locations.iter().take(6) {
                out.push_str(&format!(
                    "- `{}` (`{}`) in `{}:{}` -> `{}`\n",
                    sym.name, sym.kind, sym.relative_path, sym.line_number, sym.snippet
                ));
            }
            out.push('\n');
        }

        if !matched_files.is_empty() {
            out.push_str("**Relevant Project Files & Snippets:**\n");
            for f in &matched_files {
                out.push_str(&format!("- File `{}` (Keywords: {}):\n", f.relative_path, f.matched_keywords.join(", ")));
                for (line_num, snip) in &f.snippet_lines {
                    let truncated: String = snip.chars().take(120).collect();
                    out.push_str(&format!("    L{}: {}\n", line_num, truncated));
                }
            }
        }
    }

    ProjectContextSummary {
        matched_files,
        symbol_locations,
        formatted_prompt_context: out,
    }
}

/// Generates a clean, compact relative file paths overview of the project workspace
pub fn generate_project_tree_overview(workspace_root: &Path, max_depth: usize) -> String {
    let mut tree_lines = Vec::new();
    build_paths_recursive(workspace_root, workspace_root, &mut tree_lines, 0, max_depth);
    tree_lines.join("\n")
}

fn build_paths_recursive(
    workspace_root: &Path,
    current_dir: &Path,
    out: &mut Vec<String>,
    depth: usize,
    max_depth: usize,
) {
    if depth >= max_depth || out.len() > 150 {
        return;
    }

    let mut entries = match std::fs::read_dir(current_dir) {
        Ok(read_dir) => {
            let mut items: Vec<_> = read_dir.flatten().collect();
            items.sort_by_key(|e| {
                let is_dir = e.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
                (!is_dir, e.file_name()) // dirs first, then alphabetical
            });
            items
        }
        Err(_) => return,
    };

    // Filter out ignored / noisy directories, assets/fonts/icons, and hidden files
    entries.retain(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        let lower = name.to_lowercase();
        !name.starts_with('.')
            && name != "target"
            && name != "node_modules"
            && name != "dist"
            && name != "build"
            && name != "vendor"
            && name != "__pycache__"
            && name != ".git"
            && lower != "assets"
            && lower != "fonts"
            && lower != "icons"
            && lower != "themes"
    });

    for entry in entries {
        if out.len() > 150 {
            break;
        }

        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        let rel_path = entry.path().strip_prefix(workspace_root)
            .unwrap_or(&entry.path())
            .to_string_lossy()
            .to_string();

        if is_dir {
            out.push(format!("{}/", rel_path));
            build_paths_recursive(workspace_root, &entry.path(), out, depth + 1, max_depth);
        } else {
            // Filter out media, fonts, and lockfiles from token overview
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                let lower = ext.to_lowercase();
                if lower == "svg" || lower == "png" || lower == "jpg" || lower == "jpeg"
                    || lower == "ico" || lower == "webp" || lower == "gif" || lower == "otf"
                    || lower == "ttf" || lower == "woff" || lower == "woff2" || lower == "mp3"
                    || lower == "mp4" || lower == "wasm" || lower == "lock" {
                    continue;
                }
            }
            out.push(rel_path);
        }
    }
}

/// Recursively collect code files while ignoring noisy build directories
fn collect_source_files(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 7 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    let allowed_exts = [
        "rs", "toml", "py", "js", "jsx", "ts", "tsx", "vue", "svelte", "json", "md", "html", "css", "yaml", "yml", "go"
    ];

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name.starts_with('.') || name == "target" || name == "node_modules" || name == "dist" || name == "build" || name == "vendor" {
            continue;
        }

        if path.is_dir() {
            collect_source_files(&path, out, depth + 1);
        } else if path.is_file() {
            if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                if allowed_exts.contains(&ext) {
                    // Check file size < 500KB to avoid heavy files
                    if let Ok(meta) = entry.metadata() {
                        if meta.len() < 500_000 {
                            out.push(path);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_project_tree_overview() {
        let tmp = std::env::temp_dir().join("zenthree_tree_test");
        let _ = std::fs::create_dir_all(tmp.join("src/components"));
        let _ = std::fs::write(tmp.join("src/components/Navbar.tsx"), "// Navbar");
        let _ = std::fs::write(tmp.join("package.json"), "{}");

        let tree = generate_project_tree_overview(&tmp, 4);
        assert!(tree.contains("src/"));
        assert!(tree.contains("Navbar.tsx"));
        assert!(tree.contains("package.json"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
