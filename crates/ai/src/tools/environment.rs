use std::path::{Path, PathBuf};
use crate::types::{ProjectEnvironment, ProjectLanguage, WorkspaceDetectionContext};
use super::AgentToolResult;

pub struct DetectorAgent;

impl DetectorAgent {
    /// Inspects the workspace root directory structure, file extensions, and candidate manifests.
    pub fn inspect_workspace_context(workspace_root: &Path) -> WorkspaceDetectionContext {
        let mut file_tree_sample = Vec::new();
        let mut ext_counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut manifest_candidates = Vec::new();
        let mut readme_sample = None;

        let ignored_dirs = [
            ".git", "target", "node_modules", "dist", "build", ".cache",
            "venv", ".venv", "__pycache__", ".idea", ".vscode", "vendor",
            ".next", ".nuxt", "bin", "obj", ".cargo",
        ];

        let manifest_names = [
            "Cargo.toml", "package.json", "tsconfig.json", "deno.json",
            "pyproject.toml", "requirements.txt", "setup.py", "Pipfile",
            "go.mod", "CMakeLists.txt", "Makefile", "build.zig", "build.zig.zon",
            "build.gradle", "build.gradle.kts", "pom.xml", "settings.gradle",
            "Package.swift", "mix.exs", "rebar.config", "dune-project", "shard.yml",
            "project.clj", "deps.edn", "justfile", "Taskfile.yml", "Dockerfile",
            "docker-compose.yml", "Gemfile", "Rakefile", "composer.json",
            "pubspec.yaml", "flake.nix", "default.nix",
        ];

        let mut dirs_to_visit = vec![(workspace_root.to_path_buf(), 0usize)];
        while let Some((dir, depth)) = dirs_to_visit.pop() {
            if let Ok(entries) = std::fs::read_dir(&dir) {
                let mut sub_entries: Vec<_> = entries.flatten().collect();
                sub_entries.sort_by_key(|e| e.file_name());

                for entry in sub_entries {
                    let path = entry.path();
                    let fname = entry.file_name().to_string_lossy().to_string();

                    if ignored_dirs.iter().any(|&ig| fname == ig) {
                        continue;
                    }

                    let rel_path = path
                        .strip_prefix(workspace_root)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();

                    if path.is_dir() {
                        if depth < 2 {
                            dirs_to_visit.push((path, depth + 1));
                        }
                    } else if path.is_file() {
                        if manifest_names.iter().any(|&m| fname.eq_ignore_ascii_case(m)) {
                            manifest_candidates.push(rel_path.clone());
                        }

                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            let clean_ext = ext.to_lowercase();
                            if !clean_ext.is_empty() && clean_ext.len() <= 10 {
                                *ext_counts.entry(clean_ext).or_insert(0) += 1;
                            }
                        }

                        if readme_sample.is_none()
                            && (fname.eq_ignore_ascii_case("readme.md")
                                || fname.eq_ignore_ascii_case("readme")
                                || fname.eq_ignore_ascii_case("readme.txt"))
                        {
                            if let Ok(text) = std::fs::read_to_string(&path) {
                                let excerpt: Vec<&str> = text.lines().take(20).collect();
                                let mut joined = excerpt.join("\n");
                                if joined.len() > 1200 {
                                    joined.truncate(1200);
                                    joined.push_str("...");
                                }
                                readme_sample = Some(joined);
                            }
                        }

                        if file_tree_sample.len() < 60 {
                            file_tree_sample.push(rel_path);
                        }
                    }
                }
            }
        }

        let mut file_extensions: Vec<(String, usize)> = ext_counts.into_iter().collect();
        file_extensions.sort_by(|a, b| b.1.cmp(&a.1));
        file_extensions.truncate(8);

        WorkspaceDetectionContext {
            file_tree_sample,
            file_extensions,
            manifest_candidates,
            readme_sample,
        }
    }

    /// Inspects the workspace root to deduce language, build system, compiler check command, and test command
    pub fn detect(workspace_root: &Path) -> ProjectEnvironment {
        // 1. Rust workspace or crate
        if workspace_root.join("Cargo.toml").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Rust,
                build_system: "Cargo".to_string(),
                compiler_check_cmd: Some(vec![
                    "cargo".to_string(),
                    "check".to_string(),
                    "--message-format=short".to_string(),
                ]),
                test_cmd: Some(vec!["cargo".to_string(), "test".to_string()]),
                details: "Rust project managed via Cargo (Cargo.toml)".to_string(),
            };
        }

        // 2. TypeScript or JavaScript project
        let has_tsconfig = workspace_root.join("tsconfig.json").exists();
        let has_pkg_json = workspace_root.join("package.json").exists();
        let has_deno = workspace_root.join("deno.json").exists() || workspace_root.join("deno.jsonc").exists();

        if has_tsconfig || has_pkg_json || has_deno {
            let is_ts = has_tsconfig
                || workspace_root.join("src").join("index.ts").exists()
                || workspace_root.join("index.ts").exists();
            let language = if is_ts {
                ProjectLanguage::TypeScript
            } else {
                ProjectLanguage::JavaScript
            };

            if has_deno {
                return ProjectEnvironment {
                    language,
                    build_system: "Deno".to_string(),
                    compiler_check_cmd: Some(vec!["deno".to_string(), "check".to_string()]),
                    test_cmd: Some(vec!["deno".to_string(), "test".to_string()]),
                    details: "Deno project detected (deno.json)".to_string(),
                };
            }

            let (pkg_mgr, check_cmd, test_cmd) = if workspace_root.join("bun.lockb").exists()
                || workspace_root.join("bun.lock").exists()
            {
                (
                    "Bun",
                    if is_ts {
                        Some(vec![
                            "bun".to_string(),
                            "x".to_string(),
                            "tsc".to_string(),
                            "--noEmit".to_string(),
                        ])
                    } else {
                        None
                    },
                    Some(vec!["bun".to_string(), "test".to_string()]),
                )
            } else if workspace_root.join("pnpm-lock.yaml").exists() {
                (
                    "pnpm",
                    if is_ts {
                        Some(vec![
                            "pnpm".to_string(),
                            "exec".to_string(),
                            "tsc".to_string(),
                            "--noEmit".to_string(),
                        ])
                    } else {
                        None
                    },
                    Some(vec!["pnpm".to_string(), "test".to_string()]),
                )
            } else if workspace_root.join("yarn.lock").exists() {
                (
                    "Yarn",
                    if is_ts {
                        Some(vec![
                            "yarn".to_string(),
                            "tsc".to_string(),
                            "--noEmit".to_string(),
                        ])
                    } else {
                        None
                    },
                    Some(vec!["yarn".to_string(), "test".to_string()]),
                )
            } else {
                (
                    "npm",
                    if is_ts {
                        Some(vec![
                            "npx".to_string(),
                            "tsc".to_string(),
                            "--noEmit".to_string(),
                        ])
                    } else {
                        None
                    },
                    Some(vec!["npm".to_string(), "test".to_string()]),
                )
            };

            return ProjectEnvironment {
                language,
                build_system: pkg_mgr.to_string(),
                compiler_check_cmd: check_cmd,
                test_cmd,
                details: format!(
                    "{} project managed via {} (package.json/tsconfig.json)",
                    if is_ts { "TypeScript" } else { "JavaScript" },
                    pkg_mgr
                ),
            };
        }

        // 3. Python project
        if workspace_root.join("pyproject.toml").exists()
            || workspace_root.join("requirements.txt").exists()
            || workspace_root.join("setup.py").exists()
            || workspace_root.join("Pipfile").exists()
        {
            let build_sys = if workspace_root.join("pyproject.toml").exists() {
                "pyproject.toml"
            } else if workspace_root.join("Pipfile").exists() {
                "Pipfile"
            } else {
                "requirements.txt"
            };

            return ProjectEnvironment {
                language: ProjectLanguage::Python,
                build_system: build_sys.to_string(),
                compiler_check_cmd: Some(vec!["mypy".to_string(), ".".to_string()]),
                test_cmd: Some(vec!["pytest".to_string()]),
                details: format!("Python project detected ({})", build_sys),
            };
        }

        // 4. Go project
        if workspace_root.join("go.mod").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Go,
                build_system: "go modules".to_string(),
                compiler_check_cmd: Some(vec!["go".to_string(), "vet".to_string(), "./...".to_string()]),
                test_cmd: Some(vec!["go".to_string(), "test".to_string(), "./...".to_string()]),
                details: "Go module project detected (go.mod)".to_string(),
            };
        }

        // 5. C / C++ project
        if workspace_root.join("CMakeLists.txt").exists() || workspace_root.join("Makefile").exists() {
            let has_cmake = workspace_root.join("CMakeLists.txt").exists();
            return ProjectEnvironment {
                language: ProjectLanguage::Cpp,
                build_system: if has_cmake { "CMake".to_string() } else { "Make".to_string() },
                compiler_check_cmd: if has_cmake {
                    Some(vec!["cmake".to_string(), "--build".to_string(), ".".to_string()])
                } else {
                    Some(vec!["make".to_string(), "-q".to_string()])
                },
                test_cmd: if has_cmake {
                    Some(vec!["ctest".to_string()])
                } else {
                    Some(vec!["make".to_string(), "test".to_string()])
                },
                details: if has_cmake {
                    "C/C++ project managed via CMake"
                } else {
                    "C/C++ project managed via Makefile"
                }
                .to_string(),
            };
        }

        // 6. Zig project
        if workspace_root.join("build.zig").exists() || workspace_root.join("build.zig.zon").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Zig,
                build_system: "Zig".to_string(),
                compiler_check_cmd: Some(vec!["zig".to_string(), "build".to_string()]),
                test_cmd: Some(vec!["zig".to_string(), "build".to_string(), "test".to_string()]),
                details: "Zig project detected (build.zig)".to_string(),
            };
        }

        // 7. Java / Kotlin project
        if workspace_root.join("pom.xml").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Java,
                build_system: "Maven".to_string(),
                compiler_check_cmd: Some(vec!["mvn".to_string(), "compile".to_string()]),
                test_cmd: Some(vec!["mvn".to_string(), "test".to_string()]),
                details: "Java project managed via Maven (pom.xml)".to_string(),
            };
        }
        if workspace_root.join("build.gradle.kts").exists() || workspace_root.join("build.gradle").exists() {
            let is_kt = workspace_root.join("build.gradle.kts").exists()
                || workspace_root.join("src/main/kotlin").exists();
            return ProjectEnvironment {
                language: if is_kt { ProjectLanguage::Kotlin } else { ProjectLanguage::Java },
                build_system: "Gradle".to_string(),
                compiler_check_cmd: Some(vec!["gradle".to_string(), "classes".to_string()]),
                test_cmd: Some(vec!["gradle".to_string(), "test".to_string()]),
                details: format!("{} project managed via Gradle", if is_kt { "Kotlin" } else { "Java" }),
            };
        }

        // 8. PHP project
        if workspace_root.join("composer.json").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::PHP,
                build_system: "Composer".to_string(),
                compiler_check_cmd: None,
                test_cmd: Some(vec!["composer".to_string(), "test".to_string()]),
                details: "PHP project managed via Composer (composer.json)".to_string(),
            };
        }

        // 9. Ruby project
        if workspace_root.join("Gemfile").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Ruby,
                build_system: "Bundler".to_string(),
                compiler_check_cmd: None,
                test_cmd: Some(vec!["bundle".to_string(), "exec".to_string(), "rake".to_string(), "test".to_string()]),
                details: "Ruby project managed via Bundler (Gemfile)".to_string(),
            };
        }

        // 10. Dart / Flutter project
        if workspace_root.join("pubspec.yaml").exists() {
            let is_flutter = if let Ok(content) = std::fs::read_to_string(workspace_root.join("pubspec.yaml")) {
                content.contains("flutter:")
            } else {
                false
            };
            return ProjectEnvironment {
                language: ProjectLanguage::Dart,
                build_system: if is_flutter { "Flutter".to_string() } else { "Dart".to_string() },
                compiler_check_cmd: Some(if is_flutter { vec!["flutter".to_string(), "analyze".to_string()] } else { vec!["dart".to_string(), "analyze".to_string()] }),
                test_cmd: Some(if is_flutter { vec!["flutter".to_string(), "test".to_string()] } else { vec!["dart".to_string(), "test".to_string()] }),
                details: format!("{} project detected (pubspec.yaml)", if is_flutter { "Flutter" } else { "Dart" }),
            };
        }

        // 11. Swift project
        if workspace_root.join("Package.swift").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Swift,
                build_system: "SwiftPM".to_string(),
                compiler_check_cmd: Some(vec!["swift".to_string(), "build".to_string()]),
                test_cmd: Some(vec!["swift".to_string(), "test".to_string()]),
                details: "Swift package detected (Package.swift)".to_string(),
            };
        }

        // 12. Elixir project
        if workspace_root.join("mix.exs").exists() {
            return ProjectEnvironment {
                language: ProjectLanguage::Elixir,
                build_system: "Mix".to_string(),
                compiler_check_cmd: Some(vec!["mix".to_string(), "compile".to_string()]),
                test_cmd: Some(vec!["mix".to_string(), "test".to_string()]),
                details: "Elixir project detected (mix.exs)".to_string(),
            };
        }

        // 13. Subfolder inspection: check if a manifest exists in a subfolder (e.g. monorepo or nested project)
        let ctx = Self::inspect_workspace_context(workspace_root);
        for manifest_rel in &ctx.manifest_candidates {
            let manifest_path = workspace_root.join(manifest_rel);
            if let Some(parent) = manifest_path.parent() {
                if parent != workspace_root {
                    let sub_env = Self::detect(parent);
                    if !sub_env.is_unidentified() {
                        let sub_rel = parent.strip_prefix(workspace_root).unwrap_or(parent).to_string_lossy().to_string();
                        return ProjectEnvironment {
                            details: format!("{} (in subfolder `{}`)", sub_env.details, sub_rel),
                            ..sub_env
                        };
                    }
                }
            }
        }

        // 14. Fallback: Check dominant file extensions if no explicit build manifest is found
        if let Some((top_ext, _)) = ctx.file_extensions.first() {
            match top_ext.as_str() {
                "py" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Python,
                        build_system: "Python".to_string(),
                        compiler_check_cmd: Some(vec!["python3".to_string(), "-m".to_string(), "py_compile".to_string()]),
                        test_cmd: Some(vec!["pytest".to_string()]),
                        details: "Standalone Python project detected via .py files".to_string(),
                    };
                }
                "rs" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Rust,
                        build_system: "Cargo".to_string(),
                        compiler_check_cmd: Some(vec!["cargo".to_string(), "check".to_string()]),
                        test_cmd: Some(vec!["cargo".to_string(), "test".to_string()]),
                        details: "Standalone Rust files detected".to_string(),
                    };
                }
                "ts" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::TypeScript,
                        build_system: "TypeScript".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone TypeScript files detected".to_string(),
                    };
                }
                "js" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::JavaScript,
                        build_system: "Node".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone JavaScript files detected".to_string(),
                    };
                }
                "go" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Go,
                        build_system: "Go".to_string(),
                        compiler_check_cmd: Some(vec!["go".to_string(), "vet".to_string(), "./...".to_string()]),
                        test_cmd: Some(vec!["go".to_string(), "test".to_string(), "./...".to_string()]),
                        details: "Standalone Go files detected".to_string(),
                    };
                }
                "c" | "cpp" | "cc" | "h" | "hpp" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Cpp,
                        build_system: "C/C++".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone C/C++ files detected".to_string(),
                    };
                }
                "zig" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Zig,
                        build_system: "Zig".to_string(),
                        compiler_check_cmd: Some(vec!["zig".to_string(), "build".to_string()]),
                        test_cmd: Some(vec!["zig".to_string(), "build".to_string(), "test".to_string()]),
                        details: "Standalone Zig files detected".to_string(),
                    };
                }
                "java" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Java,
                        build_system: "Java".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone Java files detected".to_string(),
                    };
                }
                "kt" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Kotlin,
                        build_system: "Kotlin".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone Kotlin files detected".to_string(),
                    };
                }
                "swift" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Swift,
                        build_system: "Swift".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone Swift files detected".to_string(),
                    };
                }
                "php" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::PHP,
                        build_system: "PHP".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone PHP files detected".to_string(),
                    };
                }
                "rb" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Ruby,
                        build_system: "Ruby".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone Ruby files detected".to_string(),
                    };
                }
                "dart" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Dart,
                        build_system: "Dart".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone Dart files detected".to_string(),
                    };
                }
                "ex" | "exs" => {
                    return ProjectEnvironment {
                        language: ProjectLanguage::Elixir,
                        build_system: "Elixir".to_string(),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: "Standalone Elixir files detected".to_string(),
                    };
                }
                other => {
                    let mut chars = other.chars();
                    let display_name = match chars.next() {
                        None => "Custom".to_string(),
                        Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
                    };
                    return ProjectEnvironment {
                        language: ProjectLanguage::Other(display_name.clone()),
                        build_system: format!("{} files", display_name),
                        compiler_check_cmd: None,
                        test_cmd: None,
                        details: format!("Project with .{} files detected", other),
                    };
                }
            }
        }

        // Default unknown
        ProjectEnvironment {
            language: ProjectLanguage::Other("Unknown".to_string()),
            build_system: "Unknown".to_string(),
            compiler_check_cmd: None,
            test_cmd: None,
            details: "No explicit build manifest detected".to_string(),
        }
    }

    /// Classifies an individual file using filename, extension, shebang, and content heuristics.
    /// If completely ambiguous, returns (type, method, Some(excerpt_for_llm)).
    pub fn classify_file(file_path: &Path) -> (String, String, Option<String>) {
        let content = std::fs::read_to_string(file_path).unwrap_or_default();
        let (name, method) = Self::classify_file_quick(file_path, &content);

        let excerpt = if method.contains("LLM") || name.contains("Unrecognized") {
            let sample: Vec<&str> = content.lines().take(30).collect();
            if !sample.is_empty() {
                Some(sample.join("\n"))
            } else {
                None
            }
        } else {
            None
        };

        (name, method, excerpt)
    }

    /// Fast classification using path and content
    pub fn classify_file_quick(file_path: &Path, content: &str) -> (String, String) {
        let fname = file_path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        let fname_lower = fname.to_lowercase();

        // 1. Check known filenames
        match fname_lower.as_str() {
            "dockerfile" => return ("Dockerfile".to_string(), "Filename".to_string()),
            "makefile" | "gnumakefile" => return ("Makefile".to_string(), "Filename".to_string()),
            "justfile" => return ("Justfile".to_string(), "Filename".to_string()),
            "gemfile" | "rakefile" => return ("Ruby / Bundler".to_string(), "Filename".to_string()),
            "procfile" => return ("Procfile".to_string(), "Filename".to_string()),
            "cmakelists.txt" => return ("CMake".to_string(), "Filename".to_string()),
            "cargo.toml" => return ("Rust / Cargo Manifest".to_string(), "Filename".to_string()),
            "package.json" => return ("Node / npm Manifest".to_string(), "Filename".to_string()),
            "pyproject.toml" => return ("Python Manifest".to_string(), "Filename".to_string()),
            "go.mod" => return ("Go Module Manifest".to_string(), "Filename".to_string()),
            _ => {}
        }

        // 2. Check Shebang
        if let Some(first_line) = content.lines().next() {
            let trimmed = first_line.trim();
            if trimmed.starts_with("#!") {
                if trimmed.contains("python") {
                    return ("Python Script".to_string(), "Shebang".to_string());
                }
                if trimmed.contains("bash") || trimmed.contains("sh") || trimmed.contains("zsh") {
                    return ("Shell Script".to_string(), "Shebang".to_string());
                }
                if trimmed.contains("node") || trimmed.contains("bun") || trimmed.contains("deno") {
                    return ("JavaScript / TypeScript Script".to_string(), "Shebang".to_string());
                }
                if trimmed.contains("ruby") {
                    return ("Ruby Script".to_string(), "Shebang".to_string());
                }
                if trimmed.contains("perl") {
                    return ("Perl Script".to_string(), "Shebang".to_string());
                }
                if trimmed.contains("php") {
                    return ("PHP Script".to_string(), "Shebang".to_string());
                }
            }
        }

        // 3. Check Extension
        if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
            let ext_lower = ext.to_lowercase();
            match ext_lower.as_str() {
                "rs" => return ("Rust".to_string(), "Extension".to_string()),
                "py" => return ("Python".to_string(), "Extension".to_string()),
                "js" | "jsx" | "mjs" | "cjs" => return ("JavaScript".to_string(), "Extension".to_string()),
                "ts" | "tsx" | "mts" | "cts" => return ("TypeScript".to_string(), "Extension".to_string()),
                "go" => return ("Go".to_string(), "Extension".to_string()),
                "c" | "h" => return ("C".to_string(), "Extension".to_string()),
                "cpp" | "cc" | "cxx" | "hpp" | "hxx" => return ("C++".to_string(), "Extension".to_string()),
                "html" | "htm" => return ("HTML".to_string(), "Extension".to_string()),
                "css" | "scss" | "less" => return ("CSS".to_string(), "Extension".to_string()),
                "json" | "jsonc" => return ("JSON".to_string(), "Extension".to_string()),
                "yaml" | "yml" => return ("YAML".to_string(), "Extension".to_string()),
                "toml" => return ("TOML".to_string(), "Extension".to_string()),
                "md" | "markdown" => return ("Markdown".to_string(), "Extension".to_string()),
                "sh" | "bash" | "zsh" => return ("Shell".to_string(), "Extension".to_string()),
                "zig" => return ("Zig".to_string(), "Extension".to_string()),
                "java" => return ("Java".to_string(), "Extension".to_string()),
                "kt" | "kts" => return ("Kotlin".to_string(), "Extension".to_string()),
                "swift" => return ("Swift".to_string(), "Extension".to_string()),
                "php" => return ("PHP".to_string(), "Extension".to_string()),
                "rb" => return ("Ruby".to_string(), "Extension".to_string()),
                "dart" => return ("Dart".to_string(), "Extension".to_string()),
                "ex" | "exs" => return ("Elixir".to_string(), "Extension".to_string()),
                "sql" => return ("SQL".to_string(), "Extension".to_string()),
                "xml" => return ("XML".to_string(), "Extension".to_string()),
                "gleam" => return ("Gleam".to_string(), "Extension".to_string()),
                "odin" => return ("Odin".to_string(), "Extension".to_string()),
                "mojo" => return ("Mojo".to_string(), "Extension".to_string()),
                "nim" => return ("Nim".to_string(), "Extension".to_string()),
                "jl" => return ("Julia".to_string(), "Extension".to_string()),
                "hs" => return ("Haskell".to_string(), "Extension".to_string()),
                _ => {}
            }
        }

        // 4. Content syntax markers
        for line in content.lines().take(50) {
            let trimmed = line.trim();
            if trimmed.starts_with("<?php") {
                return ("PHP".to_string(), "Syntax '<?php'".to_string());
            }
            if trimmed.starts_with("<!DOCTYPE html") || trimmed.starts_with("<html") {
                return ("HTML".to_string(), "Syntax '<html>'".to_string());
            }
            if trimmed.starts_with("<?xml") {
                return ("XML".to_string(), "Syntax '<?xml'".to_string());
            }
            if trimmed.starts_with("package ") && content.contains("func ") {
                return ("Go".to_string(), "Syntax 'package/func'".to_string());
            }
            if trimmed.starts_with("pub fn ") || (trimmed.starts_with("fn ") && content.contains("let mut ")) {
                return ("Rust".to_string(), "Syntax 'fn/pub fn'".to_string());
            }
            if (trimmed.starts_with("def ") || trimmed.starts_with("import ")) && content.contains("self") {
                return ("Python".to_string(), "Syntax 'def/import'".to_string());
            }
            if trimmed.starts_with("import React") || trimmed.starts_with("export default") {
                return ("JavaScript/TypeScript".to_string(), "Syntax 'import/export'".to_string());
            }
            if trimmed.starts_with("#include <") {
                return ("C/C++".to_string(), "Syntax '#include'".to_string());
            }
            if trimmed.starts_with("SELECT ") || trimmed.starts_with("CREATE TABLE") {
                return ("SQL".to_string(), "Syntax 'SQL'".to_string());
            }
        }

        // 5. Unrecognized / Ambiguous - Hand off to LLM
        ("Unrecognized / Custom".to_string(), "LLM Content Analysis Required".to_string())
    }
}

pub fn execute_detect_environment(workspace: &Path, subpath: Option<&str>) -> AgentToolResult {
    let target = match subpath {
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

    if target.is_file() {
        let (name, method, sample) = DetectorAgent::classify_file(&target);
        let mut out = format!(
            "File Analysis for `{}`:\nDetected Type: {}\nIdentification Method: {}",
            target.display(),
            name,
            method
        );
        if let Some(excerpt) = sample {
            out.push_str(&format!(
                "\n\n[Unrecognized Syntax - LLM Inspection Excerpt]:\n```\n{}\n```\n(AI Architect: Analyze the syntax patterns above to determine the file type, schema, or language).",
                excerpt
            ));
        }
        return AgentToolResult {
            tool_name: "detect_environment".to_string(),
            ok: true,
            output: out,
        };
    }

    let env = DetectorAgent::detect(&target);
    AgentToolResult {
        tool_name: "detect_environment".to_string(),
        ok: true,
        output: format!(
            "Environment for `{}`:\nLanguage: {}\nBuild System: {}\nCompiler Check: {:?}\nTest Runner: {:?}\nDetails: {}",
            target.display(),
            env.language.display_name(),
            env.build_system,
            env.compiler_check_cmd,
            env.test_cmd,
            env.details
        ),
    }
}
