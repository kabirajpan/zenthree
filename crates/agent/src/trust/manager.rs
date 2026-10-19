use std::collections::HashSet;
use std::fs;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use super::status::TrustStatus;

#[derive(Debug, Serialize, Deserialize, Default)]
struct TrustStore {
    #[serde(default = "default_version")]
    version: u32,
    #[serde(default)]
    trusted_workspaces: HashSet<String>,
    #[serde(default)]
    untrusted_workspaces: HashSet<String>,
}

fn default_version() -> u32 {
    1
}

/// Manages workspace trust records and persists them to disk.
pub struct WorkspaceTrustManager {
    store_path: PathBuf,
}

impl Default for WorkspaceTrustManager {
    fn default() -> Self {
        Self::with_default_path()
    }
}

impl WorkspaceTrustManager {
    /// Create trust manager pointing to the default persistence file.
    pub fn with_default_path() -> Self {
        let store_path = if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("zene").join("trusted_workspaces.json")
        } else if let Some(home) = dirs::home_dir() {
            home.join(".zene").join("trusted_workspaces.json")
        } else {
            PathBuf::from("trusted_workspaces.json")
        };
        Self { store_path }
    }

    /// Create trust manager with a specific JSON store file path (useful for testing).
    pub fn with_path(store_path: PathBuf) -> Self {
        Self { store_path }
    }

    /// Resolves canonical workspace path to defeat symlink traversal and relative path trickery.
    pub fn canonicalize_path(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| {
            if path.is_absolute() {
                path.to_path_buf()
            } else if let Ok(cwd) = std::env::current_dir() {
                cwd.join(path)
            } else {
                path.to_path_buf()
            }
        })
    }

    /// Check if workspace trust enforcement is globally disabled via environment variable.
    pub fn is_trust_disabled() -> bool {
        let vars = ["ZENE_DISABLE_WORKSPACE_TRUST", "AGY_ACP_DISABLE_WORKSPACE_TRUST"];
        for var in &vars {
            if let Ok(val) = std::env::var(var) {
                let v = val.trim().to_lowercase();
                if v == "1" || v == "true" || v == "yes" {
                    return true;
                }
            }
        }
        false
    }

    /// Evaluate trust status for the given workspace path.
    pub fn check_trust(&self, workspace: &Path) -> TrustStatus {
        if Self::is_trust_disabled() {
            return TrustStatus::Trusted;
        }

        let canonical = Self::canonicalize_path(workspace);
        let path_str = canonical.to_string_lossy().to_string();

        let store = self.load_store();
        if store.trusted_workspaces.contains(&path_str) {
            TrustStatus::Trusted
        } else if store.untrusted_workspaces.contains(&path_str) {
            TrustStatus::Untrusted
        } else {
            TrustStatus::Unknown
        }
    }

    /// Convenience helper returning true if workspace is explicitly trusted or trust checks are disabled.
    pub fn is_trusted(&self, workspace: &Path) -> bool {
        self.check_trust(workspace) == TrustStatus::Trusted
    }

    /// Set trust status for a workspace path and persist to disk.
    pub fn set_trust(&self, workspace: &Path, status: TrustStatus) -> Result<(), std::io::Error> {
        let canonical = Self::canonicalize_path(workspace);
        let path_str = canonical.to_string_lossy().to_string();

        let mut store = self.load_store();
        match status {
            TrustStatus::Trusted => {
                store.untrusted_workspaces.remove(&path_str);
                store.trusted_workspaces.insert(path_str);
            }
            TrustStatus::Untrusted => {
                store.trusted_workspaces.remove(&path_str);
                store.untrusted_workspaces.insert(path_str);
            }
            TrustStatus::Unknown => {
                store.trusted_workspaces.remove(&path_str);
                store.untrusted_workspaces.remove(&path_str);
            }
        }

        self.save_store(&store)
    }

    fn load_store(&self) -> TrustStore {
        if !self.store_path.exists() {
            return TrustStore::default();
        }

        fs::read_to_string(&self.store_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }

    fn save_store(&self, store: &TrustStore) -> Result<(), std::io::Error> {
        if let Some(parent) = self.store_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(store)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.store_path, json)
    }
}
