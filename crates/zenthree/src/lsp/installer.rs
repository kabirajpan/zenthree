use std::path::PathBuf;
use std::process::Command;
use std::thread;
use log::{info, error};
use zenthra_state::ArcSignal;

#[derive(Debug, Clone)]
pub enum LspInstallEvent {
    Started {
        language_id: String,
        message: String,
    },
    Progress {
        language_id: String,
        message: String,
    },
    Success {
        language_id: String,
        binary_name: String,
        message: String,
    },
    Failed {
        language_id: String,
        error: String,
    },
}

pub struct LspInstaller;

impl LspInstaller {
    /// Get the dedicated Zenthree tools directory: ~/.local/share/zenthree
    pub fn zenthree_data_dir() -> PathBuf {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local").join("share").join("zenthree")
        } else {
            PathBuf::from("/tmp").join("zenthree")
        }
    }

    /// Path to python virtualenv: ~/.local/share/zenthree/lsp_venv
    pub fn python_venv_dir() -> PathBuf {
        Self::zenthree_data_dir().join("lsp_venv")
    }

    /// Launch background installation for a language server
    pub fn start_install(language_id: &str, signal: ArcSignal<Vec<LspInstallEvent>>) {
        let lang = language_id.to_string();
        info!("Starting background LSP install for language: {}", lang);

        signal.with_mut(|q| {
            q.push(LspInstallEvent::Started {
                language_id: lang.clone(),
                message: format!("Installing {} Language Server...", lang),
            });
        });

        thread::spawn(move || {
            let res = match lang.as_str() {
                "python" => Self::install_python(&lang, &signal),
                "rust" => Self::install_rust(&lang, &signal),
                "typescript" | "javascript" => Self::install_typescript(&lang, &signal),
                "go" => Self::install_go(&lang, &signal),
                "cpp" => Self::install_cpp(&lang, &signal),
                "shell" => Self::install_shell(&lang, &signal),
                "markdown" => Self::install_markdown(&lang, &signal),
                "lua" => Self::install_lua(&lang, &signal),
                "html_css" => Self::install_html_css(&lang, &signal),
                "json_yaml" => Self::install_json_yaml(&lang, &signal),
                "toml" => Self::install_toml(&lang, &signal),
                "dockerfile" => Self::install_dockerfile(&lang, &signal),
                "sql" => Self::install_sql(&lang, &signal),
                "elixir" => Self::install_elixir(&lang, &signal),
                "svelte" => Self::install_npm_package(&lang, &signal, "svelte-language-server", "svelteserver"),
                "vue" => Self::install_npm_package(&lang, &signal, "@vue/language-server", "vue-language-server"),
                "graphql" => Self::install_npm_package(&lang, &signal, "graphql-language-service-cli", "graphql-lsp"),
                "prisma" => Self::install_npm_package(&lang, &signal, "@prisma/language-server", "prisma-language-server"),
                "astro" => Self::install_npm_package(&lang, &signal, "@astrojs/language-server", "astro-ls"),
                "tailwind" => Self::install_npm_package(&lang, &signal, "@tailwindcss/language-server", "tailwindcss-language-server"),
                "mdx" => Self::install_npm_package(&lang, &signal, "@mdx-js/language-server", "mdx-language-server"),
                "angular" => Self::install_npm_package(&lang, &signal, "@angular/language-server", "ngserver"),
                _ => Err(format!("Automated installation not configured for '{}'", lang)),
            };

            match res {
                Ok((binary, msg)) => {
                    info!("LSP installation succeeded for {}: {}", lang, binary);
                    signal.with_mut(|q| {
                        q.push(LspInstallEvent::Success {
                            language_id: lang,
                            binary_name: binary,
                            message: msg,
                        });
                    });
                }
                Err(err) => {
                    error!("LSP installation failed for {}: {}", lang, err);
                    signal.with_mut(|q| {
                        q.push(LspInstallEvent::Failed {
                            language_id: lang,
                            error: err,
                        });
                    });
                }
            }
        });
    }

    fn install_python(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        // Strategy 1: If npm is installed and has global privileges, try pyright
        if let Ok(output) = Command::new("npm").args(&["--version"]).output() {
            if output.status.success() {
                signal.with_mut(|q| {
                    q.push(LspInstallEvent::Progress {
                        language_id: lang.to_string(),
                        message: "Installing pyright via npm...".to_string(),
                    });
                });
                if let Ok(install_out) = Command::new("npm").args(&["install", "-g", "pyright"]).output() {
                    if install_out.status.success() {
                        return Ok(("pyright".to_string(), "Installed Pyright successfully via npm!".to_string()));
                    }
                }
            }
        }

        // Strategy 2: Dedicated isolated virtual environment ~/.local/share/zenthree/lsp_venv
        let venv_dir = Self::python_venv_dir();
        let pip_bin = venv_dir.join("bin").join("pip");
        let pylsp_bin = venv_dir.join("bin").join("pylsp");

        let _ = std::fs::create_dir_all(Self::zenthree_data_dir());

        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Setting up Python LSP virtual environment...".to_string(),
            });
        });

        // Create venv if it doesn't already exist
        if !pip_bin.exists() {
            let venv_status = Command::new("python3")
                .args(&["-m", "venv", venv_dir.to_str().unwrap_or("")])
                .output();

            match venv_status {
                Ok(out) if out.status.success() => {
                    info!("Created Python LSP virtualenv at {:?}", venv_dir);
                }
                Ok(out) => {
                    let err = String::from_utf8_lossy(&out.stderr);
                    info!("venv creation returned stderr: {}", err);
                }
                Err(e) => {
                    info!("Could not run python3 -m venv: {:?}", e);
                }
            }
        }

        if pip_bin.exists() {
            signal.with_mut(|q| {
                q.push(LspInstallEvent::Progress {
                    language_id: lang.to_string(),
                    message: "Installing python-lsp-server into isolated environment...".to_string(),
                });
            });

            let install_res = Command::new(&pip_bin)
                .args(&["install", "python-lsp-server"])
                .output();

            if let Ok(out) = install_res {
                if out.status.success() || pylsp_bin.exists() {
                    return Ok((
                        pylsp_bin.to_string_lossy().to_string(),
                        "Installed Python LSP (pylsp) successfully!".to_string(),
                    ));
                }
            }
        }

        // Strategy 3: Try user pip directly
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Trying pip install --user python-lsp-server...".to_string(),
            });
        });

        let user_pip = Command::new("pip3")
            .args(&["install", "--user", "--break-system-packages", "python-lsp-server"])
            .output()
            .or_else(|_| {
                Command::new("pip")
                    .args(&["install", "--user", "--break-system-packages", "python-lsp-server"])
                    .output()
            })
            .or_else(|_| {
                Command::new("pip3")
                    .args(&["install", "--user", "python-lsp-server"])
                    .output()
            });

        match user_pip {
            Ok(out) if out.status.success() => {
                Ok(("pylsp".to_string(), "Installed Python LSP (pylsp) via pip!".to_string()))
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                Err(format!("pip install failed: {}", stderr.lines().next().unwrap_or("unknown error")))
            }
            Err(e) => Err(format!("Failed to execute pip: {}", e)),
        }
    }

    fn install_rust(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Adding rust-analyzer component via rustup...".to_string(),
            });
        });

        if let Ok(out) = Command::new("rustup").args(&["component", "add", "rust-analyzer"]).output() {
            if out.status.success() {
                return Ok(("rust-analyzer".to_string(), "Installed rust-analyzer via rustup!".to_string()));
            }
        }

        Err("rustup component add rust-analyzer failed. Please ensure rustup is on your PATH.".to_string())
    }

    fn install_typescript(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing typescript-language-server via npm...".to_string(),
            });
        });

        let npm_res = Command::new("npm")
            .args(&["install", "-g", "typescript", "typescript-language-server"])
            .output();

        match npm_res {
            Ok(out) if out.status.success() => {
                Ok((
                    "typescript-language-server".to_string(),
                    "Installed typescript-language-server via npm!".to_string(),
                ))
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(format!("npm install failed: {}", err.lines().next().unwrap_or("error")))
            }
            Err(e) => Err(format!("npm not found: {}. Please install Node.js/npm first.", e)),
        }
    }

    fn install_go(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing gopls via go install...".to_string(),
            });
        });

        let go_res = Command::new("go")
            .args(&["install", "golang.org/x/tools/gopls@latest"])
            .output();

        match go_res {
            Ok(out) if out.status.success() => {
                Ok(("gopls".to_string(), "Installed gopls via go install!".to_string()))
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(format!("go install failed: {}", err.lines().next().unwrap_or("error")))
            }
            Err(e) => Err(format!("go toolchain not found: {}. Please install Go first.", e)),
        }
    }

    fn install_cpp(_lang: &str, _signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        // clangd is typically packaged with the system
        if let Ok(out) = Command::new("sudo").args(&["apt-get", "install", "-y", "clangd"]).output() {
            if out.status.success() {
                return Ok(("clangd".to_string(), "Installed clangd via apt!".to_string()));
            }
        }
        Err("Please install clangd using your system package manager (e.g. 'sudo apt install clangd' or 'brew install llvm').".to_string())
    }

    fn install_shell(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing bash-language-server via npm...".to_string(),
            });
        });
        if let Ok(out) = Command::new("npm").args(&["install", "-g", "bash-language-server"]).output() {
            if out.status.success() {
                return Ok(("bash-language-server".to_string(), "Installed bash-language-server via npm!".to_string()));
            }
        }
        Err("npm install -g bash-language-server failed. Please ensure Node.js/npm is installed.".to_string())
    }

    fn install_markdown(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing marksman via cargo...".to_string(),
            });
        });
        if let Ok(out) = Command::new("cargo").args(&["install", "--locked", "marksman"]).output() {
            if out.status.success() {
                return Ok(("marksman".to_string(), "Installed marksman via cargo!".to_string()));
            }
        }
        Err("cargo install --locked marksman failed. Please ensure Rust/cargo is available.".to_string())
    }

    fn install_lua(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Checking lua-language-server...".to_string(),
            });
        });
        if let Ok(out) = Command::new("sudo").args(&["apt-get", "install", "-y", "lua-language-server"]).output() {
            if out.status.success() {
                return Ok(("lua-language-server".to_string(), "Installed lua-language-server!".to_string()));
            }
        }
        Err("Please install lua-language-server via your system package manager.".to_string())
    }

    fn install_html_css(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing vscode-langservers-extracted via npm...".to_string(),
            });
        });
        if let Ok(out) = Command::new("npm").args(&["install", "-g", "vscode-langservers-extracted"]).output() {
            if out.status.success() {
                return Ok(("vscode-html-language-server".to_string(), "Installed HTML/CSS language servers via npm!".to_string()));
            }
        }
        Err("npm install -g vscode-langservers-extracted failed. Please install Node.js/npm first.".to_string())
    }

    fn install_json_yaml(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing yaml-language-server via npm...".to_string(),
            });
        });
        if let Ok(out) = Command::new("npm").args(&["install", "-g", "yaml-language-server"]).output() {
            if out.status.success() {
                return Ok(("yaml-language-server".to_string(), "Installed YAML language server via npm!".to_string()));
            }
        }
        Err("npm install -g yaml-language-server failed. Please install Node.js/npm first.".to_string())
    }

    fn install_toml(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing taplo via cargo / npm...".to_string(),
            });
        });

        // Try cargo install taplo-cli
        if let Ok(out) = Command::new("cargo")
            .args(&["install", "--locked", "taplo-cli", "--features", "lsp"])
            .output()
        {
            if out.status.success() {
                return Ok(("taplo".to_string(), "Installed taplo (TOML LSP) via cargo!".to_string()));
            }
        }

        // Try npm install -g @taplo/cli
        if let Ok(out) = Command::new("npm")
            .args(&["install", "-g", "@taplo/cli"])
            .output()
        {
            if out.status.success() {
                return Ok(("taplo".to_string(), "Installed taplo via npm!".to_string()));
            }
        }

        Err("Failed to install taplo. Please run 'cargo install --locked taplo-cli --features lsp' or 'npm install -g @taplo/cli'.".to_string())
    }

    fn install_dockerfile(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing dockerfile-language-server-nodejs via npm...".to_string(),
            });
        });

        if let Ok(out) = Command::new("npm")
            .args(&["install", "-g", "dockerfile-language-server-nodejs"])
            .output()
        {
            if out.status.success() {
                return Ok((
                    "docker-langserver".to_string(),
                    "Installed dockerfile-language-server via npm!".to_string(),
                ));
            }
        }

        Err("npm install -g dockerfile-language-server-nodejs failed. Please ensure Node.js/npm is installed.".to_string())
    }

    fn install_sql(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing sqls via go install...".to_string(),
            });
        });

        if let Ok(out) = Command::new("go")
            .args(&["install", "github.com/sqls-server/sqls@latest"])
            .output()
        {
            if out.status.success() {
                return Ok(("sqls".to_string(), "Installed sqls (SQL LSP) via go install!".to_string()));
            }
        }

        Err("go install github.com/sqls-server/sqls@latest failed. Please ensure Go is installed.".to_string())
    }

    fn install_elixir(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Downloading prebuilt elixir-ls from GitHub Releases...".to_string(),
            });
        });

        let target_dir = Self::zenthree_data_dir().join("lsp").join("elixir-ls");
        let _ = std::fs::create_dir_all(&target_dir);
        let zip_path = Self::zenthree_data_dir().join("lsp").join("elixir-ls.zip");

        // Download release zip from GitHub
        let dl = Command::new("curl")
            .args(&[
                "-sL",
                "https://github.com/elixir-lsp/elixir-ls/releases/latest/download/elixir-ls.zip",
                "-o",
                zip_path.to_str().unwrap_or(""),
            ])
            .output();

        if let Ok(dl_out) = dl {
            if dl_out.status.success() && zip_path.exists() {
                signal.with_mut(|q| {
                    q.push(LspInstallEvent::Progress {
                        language_id: lang.to_string(),
                        message: "Extracting elixir-ls...".to_string(),
                    });
                });

                let _ = Command::new("unzip")
                    .args(&["-q", "-o", zip_path.to_str().unwrap_or(""), "-d", target_dir.to_str().unwrap_or("")])
                    .output();
                let _ = std::fs::remove_file(&zip_path);

                let sh_script = target_dir.join("language_server.sh");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&sh_script) {
                        let mut perms = metadata.permissions();
                        perms.set_mode(0o755);
                        let _ = std::fs::set_permissions(&sh_script, perms);
                    }
                }

                if sh_script.exists() {
                    return Ok((
                        sh_script.to_string_lossy().to_string(),
                        "Installed Elixir LSP (elixir-ls) from GitHub!".to_string(),
                    ));
                }
            }
        }

        // Fallback: try mix archive.install
        if let Ok(mix_out) = Command::new("mix").args(&["archive.install", "hex", "elixir_ls", "--force"]).output() {
            if mix_out.status.success() {
                return Ok(("elixir-ls".to_string(), "Installed elixir-ls via mix!".to_string()));
            }
        }

        Err("Failed to download elixir-ls from GitHub. Please check your internet connection or install manually via mix.".to_string())
    }

    fn install_npm_package(
        lang: &str,
        signal: &ArcSignal<Vec<LspInstallEvent>>,
        pkg: &str,
        bin_name: &str,
    ) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: format!("Installing {} via npm...", pkg),
            });
        });

        let npm_res = Command::new("npm")
            .args(&["install", "-g", pkg])
            .output();

        match npm_res {
            Ok(out) if out.status.success() => {
                Ok((
                    bin_name.to_string(),
                    format!("Installed {} successfully via npm!", pkg),
                ))
            }
            Ok(out) => {
                let err = String::from_utf8_lossy(&out.stderr);
                Err(format!("npm install {} failed: {}", pkg, err.lines().next().unwrap_or("error")))
            }
            Err(e) => Err(format!("npm not found: {}. Please install Node.js/npm first.", e)),
        }
    }
}
