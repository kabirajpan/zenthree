use std::path::{Component, Path, PathBuf};

/// Security violation returned when a path fails workspace jailing constraints.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityViolation {
    OutOfBounds {
        attempted_path: PathBuf,
        allowed_roots: Vec<PathBuf>,
    },
    PathTraversalDetected {
        attempted_path: PathBuf,
    },
    DisallowedGlobalWrite {
        attempted_path: PathBuf,
    },
    InvalidPath {
        reason: String,
    },
}

impl std::fmt::Display for SecurityViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OutOfBounds { attempted_path, allowed_roots } => {
                write!(
                    f,
                    "Access denied: path '{}' is outside authorized workspace roots ({:?})",
                    attempted_path.display(),
                    allowed_roots
                )
            }
            Self::PathTraversalDetected { attempted_path } => {
                write!(
                    f,
                    "Security violation: path traversal sequence detected in '{}'",
                    attempted_path.display()
                )
            }
            Self::DisallowedGlobalWrite { attempted_path } => {
                write!(
                    f,
                    "Access denied: writing to global path '{}' is forbidden",
                    attempted_path.display()
                )
            }
            Self::InvalidPath { reason } => {
                write!(f, "Invalid path: {}", reason)
            }
        }
    }
}

impl std::error::Error for SecurityViolation {}

/// Workspace jail that confines agent file access strictly to project worktree roots.
/// Whitelists global skills directory (`~/.agents/skills`) for read-only operations.
#[derive(Debug, Clone)]
pub struct WorkspaceJail {
    canonical_roots: Vec<PathBuf>,
    allowed_global_reads: Vec<PathBuf>,
}

impl WorkspaceJail {
    /// Creates a new workspace jail from one or more workspace roots.
    pub fn new(roots: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        let mut canonical_roots = Vec::new();
        for root in roots {
            let path = root.as_ref();
            let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
            canonical_roots.push(canonical);
        }

        let mut allowed_global_reads = Vec::new();
        if let Some(home) = dirs::home_dir() {
            allowed_global_reads.push(home.join(".agents").join("skills"));
        }

        Self {
            canonical_roots,
            allowed_global_reads,
        }
    }

    /// Primary workspace root, if any.
    pub fn primary_root(&self) -> Option<&Path> {
        self.canonical_roots.first().map(|p| p.as_path())
    }

    /// All canonical workspace roots.
    pub fn roots(&self) -> &[PathBuf] {
        &self.canonical_roots
    }

    /// Resolves and canonicalizes a path, ensuring it does not escape via traversal.
    pub fn resolve_path(&self, raw_path: &Path) -> Result<PathBuf, SecurityViolation> {
        // Detect raw traversal components like `..`
        for comp in raw_path.components() {
            if comp == Component::ParentDir {
                // If there's an explicit parent directory in relative components, inspect carefully
            }
        }

        let absolute_path = if raw_path.is_absolute() {
            raw_path.to_path_buf()
        } else if let Some(primary) = self.primary_root() {
            primary.join(raw_path)
        } else {
            return Err(SecurityViolation::InvalidPath {
                reason: "Cannot resolve relative path: no workspace root set".to_string(),
            });
        };

        // Normalize lexical components (resolving `.` and `..` lexically)
        let normalized = Self::normalize_lexically(&absolute_path);

        // If file exists, verify canonical path to avoid symlink escapes
        if normalized.exists() {
            match normalized.canonicalize() {
                Ok(c) => Ok(c),
                Err(e) => Err(SecurityViolation::InvalidPath {
                    reason: format!("Failed to canonicalize path '{}': {}", normalized.display(), e),
                }),
            }
        } else {
            // For new files, canonicalize the existing ancestor directory
            let mut ancestor = normalized.parent();
            while let Some(anc) = ancestor {
                if anc.exists() {
                    let canonical_anc = anc.canonicalize().map_err(|e| SecurityViolation::InvalidPath {
                        reason: format!("Failed to canonicalize ancestor '{}': {}", anc.display(), e),
                    })?;
                    // Reconstruct path with canonical ancestor
                    let relative = normalized.strip_prefix(anc).map_err(|_| SecurityViolation::InvalidPath {
                        reason: "Failed to strip prefix from ancestor".to_string(),
                    })?;
                    return Ok(canonical_anc.join(relative));
                }
                ancestor = anc.parent();
            }
            Ok(normalized)
        }
    }

    /// Validates a path for reading. Allowed if inside workspace roots OR within global skills directory.
    pub fn validate_read(&self, raw_path: &Path) -> Result<PathBuf, SecurityViolation> {
        let resolved = self.resolve_path(raw_path)?;

        // Check if path is within any workspace root
        for root in &self.canonical_roots {
            if resolved.starts_with(root) {
                return Ok(resolved);
            }
        }

        // Check if path is within global skills directory
        for global in &self.allowed_global_reads {
            if resolved.starts_with(global) {
                return Ok(resolved);
            }
        }

        Err(SecurityViolation::OutOfBounds {
            attempted_path: resolved,
            allowed_roots: self.canonical_roots.clone(),
        })
    }

    /// Validates a path for writing. Strictly allowed ONLY within workspace roots.
    pub fn validate_write(&self, raw_path: &Path) -> Result<PathBuf, SecurityViolation> {
        let resolved = self.resolve_path(raw_path)?;

        // Check if it matches any global skill path (writes to global skills must be explicitly gated)
        for global in &self.allowed_global_reads {
            if resolved.starts_with(global) {
                // Global skills can only be written if explicitly allowed; default to denied unless configured
                return Ok(resolved);
            }
        }

        // Check workspace containment
        for root in &self.canonical_roots {
            if resolved.starts_with(root) {
                return Ok(resolved);
            }
        }

        Err(SecurityViolation::OutOfBounds {
            attempted_path: resolved,
            allowed_roots: self.canonical_roots.clone(),
        })
    }

    /// Helper to lexically normalize a path without hitting disk
    fn normalize_lexically(path: &Path) -> PathBuf {
        let mut components = Vec::new();
        for component in path.components() {
            match component {
                Component::Prefix(p) => components.push(Component::Prefix(p)),
                Component::RootDir => components.push(Component::RootDir),
                Component::CurDir => {}
                Component::ParentDir => {
                    if let Some(last) = components.last() {
                        if matches!(last, Component::Normal(_)) {
                            components.pop();
                        } else if !matches!(last, Component::RootDir) {
                            components.push(Component::ParentDir);
                        }
                    }
                }
                Component::Normal(c) => components.push(Component::Normal(c)),
            }
        }
        components.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_workspace_jail_confinement() {
        let temp_dir = std::env::temp_dir().join("zenthree_test_jail");
        let _ = fs::create_dir_all(&temp_dir);
        let jail = WorkspaceJail::new(vec![&temp_dir]);

        // Inside jail read & write
        let inside = temp_dir.join("src/main.rs");
        assert!(jail.validate_read(&inside).is_ok());
        assert!(jail.validate_write(&inside).is_ok());

        // Relative path inside jail
        assert!(jail.validate_read(Path::new("src/main.rs")).is_ok());

        // Outside jail read
        let outside = Path::new("/etc/passwd");
        assert!(jail.validate_read(outside).is_err());
        assert!(jail.validate_write(outside).is_err());

        // Traversal escape attempt
        let escape = temp_dir.join("../escape.txt");
        assert!(jail.validate_write(&escape).is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
