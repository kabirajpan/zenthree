pub mod hook;
pub mod manager;
pub mod status;

pub use hook::WorkspaceTrustHook;
pub use manager::WorkspaceTrustManager;
pub use status::TrustStatus;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use serde_json::json;
    use crate::orchestrator::hooks::{AgentHook, HookDecision};
    use crate::traits::tool::ToolRisk;

    #[test]
    fn test_trust_manager_persistence() {
        let temp_dir = std::env::temp_dir().join("zene_test_trust_manager");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        let store_file = temp_dir.join("trusted_workspaces.json");
        let manager = WorkspaceTrustManager::with_path(store_file);

        let ws_a = temp_dir.join("project_a");
        let ws_b = temp_dir.join("project_b");
        fs::create_dir_all(&ws_a).unwrap();
        fs::create_dir_all(&ws_b).unwrap();

        // 1. Initial state is Unknown
        assert_eq!(manager.check_trust(&ws_a), TrustStatus::Unknown);
        assert!(!manager.is_trusted(&ws_a));

        // 2. Mark ws_a as Trusted
        manager.set_trust(&ws_a, TrustStatus::Trusted).unwrap();
        assert_eq!(manager.check_trust(&ws_a), TrustStatus::Trusted);
        assert!(manager.is_trusted(&ws_a));

        // 3. Mark ws_b as Untrusted
        manager.set_trust(&ws_b, TrustStatus::Untrusted).unwrap();
        assert_eq!(manager.check_trust(&ws_b), TrustStatus::Untrusted);
        assert!(!manager.is_trusted(&ws_b));

        // 4. Reload from disk with fresh manager instance
        let fresh_manager = WorkspaceTrustManager::with_path(temp_dir.join("trusted_workspaces.json"));
        assert_eq!(fresh_manager.check_trust(&ws_a), TrustStatus::Trusted);
        assert_eq!(fresh_manager.check_trust(&ws_b), TrustStatus::Untrusted);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_untrusted_workspace_hook_blocks_terminal() {
        let hook = WorkspaceTrustHook::new(TrustStatus::Untrusted);

        // run_terminal must be explicitly denied with policy error
        let decision = hook.decide("run_terminal", &json!({ "command": "ls" }), ToolRisk::Mutating);
        match decision {
            HookDecision::Deny(msg) => {
                assert!(msg.contains("Workspace is untrusted"));
                assert!(msg.contains("terminal command execution is blocked"));
            }
            other => panic!("Expected HookDecision::Deny, got {:?}", other),
        }

        // write_file must require interactive user approval
        let write_decision = hook.decide("write_file", &json!({ "path": "test.txt" }), ToolRisk::Destructive);
        match write_decision {
            HookDecision::RequireApproval(msg) => {
                assert!(msg.contains("requires explicit user authorization"));
            }
            other => panic!("Expected HookDecision::RequireApproval, got {:?}", other),
        }

        // read_file (ReadOnly) must be allowed
        let read_decision = hook.decide("read_file", &json!({ "path": "test.txt" }), ToolRisk::ReadOnly);
        assert_eq!(read_decision, HookDecision::Allow);
    }

    #[test]
    fn test_trusted_workspace_hook_allows_execution() {
        let hook = WorkspaceTrustHook::new(TrustStatus::Trusted);

        let term_res = hook.decide("run_terminal", &json!({ "command": "cargo test" }), ToolRisk::Mutating);
        assert_eq!(term_res, HookDecision::Allow);

        let write_res = hook.decide("write_file", &json!({ "path": "main.rs" }), ToolRisk::Destructive);
        assert_eq!(write_res, HookDecision::Allow);
    }
}
