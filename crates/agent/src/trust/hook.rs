use serde_json::Value;

use crate::orchestrator::hooks::{AgentHook, HookDecision};
use crate::traits::tool::ToolRisk;
use super::status::TrustStatus;

/// Lifecycle security hook enforcing workspace trust boundaries in accordance with
/// `AGENT_ENTERPRISE_ADMIN_CONTROLS_AND_WORKSPACE_TRUST.md`.
pub struct WorkspaceTrustHook {
    status: TrustStatus,
}

impl WorkspaceTrustHook {
    pub fn new(status: TrustStatus) -> Self {
        Self { status }
    }

    pub fn status(&self) -> TrustStatus {
        self.status
    }

    pub fn set_status(&mut self, status: TrustStatus) {
        self.status = status;
    }
}

impl AgentHook for WorkspaceTrustHook {
    fn name(&self) -> &str {
        "WorkspaceTrustEnforcement"
    }

    fn decide(&self, tool_name: &str, _args: &Value, risk: ToolRisk) -> HookDecision {
        match self.status {
            TrustStatus::Trusted => HookDecision::Allow,
            TrustStatus::Untrusted => {
                // Strict isolation: completely block terminal execution in untrusted workspaces
                if tool_name == "run_terminal" {
                    HookDecision::Deny(
                        "Workspace is untrusted. Automated shell and terminal command execution is blocked to protect the host environment.".to_string(),
                    )
                } else if risk != ToolRisk::ReadOnly {
                    // Any mutating filesystem action demands interactive user authorization
                    HookDecision::RequireApproval(format!(
                        "Workspace is untrusted. File mutation tool '{}' requires explicit user authorization.",
                        tool_name
                    ))
                } else {
                    HookDecision::Allow
                }
            }
            TrustStatus::Unknown => {
                // Unverified workspace: require approval for terminal or destructive actions
                if tool_name == "run_terminal" {
                    HookDecision::RequireApproval(
                        "Workspace trust is unverified. Shell execution requires explicit confirmation.".to_string(),
                    )
                } else if risk == ToolRisk::Destructive {
                    HookDecision::RequireApproval(format!(
                        "Workspace trust is unverified. Destructive action '{}' requires explicit confirmation.",
                        tool_name
                    ))
                } else {
                    HookDecision::Allow
                }
            }
        }
    }
}
