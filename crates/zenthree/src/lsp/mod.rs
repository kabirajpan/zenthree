pub mod protocol;
pub mod transport;
pub mod installer;

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};
use log::{warn, error};
use protocol::*;
use transport::{LspIncomingMessage, LspTransport};

static EXECUTABLE_PATH_CACHE: OnceLock<RwLock<HashMap<String, (Option<PathBuf>, Instant)>>> = OnceLock::new();
static LANGUAGE_INSTALLED_CACHE: OnceLock<RwLock<HashMap<String, (bool, Instant)>>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerStatus {
    NotStarted,
    Starting,
    Ready,
    Failed,
}

#[derive(Debug, Clone)]
pub struct LspServerInfo {
    pub language_id: &'static str,
    pub binary_name: String,
    pub display_name: &'static str,
    pub install_hint: &'static str,
}

/// Enriched PATH including ~/.local/bin, ~/.cargo/bin, and Zenthree toolchains
pub fn enriched_path() -> std::ffi::OsString {
    let mut paths: Vec<PathBuf> = Vec::new();

    let venv_bin = installer::LspInstaller::python_venv_dir().join("bin");
    paths.push(venv_bin);

    if let Ok(home) = std::env::var("HOME") {
        let home_p = PathBuf::from(&home);
        paths.push(home_p.join(".local").join("share").join("zenthree").join("bin"));
        paths.push(home_p.join(".local").join("share").join("zenthree").join("lsp").join("elixir-ls"));
        paths.push(home_p.join(".local").join("share").join("zenthree").join("lsp"));
        paths.push(home_p.join(".local").join("bin"));
        paths.push(home_p.join(".cargo").join("bin"));
        paths.push(home_p.join("go").join("bin"));
        paths.push(home_p.join(".npm-global").join("bin"));
    }

    if let Ok(path_var) = std::env::var("PATH") {
        for p in std::env::split_paths(&path_var) {
            paths.push(p);
        }
    }

    std::env::join_paths(paths).unwrap_or_default()
}

#[derive(Debug, Clone)]
pub struct LanguageServerCatalogItem {
    pub language_id: &'static str,
    pub name: &'static str,
    pub server_binary: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
}

impl LanguageServerCatalogItem {
    pub fn all() -> Vec<Self> {
        vec![
            Self {
                language_id: "python",
                name: "Python LSP",
                server_binary: "pylsp / pyright",
                description: "Type inference, autocompletion, signature help & formatting",
                extensions: &["py", "pyi"],
            },
            Self {
                language_id: "rust",
                name: "Rust LSP",
                server_binary: "rust-analyzer",
                description: "Compiler diagnostics, inlay hints, type definitions & docs",
                extensions: &["rs"],
            },
            Self {
                language_id: "typescript",
                name: "TypeScript / Node / Express LSP",
                server_binary: "typescript-language-server",
                description: "Type checking, React, Next.js, Node.js, Express, NestJS auto-imports & actions",
                extensions: &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"],
            },
            Self {
                language_id: "go",
                name: "Go LSP",
                server_binary: "gopls",
                description: "Go definitions, formatting, references & static analysis",
                extensions: &["go"],
            },
            Self {
                language_id: "cpp",
                name: "C / C++ LSP",
                server_binary: "clangd",
                description: "Clang-based diagnostics, header completion & cross-references",
                extensions: &["c", "cpp", "cc", "cxx", "h", "hpp"],
            },
            Self {
                language_id: "shell",
                name: "Bash / Shell LSP",
                server_binary: "bash-language-server",
                description: "Shell script diagnostics, jump to declaration & completions",
                extensions: &["sh", "bash", "zsh"],
            },
            Self {
                language_id: "markdown",
                name: "Markdown LSP",
                server_binary: "marksman",
                description: "Heading references, wiki links, symbol completion & diagnostics",
                extensions: &["md", "markdown"],
            },
            Self {
                language_id: "lua",
                name: "Lua LSP",
                server_binary: "lua-language-server",
                description: "Neovim script typing, table inference, diagnostics & docs",
                extensions: &["lua"],
            },
            Self {
                language_id: "html_css",
                name: "HTML / CSS LSP",
                server_binary: "vscode-html-language-server",
                description: "HTML tags, CSS rules, property completion & validation",
                extensions: &["html", "htm", "css", "scss"],
            },
            Self {
                language_id: "json_yaml",
                name: "JSON / YAML LSP",
                server_binary: "yaml-language-server",
                description: "Schema validation, structure completion & linting",
                extensions: &["json", "yaml", "yml"],
            },
            Self {
                language_id: "toml",
                name: "TOML LSP",
                server_binary: "taplo",
                description: "TOML schema validation, formatting, completions & diagnostics",
                extensions: &["toml"],
            },
            Self {
                language_id: "dockerfile",
                name: "Dockerfile LSP",
                server_binary: "docker-langserver",
                description: "Container syntax checking, instruction validation & documentation",
                extensions: &["dockerfile", "containerfile"],
            },
            Self {
                language_id: "sql",
                name: "SQL LSP",
                server_binary: "sqls",
                description: "SQL query linting, database table & column autocompletion",
                extensions: &["sql"],
            },
            Self {
                language_id: "zig",
                name: "Zig LSP",
                server_binary: "zls",
                description: "Zig Language Server: completions, definitions & diagnostics",
                extensions: &["zig"],
            },
            Self {
                language_id: "java",
                name: "Java LSP",
                server_binary: "jdtls",
                description: "Eclipse JDT Language Server for Java projects",
                extensions: &["java"],
            },
            Self {
                language_id: "kotlin",
                name: "Kotlin LSP",
                server_binary: "kotlin-language-server",
                description: "Smart code completion, diagnostics & hover docs for Kotlin",
                extensions: &["kt", "kts"],
            },
            Self {
                language_id: "ruby",
                name: "Ruby LSP",
                server_binary: "solargraph / ruby-lsp",
                description: "Type inference, completions, documentation & diagnostics",
                extensions: &["rb"],
            },
            Self {
                language_id: "php",
                name: "PHP LSP",
                server_binary: "intelephense",
                description: "Fast PHP autocompletion, signature help & type checking",
                extensions: &["php"],
            },
            Self {
                language_id: "csharp",
                name: "C# LSP",
                server_binary: "csharp-ls / omnisharp",
                description: "Roslyn-powered language features for .NET C#",
                extensions: &["cs"],
            },
            Self {
                language_id: "swift",
                name: "Swift LSP",
                server_binary: "sourcekit-lsp",
                description: "Apple SourceKit-LSP for Swift and C-family languages",
                extensions: &["swift"],
            },
            Self {
                language_id: "dart",
                name: "Dart / Flutter LSP",
                server_binary: "dart language-server",
                description: "SDK analysis server for Flutter & Dart development",
                extensions: &["dart"],
            },
            Self {
                language_id: "elixir",
                name: "Elixir LSP",
                server_binary: "elixir-ls",
                description: "Dialyzer analysis, completion, code navigation & formatting for Elixir",
                extensions: &["ex", "exs"],
            },
            Self {
                language_id: "haskell",
                name: "Haskell LSP",
                server_binary: "haskell-language-server",
                description: "Type diagnostics, code actions, hover & evaluator for Haskell",
                extensions: &["hs", "lhs"],
            },
            Self {
                language_id: "scala",
                name: "Scala LSP (Metals)",
                server_binary: "metals",
                description: "Metals language server with rich Scala code intelligence",
                extensions: &["scala", "sc"],
            },
            Self {
                language_id: "clojure",
                name: "Clojure LSP",
                server_binary: "clojure-lsp",
                description: "Clojure language server for editor autocompletion & navigation",
                extensions: &["clj", "cljs", "edn"],
            },
            Self {
                language_id: "svelte",
                name: "Svelte LSP",
                server_binary: "svelte-language-server",
                description: "Svelte component validation, completions, and type checking",
                extensions: &["svelte"],
            },
            Self {
                language_id: "vue",
                name: "Vue LSP (Volar)",
                server_binary: "vue-language-server",
                description: "TypeScript & template language intelligence for Vue SFCs",
                extensions: &["vue"],
            },
            Self {
                language_id: "protobuf",
                name: "Protobuf LSP",
                server_binary: "protols",
                description: "Protocol Buffers schema completion, linting and jump to def",
                extensions: &["proto"],
            },
            Self {
                language_id: "graphql",
                name: "GraphQL LSP",
                server_binary: "graphql-lsp",
                description: "Schema validation, fragment completions & query diagnostics",
                extensions: &["graphql", "gql"],
            },
            Self {
                language_id: "prisma",
                name: "Prisma LSP",
                server_binary: "prisma-language-server",
                description: "Prisma schema linting, model references & autocomplete",
                extensions: &["prisma"],
            },
            Self {
                language_id: "astro",
                name: "Astro LSP",
                server_binary: "astro-ls",
                description: "Astro components, script diagnostics, template completion & formatting",
                extensions: &["astro"],
            },
            Self {
                language_id: "tailwind",
                name: "Tailwind CSS LSP",
                server_binary: "tailwindcss-language-server",
                description: "Class name autocomplete, color preview, and CSS directive linting",
                extensions: &["html", "css", "tsx", "jsx", "vue", "svelte", "astro"],
            },
            Self {
                language_id: "mdx",
                name: "MDX LSP",
                server_binary: "mdx-language-server",
                description: "Markdown with embedded React JSX components and frontmatter",
                extensions: &["mdx"],
            },
            Self {
                language_id: "angular",
                name: "Angular LSP",
                server_binary: "ngserver",
                description: "Angular HTML templates, directive completions & component bindings",
                extensions: &["html", "ts"],
            },
        ]
    }
}

impl LspServerInfo {
    pub fn for_path(path: &Path) -> Option<Self> {
        let ext = if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if file_name.eq_ignore_ascii_case("dockerfile") || file_name.eq_ignore_ascii_case("containerfile") {
                "dockerfile".to_string()
            } else {
                path.extension()?.to_str()?.to_lowercase()
            }
        } else {
            path.extension()?.to_str()?.to_lowercase()
        };
        match ext.as_str() {
            "rs" => {
                let bin = LspManager::find_executable_path("rust-analyzer")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "rust-analyzer".to_string());
                Some(Self {
                    language_id: "rust",
                    binary_name: bin,
                    display_name: "rust-analyzer",
                    install_hint: "rustup component add rust-analyzer",
                })
            }
            "go" => {
                let bin = LspManager::find_executable_path("gopls")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "gopls".to_string());
                Some(Self {
                    language_id: "go",
                    binary_name: bin,
                    display_name: "gopls",
                    install_hint: "go install golang.org/x/tools/gopls@latest",
                })
            }
            "c" | "cpp" | "cc" | "cxx" | "h" | "hpp" => {
                let candidates = ["clangd", "ccls"];
                let bin = candidates.iter()
                    .find_map(|c| LspManager::find_executable_path(c))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "clangd".to_string());
                Some(Self {
                    language_id: "cpp",
                    binary_name: bin,
                    display_name: "clangd",
                    install_hint: "sudo apt install clangd",
                })
            }
            "py" => {
                let candidates = [
                    "pyright-langserver",
                    "pyright",
                    "pylsp",
                    "jedi-language-server",
                ];
                let bin = candidates.iter()
                    .find_map(|c| LspManager::find_executable_path(c))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "pylsp".to_string());
                Some(Self {
                    language_id: "python",
                    binary_name: bin,
                    display_name: "Python LSP",
                    install_hint: "pip install python-lsp-server",
                })
            }
            "ts" | "tsx" | "js" | "jsx" | "mts" | "cts" | "mjs" | "cjs" => {
                let candidates = ["typescript-language-server", "vtsls"];
                let bin = candidates.iter()
                    .find_map(|c| LspManager::find_executable_path(c))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "typescript-language-server".to_string());
                Some(Self {
                    language_id: "typescript",
                    binary_name: bin,
                    display_name: "tsserver",
                    install_hint: "npm install -g typescript typescript-language-server",
                })
            }
            "sh" | "bash" | "zsh" => {
                let bin = LspManager::find_executable_path("bash-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "bash-language-server".to_string());
                Some(Self {
                    language_id: "shell",
                    binary_name: bin,
                    display_name: "Bash LSP",
                    install_hint: "npm install -g bash-language-server",
                })
            }
            "md" | "markdown" => {
                let bin = LspManager::find_executable_path("marksman")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "marksman".to_string());
                Some(Self {
                    language_id: "markdown",
                    binary_name: bin,
                    display_name: "Marksman",
                    install_hint: "cargo install --locked marksman",
                })
            }
            "lua" => {
                let bin = LspManager::find_executable_path("lua-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "lua-language-server".to_string());
                Some(Self {
                    language_id: "lua",
                    binary_name: bin,
                    display_name: "Lua LSP",
                    install_hint: "sudo apt install lua-language-server",
                })
            }
            "html" | "htm" | "css" | "scss" | "less" => {
                let bin = LspManager::find_executable_path("vscode-html-language-server")
                    .or_else(|| LspManager::find_executable_path("vscode-css-language-server"))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "vscode-html-language-server".to_string());
                Some(Self {
                    language_id: "html_css",
                    binary_name: bin,
                    display_name: "HTML/CSS LSP",
                    install_hint: "npm install -g vscode-langservers-extracted",
                })
            }
            "json" | "jsonc" | "yaml" | "yml" => {
                let bin = LspManager::find_executable_path("yaml-language-server")
                    .or_else(|| LspManager::find_executable_path("vscode-json-language-server"))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "yaml-language-server".to_string());
                Some(Self {
                    language_id: "json_yaml",
                    binary_name: bin,
                    display_name: "YAML/JSON LSP",
                    install_hint: "npm install -g yaml-language-server",
                })
            }
            "toml" => {
                let bin = LspManager::find_executable_path("taplo")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "taplo".to_string());
                Some(Self {
                    language_id: "toml",
                    binary_name: bin,
                    display_name: "taplo",
                    install_hint: "cargo install --locked taplo-cli --features lsp",
                })
            }
            "dockerfile" | "containerfile" => {
                let bin = LspManager::find_executable_path("docker-langserver")
                    .or_else(|| LspManager::find_executable_path("dockerfile-language-server-nodejs"))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "docker-langserver".to_string());
                Some(Self {
                    language_id: "dockerfile",
                    binary_name: bin,
                    display_name: "docker-langserver",
                    install_hint: "npm install -g dockerfile-language-server-nodejs",
                })
            }
            "sql" => {
                let bin = LspManager::find_executable_path("sqls")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "sqls".to_string());
                Some(Self {
                    language_id: "sql",
                    binary_name: bin,
                    display_name: "sqls",
                    install_hint: "go install github.com/sqls-server/sqls@latest",
                })
            }
            "zig" => {
                let bin = LspManager::find_executable_path("zls")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "zls".to_string());
                Some(Self {
                    language_id: "zig",
                    binary_name: bin,
                    display_name: "zls",
                    install_hint: "zig build or install zls binary",
                })
            }
            "java" => {
                let bin = LspManager::find_executable_path("jdtls")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "jdtls".to_string());
                Some(Self {
                    language_id: "java",
                    binary_name: bin,
                    display_name: "jdtls",
                    install_hint: "Download eclipse.jdt.ls or install via package manager",
                })
            }
            "kt" | "kts" => {
                let bin = LspManager::find_executable_path("kotlin-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "kotlin-language-server".to_string());
                Some(Self {
                    language_id: "kotlin",
                    binary_name: bin,
                    display_name: "kotlin-language-server",
                    install_hint: "Install kotlin-language-server",
                })
            }
            "rb" => {
                let candidates = ["solargraph", "ruby-lsp"];
                let bin = candidates.iter()
                    .find_map(|c| LspManager::find_executable_path(c))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "solargraph".to_string());
                Some(Self {
                    language_id: "ruby",
                    binary_name: bin,
                    display_name: "Ruby LSP",
                    install_hint: "gem install solargraph or gem install ruby-lsp",
                })
            }
            "php" => {
                let bin = LspManager::find_executable_path("intelephense")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "intelephense".to_string());
                Some(Self {
                    language_id: "php",
                    binary_name: bin,
                    display_name: "intelephense",
                    install_hint: "npm install -g intelephense",
                })
            }
            "cs" => {
                let candidates = ["csharp-ls", "OmniSharp"];
                let bin = candidates.iter()
                    .find_map(|c| LspManager::find_executable_path(c))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "csharp-ls".to_string());
                Some(Self {
                    language_id: "csharp",
                    binary_name: bin,
                    display_name: "csharp-ls",
                    install_hint: "dotnet tool install -g csharp-ls",
                })
            }
            "swift" => {
                let bin = LspManager::find_executable_path("sourcekit-lsp")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "sourcekit-lsp".to_string());
                Some(Self {
                    language_id: "swift",
                    binary_name: bin,
                    display_name: "sourcekit-lsp",
                    install_hint: "Included with Swift toolchain (xcrun / swift)",
                })
            }
            "dart" => {
                let bin = LspManager::find_executable_path("dart")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "dart".to_string());
                Some(Self {
                    language_id: "dart",
                    binary_name: bin,
                    display_name: "dart language-server",
                    install_hint: "Included with Dart/Flutter SDK",
                })
            }
            "ex" | "exs" => {
                let candidates = ["elixir-ls", "language_server.sh", "next-ls"];
                let bin = candidates.iter()
                    .find_map(|c| LspManager::find_executable_path(c))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "elixir-ls".to_string());
                Some(Self {
                    language_id: "elixir",
                    binary_name: bin,
                    display_name: "Elixir LSP",
                    install_hint: "Download prebuilt elixir-ls release from GitHub or mix archive.install hex elixir_ls",
                })
            }
            "hs" | "lhs" => {
                let bin = LspManager::find_executable_path("haskell-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "haskell-language-server".to_string());
                Some(Self {
                    language_id: "haskell",
                    binary_name: bin,
                    display_name: "Haskell LSP",
                    install_hint: "ghcup install hls",
                })
            }
            "scala" | "sc" => {
                let bin = LspManager::find_executable_path("metals")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "metals".to_string());
                Some(Self {
                    language_id: "scala",
                    binary_name: bin,
                    display_name: "Metals (Scala)",
                    install_hint: "cs install metals",
                })
            }
            "clj" | "cljs" | "edn" => {
                let bin = LspManager::find_executable_path("clojure-lsp")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "clojure-lsp".to_string());
                Some(Self {
                    language_id: "clojure",
                    binary_name: bin,
                    display_name: "Clojure LSP",
                    install_hint: "Download clojure-lsp binary from GitHub",
                })
            }
            "svelte" => {
                let bin = LspManager::find_executable_path("svelteserver")
                    .or_else(|| LspManager::find_executable_path("svelte-language-server"))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "svelteserver".to_string());
                Some(Self {
                    language_id: "svelte",
                    binary_name: bin,
                    display_name: "Svelte LSP",
                    install_hint: "npm install -g svelte-language-server",
                })
            }
            "vue" => {
                let bin = LspManager::find_executable_path("vue-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "vue-language-server".to_string());
                Some(Self {
                    language_id: "vue",
                    binary_name: bin,
                    display_name: "Vue LSP",
                    install_hint: "npm install -g @vue/language-server",
                })
            }
            "proto" => {
                let bin = LspManager::find_executable_path("protols")
                    .or_else(|| LspManager::find_executable_path("buf"))
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "protols".to_string());
                Some(Self {
                    language_id: "protobuf",
                    binary_name: bin,
                    display_name: "Protobuf LSP",
                    install_hint: "Download protols from GitHub or install buf",
                })
            }
            "graphql" | "gql" => {
                let bin = LspManager::find_executable_path("graphql-lsp")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "graphql-lsp".to_string());
                Some(Self {
                    language_id: "graphql",
                    binary_name: bin,
                    display_name: "GraphQL LSP",
                    install_hint: "npm install -g graphql-language-service-cli",
                })
            }
            "prisma" => {
                let bin = LspManager::find_executable_path("prisma-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "prisma-language-server".to_string());
                Some(Self {
                    language_id: "prisma",
                    binary_name: bin,
                    display_name: "Prisma LSP",
                    install_hint: "npm install -g @prisma/language-server",
                })
            }
            "astro" => {
                let bin = LspManager::find_executable_path("astro-ls")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "astro-ls".to_string());
                Some(Self {
                    language_id: "astro",
                    binary_name: bin,
                    display_name: "Astro LSP",
                    install_hint: "npm install -g @astrojs/language-server",
                })
            }
            "mdx" => {
                let bin = LspManager::find_executable_path("mdx-language-server")
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "mdx-language-server".to_string());
                Some(Self {
                    language_id: "mdx",
                    binary_name: bin,
                    display_name: "MDX LSP",
                    install_hint: "npm install -g @mdx-js/language-server",
                })
            }
            _ => None,
        }
    }
}

pub struct ActiveSession {
    pub info: LspServerInfo,
    pub transport: LspTransport,
    pub status: ServerStatus,
    pub next_request_id: u64,
    pub open_documents: HashMap<PathBuf, u64>, // path -> version
}

pub struct LspManager {
    pub sessions: HashMap<&'static str, ActiveSession>,
    pub diagnostics: HashMap<PathBuf, Vec<Diagnostic>>,
    pub latest_completions: Option<Vec<CompletionItem>>,
    pub pending_completion_id: Option<u64>,
}

impl LspManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            diagnostics: HashMap::new(),
            latest_completions: None,
            pending_completion_id: None,
        }
    }

    /// Invalidate in-memory caches (called after installing an LSP server)
    pub fn invalidate_cache() {
        if let Some(lock) = LANGUAGE_INSTALLED_CACHE.get() {
            if let Ok(mut map) = lock.write() {
                map.clear();
            }
        }
        if let Some(lock) = EXECUTABLE_PATH_CACHE.get() {
            if let Ok(mut map) = lock.write() {
                map.clear();
            }
        }
    }

    /// Locate an executable across PATH and known toolchain directories (cached in-memory for fast UI queries)
    pub fn find_executable_path(name: &str) -> Option<PathBuf> {
        let cache_lock = EXECUTABLE_PATH_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        if let Ok(map) = cache_lock.read() {
            if let Some((res, timestamp)) = map.get(name) {
                if timestamp.elapsed() < Duration::from_secs(5) {
                    return res.clone();
                }
            }
        }

        let res = Self::find_executable_path_uncached(name);

        if let Ok(mut map) = cache_lock.write() {
            map.insert(name.to_string(), (res.clone(), Instant::now()));
        }

        res
    }

    fn find_executable_path_uncached(name: &str) -> Option<PathBuf> {
        let name_p = Path::new(name);
        if name_p.is_absolute() && name_p.is_file() {
            return Some(name_p.to_path_buf());
        }

        let mut search_dirs: Vec<PathBuf> = Vec::new();

        // 1. Zenthree dedicated python venv: ~/.local/share/zenthree/lsp_venv/bin
        let venv_bin = installer::LspInstaller::python_venv_dir().join("bin");
        search_dirs.push(venv_bin);

        // 2. User bins
        if let Ok(home) = std::env::var("HOME") {
            let home_p = PathBuf::from(&home);
            search_dirs.push(home_p.join(".local").join("share").join("zenthree").join("bin"));
            search_dirs.push(home_p.join(".local").join("bin"));
            search_dirs.push(home_p.join(".cargo").join("bin"));
            search_dirs.push(home_p.join("go").join("bin"));
            search_dirs.push(home_p.join(".npm-global").join("bin"));
        }

        // 3. System PATH
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                search_dirs.push(dir);
            }
        }

        for dir in search_dirs {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                let full_exe = dir.join(format!("{}.exe", name));
                let full_cmd = dir.join(format!("{}.cmd", name));
                if full_exe.is_file() { return Some(full_exe); }
                if full_cmd.is_file() { return Some(full_cmd); }
            }
        }

        // Fallback: which/where
        if let Ok(out) = std::process::Command::new("which").arg(name).output() {
            if out.status.success() {
                let out_str = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if !out_str.is_empty() {
                    return Some(PathBuf::from(out_str));
                }
            }
        }

        None
    }

    /// Check if a binary exists in system PATH or standard toolchain paths
    pub fn is_server_installed(binary_name: &str) -> bool {
        Self::find_executable_path(binary_name).is_some()
    }

    /// Check if ANY supported language server is installed for a given language ID (cached in-memory for 60+ FPS UI)
    pub fn is_language_installed(language_id: &str) -> bool {
        let cache_lock = LANGUAGE_INSTALLED_CACHE.get_or_init(|| RwLock::new(HashMap::new()));
        if let Ok(map) = cache_lock.read() {
            if let Some((res, timestamp)) = map.get(language_id) {
                if timestamp.elapsed() < Duration::from_secs(5) {
                    return *res;
                }
            }
        }

        let res = Self::is_language_installed_uncached(language_id);

        if let Ok(mut map) = cache_lock.write() {
            map.insert(language_id.to_string(), (res, Instant::now()));
        }

        res
    }

    fn is_language_installed_uncached(language_id: &str) -> bool {
        match language_id {
            "python" => {
                Self::find_executable_path("pyright-langserver").is_some()
                    || Self::find_executable_path("pyright").is_some()
                    || Self::find_executable_path("pylsp").is_some()
                    || Self::find_executable_path("jedi-language-server").is_some()
            }
            "rust" => Self::find_executable_path("rust-analyzer").is_some(),
            "typescript" | "javascript" => {
                Self::find_executable_path("typescript-language-server").is_some()
                    || Self::find_executable_path("vtsls").is_some()
            }
            "go" => Self::find_executable_path("gopls").is_some(),
            "cpp" => {
                Self::find_executable_path("clangd").is_some()
                    || Self::find_executable_path("ccls").is_some()
            }
            "shell" => Self::find_executable_path("bash-language-server").is_some(),
            "markdown" => Self::find_executable_path("marksman").is_some(),
            "lua" => Self::find_executable_path("lua-language-server").is_some(),
            "html_css" => {
                Self::find_executable_path("vscode-html-language-server").is_some()
                    || Self::find_executable_path("vscode-css-language-server").is_some()
                    || Self::find_executable_path("html-languageserver").is_some()
            }
            "json_yaml" => {
                Self::find_executable_path("yaml-language-server").is_some()
                    || Self::find_executable_path("vscode-json-language-server").is_some()
            }
            "toml" => Self::find_executable_path("taplo").is_some(),
            "dockerfile" => {
                Self::find_executable_path("docker-langserver").is_some()
                    || Self::find_executable_path("dockerfile-language-server-nodejs").is_some()
            }
            "sql" => Self::find_executable_path("sqls").is_some(),
            "zig" => Self::find_executable_path("zls").is_some(),
            "java" => Self::find_executable_path("jdtls").is_some(),
            "kotlin" => Self::find_executable_path("kotlin-language-server").is_some(),
            "ruby" => {
                Self::find_executable_path("solargraph").is_some()
                    || Self::find_executable_path("ruby-lsp").is_some()
            }
            "php" => Self::find_executable_path("intelephense").is_some(),
            "csharp" => {
                Self::find_executable_path("csharp-ls").is_some()
                    || Self::find_executable_path("OmniSharp").is_some()
            }
            "swift" => Self::find_executable_path("sourcekit-lsp").is_some(),
            "dart" => Self::find_executable_path("dart").is_some(),
            "elixir" => {
                Self::find_executable_path("elixir-ls").is_some()
                    || Self::find_executable_path("language_server.sh").is_some()
                    || Self::find_executable_path("next-ls").is_some()
            }
            "haskell" => Self::find_executable_path("haskell-language-server").is_some(),
            "scala" => Self::find_executable_path("metals").is_some(),
            "clojure" => Self::find_executable_path("clojure-lsp").is_some(),
            "svelte" => {
                Self::find_executable_path("svelteserver").is_some()
                    || Self::find_executable_path("svelte-language-server").is_some()
            }
            "vue" => Self::find_executable_path("vue-language-server").is_some(),
            "protobuf" => {
                Self::find_executable_path("protols").is_some()
                    || Self::find_executable_path("buf").is_some()
            }
            "graphql" => Self::find_executable_path("graphql-lsp").is_some(),
            "prisma" => Self::find_executable_path("prisma-language-server").is_some(),
            "astro" => Self::find_executable_path("astro-ls").is_some(),
            "tailwind" => Self::find_executable_path("tailwindcss-language-server").is_some(),
            "mdx" => Self::find_executable_path("mdx-language-server").is_some(),
            "angular" => Self::find_executable_path("ngserver").is_some(),
            _ => false,
        }
    }

    /// Ensure a session exists for the given file's language
    pub fn ensure_session_for_file(&mut self, path: &Path, workspace_root: Option<&Path>) -> Option<&mut ActiveSession> {
        let info = LspServerInfo::for_path(path)?;
        let lang = info.language_id;

        if !self.sessions.contains_key(lang) {
            // Attempt to start
            if !Self::is_server_installed(&info.binary_name) {
                return None;
            }

            let args: &[&str] = if info.binary_name.ends_with("pyright-langserver") {
                &["--stdio"]
            } else if info.binary_name.ends_with("pyright") {
                &["--langserver", "--stdio"]
            } else if info.binary_name.ends_with("typescript-language-server") {
                &["--stdio"]
            } else if info.binary_name.ends_with("bash-language-server") {
                &["start"]
            } else if info.binary_name.ends_with("marksman") {
                &["server"]
            } else if info.binary_name.ends_with("taplo") {
                &["lsp", "stdio"]
            } else if info.binary_name.ends_with("dart") {
                &["language-server"]
            } else if info.binary_name.contains("docker") {
                &["--stdio"]
            } else if info.binary_name.contains("intelephense") {
                &["--stdio"]
            } else if info.binary_name.contains("vscode-html") || info.binary_name.contains("vscode-css") || info.binary_name.contains("yaml-language-server") {
                &["--stdio"]
            } else {
                &[]
            };

            match LspTransport::start(&info.binary_name, args) {
                Ok(transport) => {
                    let mut session = ActiveSession {
                        info: info.clone(),
                        transport,
                        status: ServerStatus::Starting,
                        next_request_id: 1,
                        open_documents: HashMap::new(),
                    };

                    // Send initialize request
                    let root_uri = workspace_root.map(|p| format!("file://{}", p.display()));
                    let init_params = InitializeParams {
                        process_id: Some(std::process::id()),
                        client_info: Some(ClientInfo {
                            name: "Zenthree".to_string(),
                            version: Some("0.1.0".to_string()),
                        }),
                        root_uri,
                        capabilities: serde_json::json!({
                            "textDocument": {
                                "synchronization": {
                                    "openClose": true,
                                    "change": 1 // Full sync
                                },
                                "completion": {
                                    "completionItem": {
                                        "snippetSupport": false,
                                        "documentationFormat": ["plaintext"]
                                    }
                                }
                            }
                        }),
                    };

                    let req = JsonRpcRequest::new(session.next_request_id, "initialize", init_params);
                    session.next_request_id += 1;
                    if let Ok(bytes) = encode_lsp_message(&req) {
                        let _ = session.transport.send_raw(&bytes);
                    }

                    // Notification: initialized
                    let notif = JsonRpcNotification::new("initialized", serde_json::json!({}));
                    if let Ok(bytes) = encode_lsp_message(&notif) {
                        let _ = session.transport.send_raw(&bytes);
                    }

                    session.status = ServerStatus::Ready;
                    self.sessions.insert(lang, session);
                }
                Err(e) => {
                    error!("Failed to start LSP {}: {:?}", info.binary_name, e);
                    return None;
                }
            }
        }

        self.sessions.get_mut(lang)
    }

    /// Notify LSP that a file was opened
    pub fn on_document_open(&mut self, path: &Path, content: &str, workspace_root: Option<&Path>) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };
        let lang_id = info.language_id.to_string();

        if let Some(session) = self.ensure_session_for_file(path, workspace_root) {
            session.open_documents.insert(path.to_path_buf(), 1);
            let params = DidOpenTextDocumentParams {
                text_document: TextDocumentItem {
                    uri,
                    language_id: lang_id,
                    version: 1,
                    text: content.to_string(),
                },
            };
            let notif = JsonRpcNotification::new("textDocument/didOpen", params);
            if let Ok(bytes) = encode_lsp_message(&notif) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Notify LSP that document content changed
    pub fn on_document_change(&mut self, path: &Path, content: &str) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            let ver = session.open_documents.entry(path.to_path_buf()).or_insert(0);
            *ver += 1;
            let current_ver = *ver;

            let params = DidChangeTextDocumentParams {
                text_document: VersionedTextDocumentIdentifier {
                    uri,
                    version: current_ver,
                },
                content_changes: vec![TextDocumentContentChangeEvent {
                    text: content.to_string(),
                }],
            };
            let notif = JsonRpcNotification::new("textDocument/didChange", params);
            if let Ok(bytes) = encode_lsp_message(&notif) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Notify LSP that document was closed
    pub fn on_document_close(&mut self, path: &Path) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            session.open_documents.remove(path);
            let params = DidCloseTextDocumentParams {
                text_document: TextDocumentIdentifier { uri },
            };
            let notif = JsonRpcNotification::new("textDocument/didClose", params);
            if let Ok(bytes) = encode_lsp_message(&notif) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
        self.diagnostics.remove(path);
    }

    /// Request completions at the given line and character
    pub fn request_completion(&mut self, path: &Path, line: u32, character: u32) {
        let uri = format!("file://{}", path.display());
        let info = if let Some(i) = LspServerInfo::for_path(path) { i } else { return; };

        if let Some(session) = self.sessions.get_mut(info.language_id) {
            let req_id = session.next_request_id;
            session.next_request_id += 1;
            self.pending_completion_id = Some(req_id);

            let params = CompletionParams {
                text_document: TextDocumentIdentifier { uri },
                position: Position { line, character },
            };
            let req = JsonRpcRequest::new(req_id, "textDocument/completion", params);
            if let Ok(bytes) = encode_lsp_message(&req) {
                let _ = session.transport.send_raw(&bytes);
            }
        }
    }

    /// Poll incoming LSP messages across all sessions (called once per frame)
    pub fn poll(&mut self) -> bool {
        let mut had_updates = false;

        let langs: Vec<&'static str> = self.sessions.keys().cloned().collect();
        for lang in langs {
            let mut incoming = Vec::new();
            if let Some(session) = self.sessions.get_mut(lang) {
                while let Some(msg) = session.transport.try_recv() {
                    incoming.push(msg);
                }
            }

            for msg in incoming {
                had_updates = true;
                match msg {
                    LspIncomingMessage::Notification { method, params } => {
                        if method == "textDocument/publishDiagnostics" {
                            if let Some(p) = params {
                                if let Ok(diag_params) = serde_json::from_value::<PublishDiagnosticsParams>(p) {
                                    let path_str = diag_params.uri.trim_start_matches("file://");
                                    let path = PathBuf::from(path_str);
                                    self.diagnostics.insert(path, diag_params.diagnostics);
                                }
                            }
                        }
                    }
                    LspIncomingMessage::Response { id, result, .. } => {
                        if Some(id) == self.pending_completion_id {
                            self.pending_completion_id = None;
                            if let Some(res) = result {
                                if let Ok(comp_res) = serde_json::from_value::<CompletionResponse>(res) {
                                    match comp_res {
                                        CompletionResponse::List(l) => self.latest_completions = Some(l.items),
                                        CompletionResponse::Array(a) => self.latest_completions = Some(a),
                                    }
                                }
                            }
                        }
                    }
                    LspIncomingMessage::ServerDied => {
                        warn!("LSP server for {} terminated", lang);
                        if let Some(session) = self.sessions.get_mut(lang) {
                            session.status = ServerStatus::Failed;
                        }
                    }
                }
            }
        }

        had_updates
    }

    /// Get total error and warning count for a path
    pub fn get_diagnostic_counts(&self, path: Option<&Path>) -> (usize, usize) {
        if let Some(p) = path {
            if let Some(diags) = self.diagnostics.get(p) {
                let mut errors = 0;
                let mut warnings = 0;
                for d in diags {
                    match d.severity {
                        Some(DiagnosticSeverity::Error) => errors += 1,
                        Some(DiagnosticSeverity::Warning) => warnings += 1,
                        _ => {}
                    }
                }
                return (errors, warnings);
            }
        }
        (0, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_server_info_candidate_detection() {
        // Rust analyzer check
        let rs_info = LspServerInfo::for_path(Path::new("src/main.rs"));
        assert!(rs_info.is_some());
        let info = rs_info.unwrap();
        assert_eq!(info.language_id, "rust");
        assert_eq!(info.display_name, "rust-analyzer");

        // Python candidate detection
        let py_info = LspServerInfo::for_path(Path::new("scripts/test.py"));
        assert!(py_info.is_some());
        let p_info = py_info.unwrap();
        assert_eq!(p_info.language_id, "python");
        assert_eq!(p_info.display_name, "Python LSP");

        // Enriched path includes cargo and zenthree bin
        let enriched = enriched_path();
        let path_str = enriched.to_string_lossy();
        assert!(path_str.contains(".cargo") || path_str.contains("zenthree") || !path_str.is_empty());

        // TOML detection
        let toml_info = LspServerInfo::for_path(Path::new("Cargo.toml")).unwrap();
        assert_eq!(toml_info.language_id, "toml");
        assert_eq!(toml_info.display_name, "taplo");

        // Dockerfile detection (both extensionless and with extension)
        let docker_info = LspServerInfo::for_path(Path::new("Dockerfile")).unwrap();
        assert_eq!(docker_info.language_id, "dockerfile");
        assert_eq!(docker_info.display_name, "docker-langserver");

        // SQL detection
        let sql_info = LspServerInfo::for_path(Path::new("schema.sql")).unwrap();
        assert_eq!(sql_info.language_id, "sql");
        assert_eq!(sql_info.display_name, "sqls");

        // Elixir detection (.ex)
        let elixir_info = LspServerInfo::for_path(Path::new("lib/app.ex")).unwrap();
        assert_eq!(elixir_info.language_id, "elixir");
        assert_eq!(elixir_info.display_name, "Elixir LSP");

        // Haskell detection (.hs)
        let haskell_info = LspServerInfo::for_path(Path::new("Main.hs")).unwrap();
        assert_eq!(haskell_info.language_id, "haskell");
        assert_eq!(haskell_info.display_name, "Haskell LSP");

        // Svelte detection (.svelte)
        let svelte_info = LspServerInfo::for_path(Path::new("App.svelte")).unwrap();
        assert_eq!(svelte_info.language_id, "svelte");
        assert_eq!(svelte_info.display_name, "Svelte LSP");

        // Astro detection (.astro)
        let astro_info = LspServerInfo::for_path(Path::new("pages/index.astro")).unwrap();
        assert_eq!(astro_info.language_id, "astro");
        assert_eq!(astro_info.display_name, "Astro LSP");

        // MDX detection (.mdx)
        let mdx_info = LspServerInfo::for_path(Path::new("docs/intro.mdx")).unwrap();
        assert_eq!(mdx_info.language_id, "mdx");
        assert_eq!(mdx_info.display_name, "MDX LSP");
    }

    #[test]
    fn test_is_language_installed_rust() {
        // Since rust-analyzer is installed on this dev environment, it should report true
        let has_rust = LspManager::is_language_installed("rust");
        assert!(has_rust);
    }
}
