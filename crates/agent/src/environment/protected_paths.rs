use std::path::{Component, Path, PathBuf};

/// Guards invariant files and directories from unauthorized modification or sensitive reading.
/// Mirrors Zed's strict Git metadata protection and Goose's credential guard.
#[derive(Debug, Default, Clone)]
pub struct ProtectedPathGuard;

impl ProtectedPathGuard {
    /// Checks if writing to this path is strictly blocked by invariant safety rules.
    /// Blocks all `.git/` metadata writes, environment secret files, and system keys.
    pub fn is_write_blocked(path: &Path) -> bool {
        // 1. Block any write under a `.git` component
        for comp in path.components() {
            if let Component::Normal(os_str) = comp {
                let name = os_str.to_string_lossy();
                if name.eq_ignore_ascii_case(".git") {
                    return true;
                }
            }
        }

        // 2. Block secret configuration files
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            let lower = file_name.to_ascii_lowercase();

            // .env files (.env, .env.local, .env.production, etc.)
            if lower == ".env" || lower.starts_with(".env.") || lower.ends_with(".env") {
                return true;
            }

            // Common secret extensions
            if lower.ends_with(".pem")
                || lower.ends_with(".key")
                || lower.ends_with(".pfx")
                || lower.ends_with(".p12")
                || lower.ends_with(".pkcs12")
                || lower.ends_with(".secret")
            {
                return true;
            }

            // Private keys and credentials
            if lower.starts_with("id_rsa")
                || lower.starts_with("id_ed25519")
                || lower.starts_with("id_ecdsa")
                || lower.starts_with("id_dsa")
                || lower == "credentials"
                || lower == "credentials.json"
                || lower == "service-account.json"
            {
                return true;
            }
        }

        // 3. User directory sensitive stores
        if let Some(home) = dirs::home_dir() {
            let sensitive_user_dirs = [
                home.join(".ssh"),
                home.join(".aws"),
                home.join(".gnupg"),
                home.join(".config").join("gh"),
            ];

            for sensitive in &sensitive_user_dirs {
                if path.starts_with(sensitive) {
                    return true;
                }
            }
        }

        false
    }

    /// Checks if reading this path should be flagged as accessing sensitive secrets.
    pub fn is_sensitive_read(path: &Path) -> bool {
        // Environment secret files are sensitive
        if let Some(file_name) = path.file_name().and_then(|f| f.to_str()) {
            let lower = file_name.to_ascii_lowercase();
            if lower == ".env"
                || lower.starts_with(".env.")
                || lower.ends_with(".pem")
                || lower.ends_with(".key")
                || lower.ends_with(".secret")
                || lower.starts_with("id_rsa")
                || lower.starts_with("id_ed25519")
            {
                return true;
            }
        }

        // Home directory credential paths are sensitive
        if let Some(home) = dirs::home_dir() {
            let sensitive_dirs = [home.join(".ssh"), home.join(".aws")];
            for dir in &sensitive_dirs {
                if path.starts_with(dir) {
                    return true;
                }
            }
        }

        false
    }

    /// Returns candidate protected `.git` directory paths for a given workspace.
    pub fn protected_git_dirs(workspace_root: &Path) -> Vec<PathBuf> {
        vec![
            workspace_root.join(".git"),
            workspace_root.join(".git").join("hooks"),
            workspace_root.join(".git").join("config"),
            workspace_root.join(".git").join("modules"),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_directory_write_blocked() {
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/.git")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/.git/hooks/pre-commit")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/.git/config")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new(".git/HEAD")));
    }

    #[test]
    fn test_env_and_secret_files_write_blocked() {
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/.env")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/.env.local")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/.env.production")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/server.key")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/cert.pem")));
        assert!(ProtectedPathGuard::is_write_blocked(Path::new("/project/id_rsa")));
    }

    #[test]
    fn test_normal_files_allowed() {
        assert!(!ProtectedPathGuard::is_write_blocked(Path::new("/project/src/main.rs")));
        assert!(!ProtectedPathGuard::is_write_blocked(Path::new("/project/Cargo.toml")));
        assert!(!ProtectedPathGuard::is_write_blocked(Path::new("/project/README.md")));
        assert!(!ProtectedPathGuard::is_write_blocked(Path::new("/project/tests/unit_test.rs")));
    }
}
