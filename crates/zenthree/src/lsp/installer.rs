use std::path::PathBuf;
use std::process::Command;
use log::{info, error};
use std::thread;
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
    /// Get the dedicated Zenthree tools directory across all platforms:
    /// - Linux: ~/.local/share/zenthree
    /// - macOS: ~/Library/Application Support/zenthree or ~/.local/share/zenthree
    /// - Windows: %LOCALAPPDATA%\zenthree
    pub fn zenthree_data_dir() -> PathBuf {
        if let Some(dir) = dirs::data_local_dir().or_else(dirs::data_dir) {
            dir.join("zenthree")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".local").join("share").join("zenthree")
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
                "java" => Self::install_java(&lang, &signal),
                "php" => Self::install_npm_package(&lang, &signal, "intelephense", "intelephense"),
                "csharp" => Self::install_csharp(&lang, &signal),
                "ruby" => Self::install_ruby(&lang, &signal),
                "zig" => Self::install_zig(&lang, &signal),
                "kotlin" => Self::install_kotlin(&lang, &signal),
                "protobuf" => Self::install_npm_package(&lang, &signal, "@bufbuild/buf", "buf"),
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

        // Strategy 2: Dedicated isolated virtual environment in zenthree_data_dir/lsp_venv
        let venv_dir = Self::python_venv_dir();
        let pip_bin = if cfg!(windows) {
            let scripts_pip = venv_dir.join("Scripts").join("pip.exe");
            if scripts_pip.exists() { scripts_pip } else { venv_dir.join("bin").join("pip") }
        } else {
            venv_dir.join("bin").join("pip")
        };
        let pylsp_bin = if cfg!(windows) {
            let scripts_pylsp = venv_dir.join("Scripts").join("pylsp.exe");
            if scripts_pylsp.exists() { scripts_pylsp } else { venv_dir.join("bin").join("pylsp") }
        } else {
            venv_dir.join("bin").join("pylsp")
        };

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
                .output()
                .or_else(|_| {
                    Command::new("python")
                        .args(&["-m", "venv", venv_dir.to_str().unwrap_or("")])
                        .output()
                });

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

    fn install_cpp(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        // 1. Check if clangd is already on the system
        if let Some(path) = super::LspManager::find_executable_path("clangd") {
            return Ok(("clangd".to_string(), format!("Found clangd at {}", path.display())));
        }

        // 2. Direct download of official prebuilt clangd binary from LLVM/clangd GitHub releases (No sudo needed!)
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Downloading official prebuilt clangd language server...".to_string(),
            });
        });

        let data_dir = Self::zenthree_data_dir();
        let clangd_dir = data_dir.join("clangd");
        let bin_dir = data_dir.join("bin");
        let _ = std::fs::create_dir_all(&clangd_dir);
        let _ = std::fs::create_dir_all(&bin_dir);

        let zip_url = if cfg!(target_os = "macos") {
            "https://github.com/clangd/clangd/releases/download/18.1.3/clangd-mac-18.1.3.zip"
        } else if cfg!(target_os = "windows") {
            "https://github.com/clangd/clangd/releases/download/18.1.3/clangd-windows-18.1.3.zip"
        } else {
            "https://github.com/clangd/clangd/releases/download/18.1.3/clangd-linux-18.1.3.zip"
        };
        let zip_path = data_dir.join("clangd.zip");

        let dl_res = Command::new("curl")
            .args(&["-sSL", zip_url, "-o", zip_path.to_str().unwrap_or("clangd.zip")])
            .output();

        let downloaded = match dl_res {
            Ok(o) if o.status.success() && zip_path.exists() => true,
            _ => {
                #[cfg(windows)]
                {
                    Command::new("powershell")
                        .args(&["-Command", &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", zip_url, zip_path.display())])
                        .output()
                        .map(|o| o.status.success() && zip_path.exists())
                        .unwrap_or(false)
                }
                #[cfg(not(windows))]
                { false }
            }
        };

        if !downloaded {
            return Err("Failed to download clangd archive. Please check your internet connection.".to_string());
        }

        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Extracting clangd binary...".to_string(),
            });
        });

        // Unzip clangd archive across platforms
        let zip_str = zip_path.to_str().unwrap_or("");
        let dest_str = clangd_dir.to_str().unwrap_or("");

        #[cfg(windows)]
        {
            let tar_res = Command::new("tar").args(&["-xf", zip_str, "-C", dest_str]).output();
            let extracted = match tar_res {
                Ok(o) if o.status.success() => true,
                _ => false,
            };
            if !extracted {
                let _ = Command::new("powershell")
                    .args(&["-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", zip_str, dest_str)])
                    .output();
            }
        }
        #[cfg(not(windows))]
        {
            let unzip_res = Command::new("unzip").args(&["-q", "-o", zip_str, "-d", dest_str]).output();
            let extracted = match unzip_res {
                Ok(o) if o.status.success() => true,
                _ => false,
            };
            if !extracted {
                let _ = Command::new("tar").args(&["-xf", zip_str, "-C", dest_str]).output();
            }
        }

        let _ = std::fs::remove_file(&zip_path);

        // Find the clangd binary inside clangd_dir (e.g. clangd_18.1.3/bin/clangd[.exe])
        let exe_name = if cfg!(windows) { "clangd.exe" } else { "clangd" };
        let mut clangd_bin = clangd_dir.join("bin").join(exe_name);
        if !clangd_bin.exists() {
            if let Ok(entries) = std::fs::read_dir(&clangd_dir) {
                for entry in entries.flatten() {
                    let sub_bin = entry.path().join("bin").join(exe_name);
                    if sub_bin.exists() {
                        clangd_bin = sub_bin;
                        break;
                    }
                }
            }
        }

        if !clangd_bin.exists() {
            return Err("Failed to locate extracted clangd executable.".to_string());
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&clangd_bin, std::fs::Permissions::from_mode(0o755));
        }

        let target_symlink = bin_dir.join(exe_name);
        let _ = std::fs::remove_file(&target_symlink);
        #[cfg(unix)]
        {
            let _ = std::os::unix::fs::symlink(&clangd_bin, &target_symlink);
            if let Ok(home) = std::env::var("HOME") {
                let user_local_bin = PathBuf::from(home).join(".local").join("bin");
                let _ = std::fs::create_dir_all(&user_local_bin);
                let _ = std::fs::remove_file(user_local_bin.join(exe_name));
                let _ = std::os::unix::fs::symlink(&clangd_bin, user_local_bin.join(exe_name));
            }
        }
        #[cfg(not(unix))]
        let _ = std::fs::copy(&clangd_bin, &target_symlink);

        Ok(("clangd".to_string(), "Installed clangd successfully!".to_string()))
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

    fn install_java(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Checking Java runtime...".to_string(),
            });
        });

        // 1. Check if Java is installed
        let has_java = Command::new("java")
            .arg("-version")
            .output()
            .map(|o| o.status.success() || !o.stderr.is_empty())
            .unwrap_or(false);

        if !has_java {
            return Err("Java runtime (JDK 17+) not found. Please install Java/OpenJDK first (e.g. 'sudo apt install default-jdk' or from adoptium.net).".to_string());
        }

        // 2. Check if jdtls is already found on system
        if let Ok(out) = Command::new("which").arg("jdtls").output() {
            if out.status.success() {
                return Ok(("jdtls".to_string(), "Eclipse JDT Language Server (jdtls) is already installed!".to_string()));
            }
        }

        // 3. Check brew (macOS / Linuxbrew)
        if let Ok(out) = Command::new("brew").args(&["install", "jdtls"]).output() {
            if out.status.success() {
                return Ok(("jdtls".to_string(), "Installed Eclipse JDTLS via Homebrew!".to_string()));
            }
        }

        // 4. Automated direct download from Eclipse official distribution (LTS 1.38.0 compatible with Java 17 and Java 21)
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Downloading Eclipse JDTLS official language server...".to_string(),
            });
        });

        let data_dir = Self::zenthree_data_dir();
        let jdtls_dir = data_dir.join("jdtls");
        let bin_dir = data_dir.join("bin");
        let _ = std::fs::create_dir_all(&jdtls_dir);
        let _ = std::fs::create_dir_all(&bin_dir);

        let tarball_url = "https://download.eclipse.org/jdtls/milestones/1.38.0/jdt-language-server-1.38.0-202408011337.tar.gz";
        let tarball_path = data_dir.join("jdtls.tar.gz");

        // Download via curl (available on Linux, macOS, and modern Windows)
        let dl_res = Command::new("curl")
            .args(&["-sSL", tarball_url, "-o", tarball_path.to_str().unwrap_or("jdtls.tar.gz")])
            .output();

        let downloaded = match dl_res {
            Ok(o) if o.status.success() && tarball_path.exists() => true,
            _ => {
                #[cfg(windows)]
                {
                    Command::new("powershell")
                        .args(&["-Command", &format!("Invoke-WebRequest -Uri '{}' -OutFile '{}'", tarball_url, tarball_path.display())])
                        .output()
                        .map(|o| o.status.success())
                        .unwrap_or(false)
                }
                #[cfg(not(windows))]
                { false }
            }
        };

        if !downloaded {
            return Err("Failed to download Eclipse JDTLS archive. Please check your internet connection.".to_string());
        }

        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Extracting Eclipse JDTLS...".to_string(),
            });
        });

        // Extract tarball using tar
        let ext_res = Command::new("tar")
            .args(&["-xzf", tarball_path.to_str().unwrap_or(""), "-C", jdtls_dir.to_str().unwrap_or("")])
            .output();

        let _ = std::fs::remove_file(&tarball_path);

        match ext_res {
            Ok(o) if o.status.success() => {}
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                return Err(format!("Failed to extract jdtls archive: {}", err));
            }
            Err(e) => return Err(format!("Failed to run tar command: {}", e)),
        }

        let jdtls_script = jdtls_dir.join("bin").join("jdtls");
        #[cfg(windows)]
        let jdtls_script = jdtls_dir.join("bin").join("jdtls.bat");

        if !jdtls_script.exists() {
            return Err("Eclipse JDTLS extraction completed, but launcher binary was not found.".to_string());
        }

        // Link/copy into zenthree bin dir and user ~/.local/bin
        let target_symlink = bin_dir.join("jdtls");
        #[cfg(windows)]
        let target_symlink = bin_dir.join("jdtls.bat");

        #[cfg(unix)]
        {
            let _ = std::fs::remove_file(&target_symlink);
            let _ = std::os::unix::fs::symlink(&jdtls_script, &target_symlink);
            if let Ok(home) = std::env::var("HOME") {
                let user_local_bin = PathBuf::from(home).join(".local").join("bin");
                let _ = std::fs::create_dir_all(&user_local_bin);
                let _ = std::fs::remove_file(user_local_bin.join("jdtls"));
                let _ = std::os::unix::fs::symlink(&jdtls_script, user_local_bin.join("jdtls"));
            }
        }
        #[cfg(windows)]
        {
            let _ = std::fs::copy(&jdtls_script, &target_symlink);
        }

        Ok((
            "jdtls".to_string(),
            "Installed Eclipse JDT Language Server (jdtls) successfully!".to_string(),
        ))
    }

    fn install_csharp(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing csharp-ls via dotnet tool...".to_string(),
            });
        });

        if let Ok(out) = Command::new("dotnet").args(&["tool", "install", "-g", "csharp-ls"]).output() {
            if out.status.success() {
                return Ok(("csharp-ls".to_string(), "Installed csharp-ls via dotnet tool!".to_string()));
            }
        }

        Err("Failed to install csharp-ls. Please ensure .NET SDK is installed ('dotnet tool install -g csharp-ls').".to_string())
    }

    fn install_ruby(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Installing solargraph via gem...".to_string(),
            });
        });

        if let Ok(out) = Command::new("gem").args(&["install", "solargraph"]).output() {
            if out.status.success() {
                return Ok(("solargraph".to_string(), "Installed solargraph via gem!".to_string()));
            }
        }

        Err("Failed to install solargraph. Please ensure Ruby and gem are installed ('gem install solargraph').".to_string())
    }

    fn install_zig(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Checking zls (Zig Language Server)...".to_string(),
            });
        });

        if let Ok(out) = Command::new("brew").args(&["install", "zls"]).output() {
            if out.status.success() {
                return Ok(("zls".to_string(), "Installed zls via Homebrew!".to_string()));
            }
        }

        Err("Please install zls using 'brew install zls' or download from github.com/zigtools/zls/releases.".to_string())
    }

    fn install_kotlin(lang: &str, signal: &ArcSignal<Vec<LspInstallEvent>>) -> Result<(String, String), String> {
        signal.with_mut(|q| {
            q.push(LspInstallEvent::Progress {
                language_id: lang.to_string(),
                message: "Checking kotlin-language-server...".to_string(),
            });
        });

        if let Ok(out) = Command::new("brew").args(&["install", "kotlin-language-server"]).output() {
            if out.status.success() {
                return Ok(("kotlin-language-server".to_string(), "Installed kotlin-language-server via brew!".to_string()));
            }
        }

        Err("Please install kotlin-language-server via brew or github.com/fwcd/kotlin-language-server/releases.".to_string())
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

    /// Completely delete/uninstall an installed language server from disk
    pub fn uninstall_server(language_id: &str) -> Result<(), String> {
        let data_dir = Self::zenthree_data_dir();
        let bin_dir = data_dir.join("bin");

        match language_id {
            "cpp" | "c" => {
                let clangd_dir = data_dir.join("clangd");
                let _ = std::fs::remove_dir_all(&clangd_dir);
                let _ = std::fs::remove_file(bin_dir.join("clangd"));
                let _ = std::fs::remove_file(bin_dir.join("clangd.exe"));
                if let Some(home) = dirs::home_dir() {
                    let _ = std::fs::remove_file(home.join(".local").join("bin").join("clangd"));
                    let _ = std::fs::remove_file(home.join(".local").join("bin").join("clangd.exe"));
                }
            }
            "java" => {
                let jdtls_dir = data_dir.join("jdtls");
                let _ = std::fs::remove_dir_all(&jdtls_dir);
                let _ = std::fs::remove_file(bin_dir.join("jdtls"));
                let _ = std::fs::remove_file(bin_dir.join("jdtls.bat"));
                if let Some(home) = dirs::home_dir() {
                    let _ = std::fs::remove_file(home.join(".local").join("bin").join("jdtls"));
                    let _ = std::fs::remove_file(home.join(".local").join("bin").join("jdtls.bat"));
                }
            }
            "python" => {
                let venv_dir = Self::python_venv_dir();
                let _ = std::fs::remove_dir_all(&venv_dir);
                let _ = Command::new("npm").args(&["uninstall", "-g", "pyright"]).output();
            }
            "elixir" => {
                let elixir_dir = data_dir.join("lsp").join("elixir-ls");
                let _ = std::fs::remove_dir_all(&elixir_dir);
            }
            "typescript" | "javascript" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "typescript", "typescript-language-server"]).output();
            }
            "shell" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "bash-language-server"]).output();
            }
            "html_css" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "vscode-langservers-extracted"]).output();
            }
            "json_yaml" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "yaml-language-server", "vscode-langservers-extracted"]).output();
            }
            "dockerfile" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "dockerfile-language-server-nodejs"]).output();
            }
            "sql" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "sql-language-server"]).output();
            }
            "php" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "intelephense"]).output();
            }
            "vue" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "@vue/language-server"]).output();
            }
            "svelte" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "svelte-language-server"]).output();
            }
            "graphql" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "graphql-language-service-cli"]).output();
            }
            "prisma" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "@prisma/language-server"]).output();
            }
            "astro" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "@astrojs/language-server"]).output();
            }
            "tailwind" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "@tailwindcss/language-server"]).output();
            }
            "mdx" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "@mdx-js/language-server"]).output();
            }
            "angular" => {
                let _ = Command::new("npm").args(&["uninstall", "-g", "@angular/language-server"]).output();
            }
            _ => {}
        }

        super::LspManager::invalidate_cache();
        Ok(())
    }
}
