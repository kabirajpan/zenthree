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

        // 7. Fallback: Check dominant file extensions if no explicit build manifest is found
        let ctx = Self::inspect_workspace_context(workspace_root);
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
                _ => {}
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
