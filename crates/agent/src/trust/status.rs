use serde::{Deserialize, Serialize};

/// The trust classification for a workspace directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustStatus {
    /// Workspace is trusted. Full autonomy, mutating tools, and terminal commands are enabled.
    Trusted,
    /// Workspace is untrusted. Execution is strictly sandboxed: terminal commands blocked, mutations locked.
    Untrusted,
    /// Initial unverified state; requires user acknowledgement before granting execution autonomy.
    Unknown,
}

impl Default for TrustStatus {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for TrustStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trusted => write!(f, "trusted"),
            Self::Untrusted => write!(f, "untrusted"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}
