use std::path::{Path, PathBuf};
use std::process::Command;

use crate::acp::AcpClient;

#[derive(Debug, Clone)]
pub enum GooseBinaryStatus {
    Installed {
        binary_path: PathBuf,
        version: String,
    },
    NotFound {
        install_command: String,
    },
}

pub struct GooseSupervisor {
    custom_binary_path: Option<PathBuf>,
}

impl Default for GooseSupervisor {
    fn default() -> Self {
        Self::new()
    }
}

impl GooseSupervisor {
    pub fn new() -> Self {
        Self {
            custom_binary_path: None,
        }
    }

    pub fn with_custom_binary(path: PathBuf) -> Self {
        Self {
            custom_binary_path: Some(path),
        }
    }

    /// Searches standard system paths to locate the `goose` binary.
    pub fn find_binary(&self) -> Option<PathBuf> {
        if let Some(custom) = &self.custom_binary_path {
            if custom.exists() {
                return Some(custom.clone());
            }
        }

        // Try standard `which goose` or `where goose` check
        let which_cmd = if cfg!(windows) { "where" } else { "which" };
        if let Ok(output) = Command::new(which_cmd).arg("goose").output() {
            if output.status.success() {
                let p_str = String::from_utf8_lossy(&output.stdout).trim().lines().next().unwrap_or("").to_string();
                let path = PathBuf::from(p_str);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // Common install locations
        let candidates = [
            dirs::home_dir().map(|h| h.join(".local").join("bin").join("goose")),
            dirs::home_dir().map(|h| h.join(".cargo").join("bin").join("goose")),
            Some(PathBuf::from("/usr/local/bin/goose")),
            Some(PathBuf::from("/usr/bin/goose")),
            Some(PathBuf::from("/opt/homebrew/bin/goose")),
        ];

        candidates.into_iter().flatten().find(|candidate| candidate.exists())
    }

    /// Checks if goose is installed and returns version and status.
    pub fn check_status(&self) -> GooseBinaryStatus {
        if let Some(bin) = self.find_binary() {
            let version = Command::new(&bin)
                .arg("--version")
                .output()
                .ok()
                .and_then(|out| {
                    if out.status.success() {
                        Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "unknown".to_string());

            GooseBinaryStatus::Installed {
                binary_path: bin,
                version,
            }
        } else {
            GooseBinaryStatus::NotFound {
                install_command: "curl -fsSL https://github.com/block/goose/releases/download/stable/download_cli.sh | bash".to_string(),
            }
        }
    }

    /// Spawns an ACP client connected to Goose for the given workspace.
    pub fn spawn_acp_client(&self, workspace_root: &Path) -> Result<AcpClient, String> {
        let bin = self.find_binary().ok_or_else(|| {
            "Goose binary not found on system. Install with: curl -fsSL https://github.com/block/goose/releases/download/stable/download_cli.sh | bash".to_string()
        })?;

        let bin_str = bin.to_string_lossy();
        // Launch Goose in ACP mode
        AcpClient::connect(&bin_str, &["acp"], workspace_root)
    }
}
