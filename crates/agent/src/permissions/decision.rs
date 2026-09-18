use serde::{Deserialize, Serialize};

/// The engine's decision for a tool invocation request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum ToolPermissionDecision {
    /// Action is safe to proceed without prompting the user.
    Allow,
    /// Action requires explicit user confirmation via the UI.
    Confirm {
        reason: String,
        suggested_action: String,
    },
    /// Action is strictly denied and cannot proceed.
    Deny {
        reason: String,
    },
}

impl ToolPermissionDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow)
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, Self::Deny { .. })
    }

    pub fn is_confirmation_required(&self) -> bool {
        matches!(self, Self::Confirm { .. })
    }
}

/// The user's response to an interactive tool confirmation prompt.
/// Synthesizes Goose and Zed permission models.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserPermissionResponse {
    /// Allow this single execution.
    AllowOnce,
    /// Allow this tool or command pattern for the rest of the session.
    AlwaysAllow,
    /// Deny this single execution (the agent receives an error and may try another approach).
    DenyOnce,
    /// Deny this tool or command pattern for the rest of the session.
    AlwaysDeny,
    /// Cancel the entire active agent turn immediately.
    Cancel,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_states() {
        let allow = ToolPermissionDecision::Allow;
        assert!(allow.is_allowed());
        assert!(!allow.is_denied());

        let deny = ToolPermissionDecision::Deny {
            reason: "blocked".to_string(),
        };
        assert!(deny.is_denied());
        assert!(!deny.is_allowed());

        let confirm = ToolPermissionDecision::Confirm {
            reason: "mutates file".to_string(),
            suggested_action: "Allow edit".to_string(),
        };
        assert!(confirm.is_confirmation_required());
    }
}
