pub mod acp;
pub mod antigravity_cli;
pub mod installer;

pub use acp::{AcpAgentConfig, AcpClient, AcpEvent, AcpMode, AcpModel, AcpSlashCommand};
pub use antigravity_cli::{
    available_models, create_antigravity_provider, AntigravityAuthManager, AntigravityCliProvider,
    ModelInfo, OAuthToken,
};
pub use installer::{install_antigravity, uninstall_antigravity, zenthree_antigravity_dir};

pub fn is_antigravity_installed() -> bool {
    acp::config::detect_antigravity_binary().is_some()
}

