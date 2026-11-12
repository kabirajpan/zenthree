use zenthra::Color;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExtensionCategory {
    LanguagePack,
    Tooling,
}

#[derive(Debug, Clone)]
pub struct LanguageExtension {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub extensions: &'static [&'static str],
    pub version: &'static str,
    pub author: &'static str,
    pub glyph: &'static str,
    pub icon_color: Color,
    pub features: &'static [&'static str],
    pub tools: &'static [&'static str],
}

pub struct ExtensionCatalog;

impl ExtensionCatalog {
    pub fn all() -> Vec<LanguageExtension> {
        vec![
            LanguageExtension {
                id: "go",
                name: "Go",
                description: "Full syntax highlighting, code navigation, and formatting for Go (.go)",
                extensions: &["go"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e627}", // Devicon Go
                icon_color: Color::rgb(0.0 / 255.0, 172.0 / 255.0, 215.0 / 255.0), // Go cyan #00acd7
                features: &["Syntax Highlighting", "Formatter (gofmt)", "Indent Calculator"],
                tools: &["gofmt", "go-tools", "tree-sitter"],
            },
            LanguageExtension {
                id: "typescript",
                name: "TypeScript / JavaScript",
                description: "Full TypeScript & JS support: React, Next.js, Node.js, Express, NestJS, and Prettier (.ts, .tsx, .js, .jsx)",
                extensions: &["ts", "tsx", "js", "jsx", "mts", "cts", "mjs", "cjs"],
                version: "1.3.1",
                author: "Zenthra Official",
                glyph: "\u{e7ba}", // React / TSX icon
                icon_color: Color::rgb(97.0 / 255.0, 218.0 / 255.0, 251.0 / 255.0), // React cyan #61dafb
                features: &["React & Next.js", "Node.js & Express", "NestJS Support", "Prettier Formatter", "Type Inference"],
                tools: &["prettier", "tsc", "typescript-language-server"],
            },
            LanguageExtension {
                id: "c",
                name: "C",
                description: "C language syntax, preprocessor directives, pointers, and clang-format (.c, .h)",
                extensions: &["c", "h"],
                version: "1.1.0",
                author: "Zenthra Official",
                glyph: "\u{e61e}", // Devicon C
                icon_color: Color::rgb(140.0 / 255.0, 160.0 / 255.0, 180.0 / 255.0),
                features: &["Syntax Highlighting", "ClangFormat", "Macro Parsing"],
                tools: &["clang-format", "tree-sitter-c"],
            },
            LanguageExtension {
                id: "cpp",
                name: "C++",
                description: "Modern C++ syntax, classes, templates, namespaces, and Clang tooling (.cpp, .cc, .cxx, .hpp, .hxx)",
                extensions: &["cpp", "cc", "cxx", "hpp", "hxx"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e61d}", // Devicon C++
                icon_color: Color::rgb(0.0 / 255.0, 89.0 / 255.0, 156.0 / 255.0), // C++ blue #00599c
                features: &["Tree-sitter AST", "Templates & Classes", "ClangFormat"],
                tools: &["clang-format", "tree-sitter-cpp"],
            },
            LanguageExtension {
                id: "html_css",
                name: "HTML / CSS",
                description: "Rich web styling syntax, tags, attributes, and colors (.html, .css, .scss)",
                extensions: &["html", "htm", "css", "scss", "less"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e736}", // Devicon HTML5
                icon_color: Color::rgb(228.0 / 255.0, 77.0 / 255.0, 38.0 / 255.0), // HTML orange #e44d26
                features: &["Syntax Highlighting", "Prettier Formatter", "Color Preview"],
                tools: &["prettier", "stylelint"],
            },
            LanguageExtension {
                id: "json_yaml",
                name: "JSON / YAML",
                description: "Structured configuration highlighting and formatting (.json, .yaml, .yml)",
                extensions: &["json", "jsonc", "yaml", "yml"],
                version: "1.1.0",
                author: "Zenthra Official",
                glyph: "\u{e60b}", // Devicon JSON
                icon_color: Color::rgb(245.0 / 255.0, 180.0 / 255.0, 0.0 / 255.0), // Gold yellow
                features: &["Syntax Highlighting", "Key-Value Validation", "Prettier"],
                tools: &["prettier", "yaml-tools"],
            },
            LanguageExtension {
                id: "markdown",
                name: "Markdown",
                description: "Headers, tables, code blocks, bold/italics, and preview styling (.md, .markdown)",
                extensions: &["md", "markdown"],
                version: "1.2.5",
                author: "Zenthra Official",
                glyph: "\u{f4ed}", // Markdown icon
                icon_color: Color::rgb(142.0 / 255.0, 192.0 / 255.0, 124.0 / 255.0), // Aqua green
                features: &["Syntax Highlighting", "Document Formatting", "Table Alignment"],
                tools: &["prettier", "markdownlint"],
            },
            LanguageExtension {
                id: "shell",
                name: "Shell",
                description: "Command syntax, variables, pipes, and script execution tools (.sh, .bash, .zsh)",
                extensions: &["sh", "bash", "zsh"],
                version: "1.0.8",
                author: "Zenthra Official",
                glyph: "\u{f120}", // Terminal prompt icon
                icon_color: Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0), // Terminal green
                features: &["Syntax Highlighting", "PTY Runner", "Shebang Detection"],
                tools: &["shfmt", "shellcheck"],
            },
            LanguageExtension {
                id: "lua",
                name: "Lua",
                description: "Lightweight scripting syntax, table definitions, and StyLua formatting (.lua)",
                extensions: &["lua"],
                version: "1.0.5",
                author: "Zenthra Official",
                glyph: "\u{e620}", // Devicon Lua
                icon_color: Color::rgb(0.0 / 255.0, 80.0 / 255.0, 180.0 / 255.0), // Lua blue
                features: &["Syntax Highlighting", "StyLua Formatter", "Neovim Config"],
                tools: &["stylua", "tree-sitter-lua"],
            },
            LanguageExtension {
                id: "rust",
                name: "Rust",
                description: "Tree-sitter native AST syntax highlighting, auto-indent, and rustfmt (.rs)",
                extensions: &["rs"],
                version: "2.1.0",
                author: "Zenthra Core",
                glyph: "\u{e7a8}", // Devicon Rust
                icon_color: Color::rgb(251.0 / 255.0, 146.0 / 255.0, 60.0 / 255.0), // Rust orange
                features: &["Tree-sitter AST", "rustfmt", "Semantic Context", "Auto-Indent"],
                tools: &["rustfmt", "cargo", "tree-sitter-rust"],
            },
            LanguageExtension {
                id: "python",
                name: "Python",
                description: "Tree-sitter syntax highlighting, indentation, and docstring formatting (.py)",
                extensions: &["py"],
                version: "2.0.2",
                author: "Zenthra Core",
                glyph: "\u{e73c}", // Devicon Python
                icon_color: Color::rgb(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0), // Python blue/yellow
                features: &["Tree-sitter AST", "Black Formatter", "Semantic Context"],
                tools: &["black", "tree-sitter-python"],
            },
            LanguageExtension {
                id: "javascript",
                name: "JavaScript",
                description: "Modern ES6+ syntax highlighting, JSX support, and Prettier integration (.js, .jsx)",
                extensions: &["js", "jsx", "mjs", "cjs"],
                version: "1.5.0",
                author: "Zenthra Core",
                glyph: "\u{e74e}", // Devicon JavaScript
                icon_color: Color::rgb(250.0 / 255.0, 204.0 / 255.0, 21.0 / 255.0), // JS Yellow
                features: &["Tree-sitter AST", "Prettier", "JSX Support"],
                tools: &["prettier", "tree-sitter-javascript"],
            },
            LanguageExtension {
                id: "toml",
                name: "TOML",
                description: "Cargo, PyProject, and configuration schema highlighting and formatting (.toml)",
                extensions: &["toml"],
                version: "1.1.2",
                author: "Zenthra Official",
                glyph: "\u{e6b2}", // TOML file icon
                icon_color: Color::rgb(156.0 / 255.0, 163.0 / 255.0, 175.0 / 255.0),
                features: &["Syntax Highlighting", "Taplo Formatter", "Schema Validation"],
                tools: &["taplo", "tree-sitter-toml"],
            },
            LanguageExtension {
                id: "dockerfile",
                name: "Dockerfile",
                description: "Container image syntax, build step instructions, and linting (Dockerfile, Containerfile)",
                extensions: &["dockerfile", "containerfile"],
                version: "1.0.4",
                author: "Zenthra Official",
                glyph: "\u{f308}", // Docker whale icon
                icon_color: Color::rgb(36.0 / 255.0, 150.0 / 255.0, 237.0 / 255.0),
                features: &["Instruction Highlighting", "Multi-stage Parsing", "Hadolint"],
                tools: &["hadolint", "dockerfile-langserver"],
            },
            LanguageExtension {
                id: "sql",
                name: "SQL",
                description: "Relational database querying syntax, keywords, joins, and dialect formatting (.sql)",
                extensions: &["sql"],
                version: "1.0.2",
                author: "Zenthra Official",
                glyph: "\u{e706}", // Database SQL icon
                icon_color: Color::rgb(227.0 / 255.0, 140.0 / 255.0, 45.0 / 255.0),
                features: &["Syntax Highlighting", "SQLFluff Formatter", "Dialect Keywords"],
                tools: &["sqlfluff", "sqls"],
            },
            LanguageExtension {
                id: "zig",
                name: "Zig",
                description: "Comptime syntax highlighting, memory safety checks, and zig fmt (.zig)",
                extensions: &["zig"],
                version: "1.1.0",
                author: "Zenthra Official",
                glyph: "\u{e6a9}", // Zig icon
                icon_color: Color::rgb(247.0 / 255.0, 164.0 / 255.0, 29.0 / 255.0),
                features: &["Syntax Highlighting", "zig fmt", "Comptime Parsing"],
                tools: &["zig", "zls"],
            },
            LanguageExtension {
                id: "java",
                name: "Java",
                description: "Class hierarchies, annotations, imports, and google-java-format (.java)",
                extensions: &["java"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e738}", // Java icon
                icon_color: Color::rgb(234.0 / 255.0, 45.0 / 255.0, 46.0 / 255.0),
                features: &["Syntax Highlighting", "Formatter", "Package Navigation"],
                tools: &["jdtls", "tree-sitter-java"],
            },
            LanguageExtension {
                id: "kotlin",
                name: "Kotlin",
                description: "JetBrains Kotlin syntax, coroutines, and ktlint formatting (.kt, .kts)",
                extensions: &["kt", "kts"],
                version: "1.0.8",
                author: "Zenthra Official",
                glyph: "\u{e634}", // Kotlin icon
                icon_color: Color::rgb(127.0 / 255.0, 82.0 / 255.0, 255.0 / 255.0),
                features: &["Syntax Highlighting", "ktlint Formatter", "Coroutines"],
                tools: &["ktlint", "kotlin-language-server"],
            },
            LanguageExtension {
                id: "ruby",
                name: "Ruby",
                description: "Ruby language highlighting, Gemfile parsing, and RuboCop integration (.rb)",
                extensions: &["rb", "rake", "gemspec"],
                version: "1.1.3",
                author: "Zenthra Official",
                glyph: "\u{e739}", // Ruby icon
                icon_color: Color::rgb(204.0 / 255.0, 52.0 / 255.0, 45.0 / 255.0),
                features: &["Syntax Highlighting", "RuboCop Formatter", "Gemfile Parsing"],
                tools: &["rubocop", "solargraph"],
            },
            LanguageExtension {
                id: "php",
                name: "PHP",
                description: "PHP 8+ syntax, types, attributes, and php-cs-fixer (.php)",
                extensions: &["php"],
                version: "1.0.9",
                author: "Zenthra Official",
                glyph: "\u{e73d}", // PHP icon
                icon_color: Color::rgb(119.0 / 255.0, 123.0 / 255.0, 180.0 / 255.0),
                features: &["Syntax Highlighting", "PHP-CS-Fixer", "Attributes Support"],
                tools: &["php-cs-fixer", "intelephense"],
            },
            LanguageExtension {
                id: "csharp",
                name: "C#",
                description: ".NET C# syntax highlighting, pattern matching, and dotnet format (.cs)",
                extensions: &["cs"],
                version: "1.1.5",
                author: "Zenthra Official",
                glyph: "\u{f031b}", // C# icon
                icon_color: Color::rgb(104.0 / 255.0, 33.0 / 255.0, 122.0 / 255.0),
                features: &["Syntax Highlighting", "dotnet format", "LINQ Parsing"],
                tools: &["dotnet", "csharp-ls"],
            },
            LanguageExtension {
                id: "swift",
                name: "Swift",
                description: "Apple Swift syntax highlighting, SwiftUI views, and swift-format (.swift)",
                extensions: &["swift"],
                version: "1.0.6",
                author: "Zenthra Official",
                glyph: "\u{e755}", // Swift icon
                icon_color: Color::rgb(240.0 / 255.0, 81.0 / 255.0, 56.0 / 255.0),
                features: &["Syntax Highlighting", "swift-format", "SwiftUI Highlighting"],
                tools: &["swift-format", "sourcekit-lsp"],
            },
            LanguageExtension {
                id: "dart",
                name: "Dart",
                description: "Flutter & Dart language syntax, widgets, and dart format (.dart)",
                extensions: &["dart"],
                version: "1.2.1",
                author: "Zenthra Official",
                glyph: "\u{e798}", // Dart icon
                icon_color: Color::rgb(0.0 / 255.0, 180.0 / 255.0, 216.0 / 255.0),
                features: &["Syntax Highlighting", "dart format", "Flutter Widgets"],
                tools: &["dart", "dart-language-server"],
            },
            LanguageExtension {
                id: "elixir",
                name: "Elixir",
                description: "Functional concurrent language syntax, mix tooling, and format (.ex, .exs)",
                extensions: &["ex", "exs"],
                version: "1.3.0",
                author: "Zenthra Official",
                glyph: "\u{e62d}", // Elixir drop icon
                icon_color: Color::rgb(142.0 / 255.0, 80.0 / 255.0, 180.0 / 255.0),
                features: &["Syntax Highlighting", "mix format", "Macro Expansion"],
                tools: &["mix", "elixir-ls"],
            },
            LanguageExtension {
                id: "haskell",
                name: "Haskell",
                description: "Purely functional language syntax, type annotations, and ormolu (.hs, .lhs)",
                extensions: &["hs", "lhs"],
                version: "1.1.0",
                author: "Zenthra Official",
                glyph: "\u{e61f}", // Haskell icon
                icon_color: Color::rgb(102.0 / 255.0, 51.0 / 255.0, 153.0 / 255.0),
                features: &["Syntax Highlighting", "ormolu", "Type Inference"],
                tools: &["cabal", "hls"],
            },
            LanguageExtension {
                id: "scala",
                name: "Scala",
                description: "Object-oriented and functional syntax, sbt build, and scalafmt (.scala, .sc)",
                extensions: &["scala", "sc"],
                version: "1.0.8",
                author: "Zenthra Official",
                glyph: "\u{e737}", // Scala icon
                icon_color: Color::rgb(220.0 / 255.0, 50.0 / 255.0, 50.0 / 255.0),
                features: &["Syntax Highlighting", "scalafmt", "Metals Support"],
                tools: &["metals", "sbt"],
            },
            LanguageExtension {
                id: "clojure",
                name: "Clojure",
                description: "Lisp dialect for functional programming with rich s-expressions (.clj, .cljs)",
                extensions: &["clj", "cljs", "edn"],
                version: "1.0.5",
                author: "Zenthra Official",
                glyph: "\u{e76a}", // Clojure icon
                icon_color: Color::rgb(90.0 / 255.0, 180.0 / 255.0, 70.0 / 255.0),
                features: &["Syntax Highlighting", "cljfmt", "S-Expression Matching"],
                tools: &["clojure-lsp", "cljfmt"],
            },
            LanguageExtension {
                id: "svelte",
                name: "Svelte",
                description: "Reactive component syntax, script/style tags, and svelte-check (.svelte)",
                extensions: &["svelte"],
                version: "1.2.0",
                author: "Zenthra Official",
                glyph: "\u{e697}", // Svelte icon
                icon_color: Color::rgb(255.0 / 255.0, 62.0 / 255.0, 0.0 / 255.0),
                features: &["Syntax Highlighting", "Prettier Plugin", "Reactivity Parsing"],
                tools: &["svelte-check", "svelte-language-server"],
            },
            LanguageExtension {
                id: "vue",
                name: "Vue",
                description: "Single-file component templates, script setup, and Vue tooling (.vue)",
                extensions: &["vue"],
                version: "1.2.2",
                author: "Zenthra Official",
                glyph: "\u{f0844}", // Vue icon
                icon_color: Color::rgb(65.0 / 255.0, 184.0 / 255.0, 131.0 / 255.0),
                features: &["Syntax Highlighting", "Template Validation", "Prettier"],
                tools: &["vue-language-server", "prettier"],
            },
            LanguageExtension {
                id: "protobuf",
                name: "Protocol Buffers",
                description: "Google Protobuf schema definitions, messages, enums, and RPCs (.proto)",
                extensions: &["proto"],
                version: "1.0.7",
                author: "Zenthra Official",
                glyph: "\u{e6b1}", // Protobuf icon
                icon_color: Color::rgb(66.0 / 255.0, 133.0 / 255.0, 244.0 / 255.0),
                features: &["Syntax Highlighting", "buf format", "Field Validation"],
                tools: &["buf", "protols"],
            },
            LanguageExtension {
                id: "graphql",
                name: "GraphQL",
                description: "Schema definition language, query syntax, mutations, and fragments (.graphql, .gql)",
                extensions: &["graphql", "gql"],
                version: "1.1.0",
                author: "Zenthra Official",
                glyph: "\u{e662}", // GraphQL icon
                icon_color: Color::rgb(225.0 / 255.0, 0.0 / 255.0, 152.0 / 255.0),
                features: &["Syntax Highlighting", "Prettier", "Schema Validation"],
                tools: &["prettier", "graphql-lsp"],
            },
            LanguageExtension {
                id: "prisma",
                name: "Prisma",
                description: "Prisma ORM schema modeling, relations, generators, and datasource (.prisma)",
                extensions: &["prisma"],
                version: "1.0.4",
                author: "Zenthra Official",
                glyph: "\u{e684}", // Prisma icon
                icon_color: Color::rgb(45.0 / 255.0, 55.0 / 255.0, 72.0 / 255.0),
                features: &["Syntax Highlighting", "prisma format", "Model Validation"],
                tools: &["prisma", "prisma-language-server"],
            },
            LanguageExtension {
                id: "astro",
                name: "Astro",
                description: "Component syntax, Astro islands, frontmatter scripts, and formatting (.astro)",
                extensions: &["astro"],
                version: "1.0.0",
                author: "Zenthra Official",
                glyph: "\u{e6b3}",
                icon_color: Color::rgb(255.0 / 255.0, 93.0 / 255.0, 1.0 / 255.0),
                features: &["Astro Islands", "Frontmatter Parsing", "Prettier Plugin"],
                tools: &["prettier", "astro-ls"],
            },
            LanguageExtension {
                id: "tailwind",
                name: "Tailwind CSS",
                description: "Utility-first CSS, class name autocompletion, directive validation, and hover color preview",
                extensions: &["css"],
                version: "1.0.2",
                author: "Zenthra Official",
                glyph: "\u{e6ba}",
                icon_color: Color::rgb(56.0 / 255.0, 189.0 / 255.0, 248.0 / 255.0),
                features: &["Class Autocomplete", "Color Preview", "CSS Directives"],
                tools: &["tailwindcss", "tailwindcss-language-server"],
            },
            LanguageExtension {
                id: "mdx",
                name: "MDX",
                description: "Markdown with embedded React JSX component syntax and frontmatter (.mdx)",
                extensions: &["mdx"],
                version: "1.0.0",
                author: "Zenthra Official",
                glyph: "\u{f4ed}",
                icon_color: Color::rgb(249.0 / 255.0, 172.0 / 255.0, 0.0 / 255.0),
                features: &["React in Markdown", "Component Autocomplete", "Frontmatter"],
                tools: &["prettier", "mdx-language-server"],
            },
            LanguageExtension {
                id: "templates",
                name: "EJS / Handlebars",
                description: "Server-side templates for Express & Node.js (.ejs, .hbs, .handlebars, .mustache)",
                extensions: &["ejs", "hbs", "handlebars", "mustache"],
                version: "1.0.0",
                author: "Zenthra Official",
                glyph: "\u{e60e}",
                icon_color: Color::rgb(234.0 / 255.0, 88.0 / 255.0, 12.0 / 255.0),
                features: &["Template Highlighting", "Express Views", "HTML Formatting"],
                tools: &["prettier"],
            },
        ]
    }

    /// Find an extension matching the given file path
    pub fn find_by_path(path: &Path) -> Option<LanguageExtension> {
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            if file_name.eq_ignore_ascii_case("dockerfile") || file_name.eq_ignore_ascii_case("containerfile") {
                return Self::find_by_id("dockerfile");
            }
        }
        let ext = path.extension()?.to_str()?.to_lowercase();
        Self::find_by_extension(&ext)
    }

    /// Find an extension matching a file extension string
    pub fn find_by_extension(ext: &str) -> Option<LanguageExtension> {
        let ext_clean = ext.trim_start_matches('.').to_lowercase();
        Self::all()
            .into_iter()
            .find(|item| item.extensions.contains(&ext_clean.as_str()))
    }

    /// Find extension by ID
    pub fn find_by_id(id: &str) -> Option<LanguageExtension> {
        Self::all().into_iter().find(|item| item.id == id)
    }
}

/// Dynamically downloads official Tree-Sitter queries from the Helix server.
/// Every language is kept strictly in its own separate directory on disk.
pub fn download_language_queries(ext_id: &str) -> Result<(), String> {
    let data_dir = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp"));
    let queries_root = data_dir.join("zenthree").join("queries");

    // Expand composite bundles into clean individual languages
    let languages_to_download: Vec<&str> = match ext_id {
        "html_css" => vec!["html", "css"],
        "json_yaml" => vec!["json", "yaml"],
        "typescript" => vec!["typescript", "javascript"],
        "cpp" => vec!["cpp", "c"],
        "c" => vec!["c"],
        "tsx" => vec!["tsx", "typescript", "javascript"],
        other => vec![other],
    };

    for lang in languages_to_download {
        let helix_folder = helix_query_folder(lang);
        let _ = download_single_helix_query(lang, helix_folder, &queries_root);
    }

    Ok(())
}

fn helix_query_folder(lang: &str) -> &'static str {
    match lang {
        "csharp" => "c-sharp",
        "shell" => "bash",
        "protobuf" => "proto",
        "c" => "c",
        "cpp" => "cpp",
        "rust" => "rust",
        "python" => "python",
        "java" => "java",
        "javascript" => "javascript",
        "typescript" => "typescript",
        "tsx" => "tsx",
        "go" => "go",
        "html" => "html",
        "css" => "css",
        "json" => "json",
        "yaml" => "yaml",
        "toml" => "toml",
        "dockerfile" => "dockerfile",
        "sql" => "sql",
        "zig" => "zig",
        "lua" => "lua",
        "ruby" => "ruby",
        "php" => "php",
        "kotlin" => "kotlin",
        "swift" => "swift",
        "elixir" => "elixir",
        "svelte" => "svelte",
        "vue" => "vue",
        "graphql" => "graphql",
        "prisma" => "prisma",
        "astro" => "astro",
        "markdown" => "markdown",
        _ => "rust",
    }
}

fn download_single_helix_query(lang: &str, helix_folder: &str, queries_root: &std::path::Path) -> Result<(), String> {
    let queries_dir = queries_root.join(lang);
    let _ = std::fs::create_dir_all(&queries_dir);

    // Complete suite of queries Helix handles:
    // highlights.scm (syntax highlighting)
    // indents.scm (smart AST auto-indentation)
    // tags.scm (document outline & symbol definitions)
    // textobjects.scm (AST structural selection)
    // injections.scm (multi-language embedding)
    const QUERY_FILES: &[&str] = &[
        "highlights.scm",
        "indents.scm",
        "tags.scm",
        "textobjects.scm",
        "injections.scm",
    ];

    let mut highlights_downloaded = false;

    for filename in QUERY_FILES {
        let target_file = queries_dir.join(filename);
        let url = format!(
            "https://raw.githubusercontent.com/helix-editor/helix/master/runtime/queries/{}/{}",
            helix_folder, filename
        );

        let output = std::process::Command::new("curl")
            .args(&["-f", "-m", "10", "-sSL", &url, "-o", target_file.to_str().unwrap_or("")])
            .output();

        let success = match output {
            Ok(out) if out.status.success() && target_file.exists() => true,
            _ => {
                #[cfg(windows)]
                {
                    std::process::Command::new("powershell")
                        .args(&["-Command", &format!("try {{ (New-Object Net.WebClient).DownloadFile('{}', '{}') }} catch {{ exit 1 }}", url, target_file.display())])
                        .output()
                        .map(|o| o.status.success() && target_file.exists())
                        .unwrap_or(false)
                }
                #[cfg(not(windows))]
                { false }
            }
        };

        if *filename == "highlights.scm" && success {
            highlights_downloaded = true;
        }
    }

    if highlights_downloaded {
        Ok(())
    } else {
        Err(format!("Failed to download Helix queries for {}", lang))
    }
}
