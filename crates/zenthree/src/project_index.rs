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
        "of", "by", "as", "from", "into", "does", "do", "did", "have", "has", "had",
        "can", "could", "will", "would", "should", "what", "which", "who", "where",
        "when", "why", "how", "there", "this", "that", "these", "those", "it", "its",
        "you", "your", "me", "my", "we", "our", "they", "them", "their", "hello", "hi",
        "hey", "tell", "show", "give", "please", "like", "want", "need", "know", "think",
        "let", "us", "any", "all", "some", "more", "much", "many", "such", "no", "not",
        "explain", "explaining", "explanation", "project", "projects", "codebase",
        "repo", "repos", "repository", "workspace", "folder", "directory", "file", "files",
        "tree", "overview", "summary", "understand", "walkthrough"
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

                            // Inspect syntax definitions across all languages
                            let trimmed_line = line.trim();
                            let is_syntax_def = trimmed_line.starts_with("fn ")
                                || trimmed_line.starts_with("pub fn ")
                                || trimmed_line.starts_with("async fn ")
                                || trimmed_line.starts_with("pub async fn ")
                                || trimmed_line.starts_with("function ")
                                || trimmed_line.starts_with("export function ")
                                || trimmed_line.starts_with("async function ")
                                || trimmed_line.starts_with("export async function ")
                                || trimmed_line.starts_with("const ")
                                || trimmed_line.starts_with("export const ")
                                || trimmed_line.starts_with("let ")
                                || trimmed_line.starts_with("export let ")
                                || trimmed_line.starts_with("class ")
                                || trimmed_line.starts_with("export class ")
                                || trimmed_line.starts_with("struct ")
                                || trimmed_line.starts_with("pub struct ")
                                || trimmed_line.starts_with("enum ")
                                || trimmed_line.starts_with("pub enum ")
                                || trimmed_line.starts_with("trait ")
                                || trimmed_line.starts_with("pub trait ")
                                || trimmed_line.starts_with("interface ")
                                || trimmed_line.starts_with("export interface ")
                                || trimmed_line.starts_with("type ")
                                || trimmed_line.starts_with("export type ")
                                || trimmed_line.starts_with("def ")
                                || trimmed_line.starts_with("async def ")
                                || trimmed_line.starts_with("func ")
                                || trimmed_line.starts_with("fun ")
                                || trimmed_line.starts_with("val ")
                                || trimmed_line.starts_with("var ")
                                || trimmed_line.starts_with("public ")
                                || trimmed_line.starts_with("private ")
                                || trimmed_line.starts_with("protected ");

                            if is_syntax_def {
                                let kind = if trimmed_line.contains("fn ")
                                    || trimmed_line.contains("def ")
                                    || trimmed_line.contains("function ")
                                    || trimmed_line.contains("func ")
                                    || trimmed_line.contains("fun ")
                                {
                                    "fn".to_string()
                                } else if trimmed_line.contains("struct ")
                                    || trimmed_line.contains("class ")
                                    || trimmed_line.contains("interface ")
                                    || trimmed_line.contains("trait ")
                                {
                                    "struct".to_string()
                                } else if trimmed_line.contains("enum ") {
                                    "enum".to_string()
                                } else if trimmed_line.contains("const ") || trimmed_line.contains("val ") {
                                    "const".to_string()
                                } else {
                                    "type".to_string()
                                };

                                symbol_locations.push(SymbolLocation {
                                    file_path: path.clone(),
                                    relative_path: rel_path.clone(),
                                    name: kw.clone(),
                                    kind,
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

    // 2. Discover detected subprojects in workspace
    let subprojects = detect_workspace_subprojects(workspace_root);
    if !subprojects.is_empty() {
        out.push_str("### Detected Subprojects in Workspace:\n");
        for (dir, manifest) in &subprojects {
            out.push_str(&format!("- Subproject `{}` (Manifest: `{}`)\n", dir, manifest));
        }
        out.push_str("NOTE: Application code, dependencies, and build/dev commands for these projects should target their respective subproject directory.\n\n");
    }

    // 3. If README exists, inject first 40 lines as immediate project overview (ideal for first-time project queries)
    for readme_name in ["README.md", "readme.md", "README", "readme.txt"] {
        let rpath = workspace_root.join(readme_name);
        if rpath.is_file() {
            if let Ok(content) = std::fs::read_to_string(&rpath) {
                let sample: Vec<&str> = content.lines().take(40).collect();
                if !sample.is_empty() {
                    out.push_str("### Project Overview (README excerpt):\n```markdown\n");
                    out.push_str(&sample.join("\n"));
                    out.push_str("\n```\n\n");
                    break;
                }
            }
        }
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

    // If no specific files matched the user's prompt (e.g. high-level question), highlight key manifests & entry points
    if matched_files.is_empty() && !candidates.is_empty() {
        let mut key_files = Vec::new();
        for cand in &candidates {
            let fname = cand.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let fname_lower = fname.to_lowercase();
            let is_manifest = fname_lower == "package.json"
                || fname_lower == "cargo.toml"
                || fname_lower == "pyproject.toml"
                || fname_lower == "requirements.txt"
                || fname_lower == "pipfile"
                || fname_lower == "setup.py"
                || fname_lower == "go.mod"
                || fname_lower == "cmakelists.txt"
                || fname_lower == "makefile"
                || fname_lower == "pom.xml"
                || fname_lower == "build.gradle"
                || fname_lower == "build.gradle.kts"
                || fname_lower == "composer.json"
                || fname_lower == "gemfile"
                || fname_lower == "pubspec.yaml"
                || fname_lower == "build.zig"
                || fname_lower == "mix.exs"
                || fname_lower == "package.swift"
                || fname_lower == "dockerfile"
                || fname_lower == "docker-compose.yml"
                || fname_lower == "docker-compose.yaml"
                || fname_lower == "deno.json"
                || fname_lower == "deno.jsonc"
                || fname_lower == "vite.config.ts"
                || fname_lower == "vite.config.js"
                || fname_lower == "next.config.js"
                || fname_lower == "next.config.mjs"
                || fname_lower == "tsconfig.json";

            let is_generic_config = fname_lower.ends_with(".toml")
                || fname_lower.ends_with(".yaml")
                || fname_lower.ends_with(".yml")
                || fname_lower.ends_with(".json")
                || fname_lower.ends_with(".config")
                || fname_lower.ends_with(".cfg")
                || fname_lower.ends_with(".ini");

            let is_generic_entrypoint = fname_lower.starts_with("main.")
                || fname_lower.starts_with("app.")
                || fname_lower.starts_with("index.")
                || fname_lower.starts_with("server.")
                || fname_lower.starts_with("entry.")
                || fname_lower.starts_with("mod.")
                || fname_lower.starts_with("lib.");

            if is_manifest || is_generic_config || is_generic_entrypoint || fname_lower.ends_with(".csproj") || fname_lower.ends_with(".sln") {
                let rel = cand.strip_prefix(workspace_root).unwrap_or(cand).to_string_lossy().to_string();
                key_files.push(rel);
            }
        }
        if !key_files.is_empty() {
            out.push_str("### Discovered Project Entry Points & Manifests:\n");
            for kf in key_files.iter().take(15) {
                out.push_str(&format!("- `{}`\n", kf));
            }
            out.push('\n');
        }
    }

    ProjectContextSummary {
        matched_files,
        symbol_locations,
        formatted_prompt_context: out,
    }
}

/// Discovers any subproject directories within depth 1-2 that contain manifest files.
pub fn detect_workspace_subprojects(workspace_root: &Path) -> Vec<(String, String)> {
    let manifests = [
        "package.json", "Cargo.toml", "pyproject.toml", "go.mod", "CMakeLists.txt",
        "pubspec.yaml", "pom.xml", "build.gradle", "build.gradle.kts", "deno.json",
        "composer.json", "Gemfile", "Makefile",
    ];
    let mut subprojects = Vec::new();
    if let Ok(entries) = std::fs::read_dir(workspace_root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name == "node_modules" || name == "target" || name == ".git" || name == ".zenthree" {
                continue;
            }
            if path.is_dir() {
                for m in &manifests {
                    let mpath = path.join(m);
                    if mpath.is_file() {
                        let rel_dir = format!("{}/", name);
                        let rel_manifest = format!("{}/{}", name, m);
                        subprojects.push((rel_dir, rel_manifest));
                        break;
                    }
                }
            }
        }
    }
    subprojects
}

/// Generates a visual file tree overview of the project workspace
pub fn generate_project_tree_overview(workspace_root: &Path, max_depth: usize) -> String {
    let mut tree_lines = Vec::new();
    tree_lines.push(".".to_string());
    build_tree_recursive(workspace_root, &mut tree_lines, "", 0, max_depth);
    tree_lines.join("\n")
}

fn build_tree_recursive(
    dir: &Path,
    out: &mut Vec<String>,
    prefix: &str,
    depth: usize,
    max_depth: usize,
) {
    if depth >= max_depth || out.len() > 250 {
        return;
    }

    let mut entries = match std::fs::read_dir(dir) {
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

    // Filter out ignored / noisy directories
    entries.retain(|e| {
        let name = e.file_name().to_string_lossy().to_string();
        name != ".git"
            && name != ".zenthree"
            && name != ".svn"
            && name != ".hg"
            && name != ".cache"
            && name != ".idea"
            && name != ".DS_Store"
            && name != "Thumbs.db"
            && name != "target"
            && name != "node_modules"
            && name != "dist"
            && name != "build"
            && name != "vendor"
            && name != "references"
            && name != ".next"
            && name != ".turbo"
            && name != ".svelte-kit"
            && name != ".nuxt"
            && name != "out"
            && name != "coverage"
            && name != ".gemini"
    });

    let count = entries.len();
    for (i, entry) in entries.iter().enumerate() {
        if out.len() > 250 {
            out.push(format!("{}└── ... (more files - use list_directory to explore)", prefix));
            break;
        }

        let is_last = i == count - 1;
        let branch = if is_last { "└── " } else { "├── " };
        let next_prefix = format!("{}{}", prefix, if is_last { "    " } else { "│   " });

        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);

        if is_dir {
            out.push(format!("{}{}{}/", prefix, branch, name));
            build_tree_recursive(&entry.path(), out, &next_prefix, depth + 1, max_depth);
        } else {
            out.push(format!("{}{}{}", prefix, branch, name));
        }
    }
}

/// Recursively collect code files while ignoring noisy build directories and binary media
fn collect_source_files(dir: &Path, out: &mut Vec<PathBuf>, depth: usize) {
    if depth > 7 {
        return;
    }
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    // Exclude binary, media, and compiled artifact files
    let binary_exts = [
        "png", "jpg", "jpeg", "gif", "ico", "webp", "avif", "bmp", "tiff",
        "mp3", "wav", "ogg", "flac", "aac", "mp4", "mkv", "avi", "mov", "webm",
        "zip", "tar", "gz", "bz2", "xz", "7z", "rar", "zst",
        "exe", "dll", "so", "dylib", "bin", "o", "a", "obj", "lib", "wasm",
        "class", "pyc", "pyo", "pyd", "elc",
        "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx",
        "ttf", "otf", "woff", "woff2", "eot",
        "db", "sqlite", "sqlite3", "parquet",
        "lockb"
    ];

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();

        if name == ".git"
            || name == ".zenthree"
            || name == ".svn"
            || name == ".hg"
            || name == ".cache"
            || name == ".idea"
            || name == ".DS_Store"
            || name == "Thumbs.db"
            || name == "target"
            || name == "node_modules"
            || name == "dist"
            || name == "build"
            || name == "vendor"
            || name == "references"
            || name == ".next"
            || name == ".turbo"
            || name == ".svelte-kit"
            || name == ".nuxt"
            || name == "out"
            || name == "coverage"
            || name == ".gemini"
        {
            continue;
        }

        if path.is_dir() {
            collect_source_files(&path, out, depth + 1);
        } else if path.is_file() {
            let lower_name = name.to_lowercase();

            // Exclude lockfiles, large data dictionaries, and minified bundles
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

            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            let ext_lower = ext.to_lowercase();
            let is_binary = binary_exts.contains(&ext_lower.as_str());

            if !is_binary {
                // Collect any non-binary text/source file under 5MB (handles 50,000+ line files)
                if let Ok(meta) = entry.metadata() {
                    if meta.len() > 0 && meta.len() < 5_000_000 {
                        out.push(path);
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

    #[test]
    fn test_custom_and_future_languages_collected_and_indexed() {
        let tmp = std::env::temp_dir().join("zenthree_polyglot_future_test");
        let _ = std::fs::create_dir_all(tmp.join("src"));
        // 1. Write an unknown future language file (e.g. .gleam, .odin, .customdsl)
        let _ = std::fs::write(tmp.join("gleam.toml"), "name = \"my_gleam_app\"\n");
        let _ = std::fs::write(tmp.join("src/main.gleam"), "pub fn compute_result() -> Int {\n    42\n}\n");
        let _ = std::fs::write(tmp.join("engine.odin"), "package main\nimport \"core:fmt\"\n");
        let _ = std::fs::write(tmp.join("rules.customdsl"), "rule ValidateUser {\n    check_active: true\n}\n");
        // 2. Binary file that should be skipped
        let _ = std::fs::write(tmp.join("logo.png"), &[0x89, 0x50, 0x4E, 0x47]);

        let mut collected = Vec::new();
        collect_source_files(&tmp, &mut collected, 0);

        let file_names: Vec<String> = collected.iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        // All text / source code files must be collected regardless of language
        assert!(file_names.contains(&"gleam.toml".to_string()));
        assert!(file_names.contains(&"main.gleam".to_string()));
        assert!(file_names.contains(&"engine.odin".to_string()));
        assert!(file_names.contains(&"rules.customdsl".to_string()));

        // Binary files must be excluded
        assert!(!file_names.contains(&"logo.png".to_string()));

        // Index searching must locate the symbol and entrypoint
        let summary = search_project_context(&tmp, "compute_result", 5);
        assert!(summary.formatted_prompt_context.contains("main.gleam"));
        assert!(summary.formatted_prompt_context.contains("compute_result"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_first_time_project_open_immediate_context() {
        let tmp = std::env::temp_dir().join(format!("zenthree_first_open_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(tmp.join(".github").join("workflows"));
        let _ = std::fs::create_dir_all(tmp.join("src"));

        // Setup a newly cloned project with README, Dockerfile, GitHub Actions, and entrypoint
        let _ = std::fs::write(tmp.join("README.md"), "# Quantum Engine\nA distributed high-performance quantum computing simulator.\n\n## Quickstart\nRun `cargo run` to start the engine.\n");
        let _ = std::fs::write(tmp.join("Dockerfile"), "FROM rust:1.80\n");
        let _ = std::fs::write(tmp.join("Cargo.toml"), "[package]\nname = \"quantum-engine\"\n");
        let _ = std::fs::write(tmp.join("src/main.rs"), "fn main() {\n    println!(\"quantum ready\");\n}\n");
        let _ = std::fs::write(tmp.join(".github/workflows/ci.yml"), "name: CI\n");

        // When a user asks a high-level question on first opening (e.g. "what is this project?")
        let summary = search_project_context(&tmp, "what is this project and how do i run it?", 5);
        let ctx = &summary.formatted_prompt_context;

        // 1. Must include full workspace tree with subfolders and dot-configs
        assert!(ctx.contains("Project Workspace File Tree"));
        assert!(ctx.contains("README.md"));
        assert!(ctx.contains(".github/"));
        assert!(ctx.contains("ci.yml"));
        assert!(ctx.contains("Dockerfile"));
        assert!(ctx.contains("src/"));
        assert!(ctx.contains("main.rs"));

        // 2. Must include the README overview excerpt
        assert!(ctx.contains("Project Overview (README excerpt)"));
        assert!(ctx.contains("Quantum Engine"));
        assert!(ctx.contains("distributed high-performance quantum computing simulator"));
        assert!(ctx.contains("Run `cargo run` to start the engine"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_broad_project_explanation_query_does_not_grepp_noise() {
        // "explain me this project" should extract zero noisy symbols
        let kw = extract_prompt_keywords("explain me this project");
        assert!(kw.is_empty(), "Expected empty keywords for broad project explanation, got: {:?}", kw);

        let kw2 = extract_prompt_keywords("explain the codebase architecture");
        assert_eq!(kw2, vec!["architecture"]);

        let tmp = std::env::temp_dir().join(format!("zenthree_explain_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(tmp.join("src"));
        let _ = std::fs::write(tmp.join("README.md"), "# Zenthree Core\nNext-gen agentic IDE.");
        let _ = std::fs::write(tmp.join("src/lib.rs"), "pub fn start() {}\n");

        let summary = search_project_context(&tmp, "explain me this project", 5);
        // It must NOT match random files via blind keyword search
        assert!(summary.matched_files.is_empty());
        // But it MUST provide the workspace tree and README overview
        assert!(summary.formatted_prompt_context.contains("Project Workspace File Tree"));
        assert!(summary.formatted_prompt_context.contains("Zenthree Core"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
