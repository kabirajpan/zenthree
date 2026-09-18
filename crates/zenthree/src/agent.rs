//! Agent subsystem re-exports and integration for Zenthree
//!
//! Re-exports all autonomous engineering agent components from the modern `agent` crate,
//! including ACP (Agent Client Protocol), Goose Process Supervisor, and the built-in Native loop.

pub use agent::*;
pub use agent::tools::ground_workspace_path;
