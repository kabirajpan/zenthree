pub mod hardcoded_denials;
pub mod command_parser;
pub mod decision;
pub mod rules;

pub use hardcoded_denials::{HardcodedSecurityRules, HARDCODED_SECURITY_DENIAL};
pub use command_parser::ShellCommandParser;
pub use decision::{ToolPermissionDecision, UserPermissionResponse};
pub use rules::{CompiledRule, PermissionPolicyTable, ToolPermissionMode, ToolRuleSet};

use std::sync::{Arc, Mutex};

/// Combined check ensuring no subcommand in a compound command violates hardcoded security rules.
pub fn check_hardcoded_security_rules(command_line: &str) -> Result<(), &'static str> {
    // 1. Check full command string
    if let Some(reason) = HardcodedSecurityRules::check_command(command_line) {
        return Err(reason);
    }

    // 2. Extract and check individual sub-commands
    let subcommands = ShellCommandParser::extract_subcommands(command_line);
    for sub in subcommands {
        if let Some(reason) = HardcodedSecurityRules::check_command(&sub) {
            return Err(reason);
        }
    }

    Ok(())
}

/// The unified PermissionEngine coordinating policy evaluation, compound command inspection,
/// and interactive user permission grants.
#[derive(Debug, Clone)]
pub struct PermissionEngine {
    policy_table: Arc<Mutex<PermissionPolicyTable>>,
}

impl Default for PermissionEngine {
    fn default() -> Self {
        Self {
            policy_table: Arc::new(Mutex::new(PermissionPolicyTable::default())),
        }
    }
}

impl PermissionEngine {
    pub fn new() -> Self {
        Self::default()
    }

    /// Evaluates permission for a tool call. If the tool is terminal execution,
    /// all sub-commands are unpacked and verified.
    pub fn evaluate(&self, tool_name: &str, input: &str) -> ToolPermissionDecision {
        let table = self.policy_table.lock().unwrap();

        if tool_name == "terminal" || tool_name == "run_command" {
            // 1. Check hardcoded security rules across full string and sub-commands
            if let Err(reason) = check_hardcoded_security_rules(input) {
                return ToolPermissionDecision::Deny {
                    reason: format!("Hardcoded security rule: {}", reason),
                };
            }

            // 2. Extract sub-commands
            let subcommands = ShellCommandParser::extract_subcommands(input);
            let mut any_confirm = false;
            let mut confirm_reason = String::new();
            let mut confirm_action = String::new();

            for sub in &subcommands {
                let dec = table.evaluate(tool_name, sub);
                match dec {
                    ToolPermissionDecision::Deny { reason } => {
                        // Short circuit on ANY sub-command denial
                        return ToolPermissionDecision::Deny { reason };
                    }
                    ToolPermissionDecision::Confirm { reason, suggested_action } => {
                        any_confirm = true;
                        confirm_reason = reason;
                        confirm_action = suggested_action;
                    }
                    ToolPermissionDecision::Allow => {}
                }
            }

            if any_confirm {
                return ToolPermissionDecision::Confirm {
                    reason: confirm_reason,
                    suggested_action: confirm_action,
                };
            }

            // If all sub-commands passed without deny or confirm, allow
            ToolPermissionDecision::Allow
        } else {
            table.evaluate(tool_name, input)
        }
    }

    /// Records the user's interactive response to a confirmation prompt.
    pub fn handle_user_response(
        &self,
        tool_name: &str,
        input: &str,
        response: UserPermissionResponse,
    ) -> ToolPermissionDecision {
        let mut table = self.policy_table.lock().unwrap();
        let grant_key = format!("{}:{}", tool_name, input);

        match response {
            UserPermissionResponse::AllowOnce => ToolPermissionDecision::Allow,
            UserPermissionResponse::AlwaysAllow => {
                table.grant_session_allow(&grant_key);
                table.grant_session_allow(tool_name);
                ToolPermissionDecision::Allow
            }
            UserPermissionResponse::DenyOnce => ToolPermissionDecision::Deny {
                reason: "Denied by user (single execution)".to_string(),
            },
            UserPermissionResponse::AlwaysDeny => {
                table.grant_session_deny(&grant_key);
                table.grant_session_deny(tool_name);
                ToolPermissionDecision::Deny {
                    reason: "Denied by user for the rest of session".to_string(),
                }
            }
            UserPermissionResponse::Cancel => ToolPermissionDecision::Deny {
                reason: "Execution cancelled by user".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chained_command_denial() {
        assert!(check_hardcoded_security_rules("echo 'starting build' && rm -rf /").is_err());
        assert!(check_hardcoded_security_rules("cargo test || rm -rf ~").is_err());
        assert!(check_hardcoded_security_rules("ls -la ; rm -rf .").is_err());
        assert!(check_hardcoded_security_rules("cargo check && cargo test --workspace").is_ok());
    }

    #[test]
    fn test_engine_chained_terminal_evaluation() {
        let engine = PermissionEngine::new();

        // Chained destructive command is denied
        let dec = engine.evaluate("terminal", "echo hello && rm -rf /");
        assert!(dec.is_denied());

        // Safe command requires confirmation by default for terminal
        let dec_safe = engine.evaluate("terminal", "cargo build");
        assert!(dec_safe.is_confirmation_required());

        // User allows once
        let user_dec = engine.handle_user_response("terminal", "cargo build", UserPermissionResponse::AllowOnce);
        assert!(user_dec.is_allowed());

        // User always allows
        engine.handle_user_response("terminal", "cargo check", UserPermissionResponse::AlwaysAllow);
        let dec_check = engine.evaluate("terminal", "cargo check");
        assert!(dec_check.is_allowed());
    }
}
