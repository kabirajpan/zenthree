//! Agent subsystem re-exports and integration for Zenthree
//!
//! Re-exports all autonomous engineering agent components from the modern `ai` crate,
//! including ACP (Agent Client Protocol), Goose Process Supervisor, and the built-in Native loop.

pub use ai::*;
pub use ai::tools::ground_workspace_path;
