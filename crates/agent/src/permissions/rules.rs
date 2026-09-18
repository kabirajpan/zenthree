use super::decision::ToolPermissionDecision;
use super::hardcoded_denials::HardcodedSecurityRules;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Default permission modes for tools when no specific regex patterns match.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermissionMode {
    Allow,
    Confirm,
    Deny,
}

/// A pre-compiled regular expression rule.
#[derive(Debug, Clone)]
pub struct CompiledRule {
    pub pattern: String,
    pub case_sensitive: bool,
    pub regex: Regex,
}

impl CompiledRule {
    pub fn new(pattern: &str, case_sensitive: bool) -> Result<Self, regex::Error> {
        let regex = regex::RegexBuilder::new(pattern)
            .case_insensitive(!case_sensitive)
            .build()?;
        Ok(Self {
            pattern: pattern.to_string(),
            case_sensitive,
            regex,
        })
    }

    pub fn is_match(&self, input: &str) -> bool {
        self.regex.is_match(input)
    }
}

/// Configured rules for a specific tool.
#[derive(Debug, Clone, Default)]
pub struct ToolRuleSet {
    pub tool_name: String,
    pub default: Option<ToolPermissionMode>,
    pub always_allow: Vec<CompiledRule>,
    pub always_confirm: Vec<CompiledRule>,
    pub always_deny: Vec<CompiledRule>,
}

impl ToolRuleSet {
    pub fn new(tool_name: &str) -> Self {
        Self {
            tool_name: tool_name.to_string(),
            default: None,
            always_allow: Vec::new(),
            always_confirm: Vec::new(),
            always_deny: Vec::new(),
        }
    }
}

/// Permission policy table that evaluates all incoming tool requests against rules and session grants.
#[derive(Debug, Clone)]
pub struct PermissionPolicyTable {
    pub global_default: ToolPermissionMode,
    pub tool_rules: HashMap<String, ToolRuleSet>,
    pub session_grants: HashSet<String>,
    pub session_denials: HashSet<String>,
}

impl Default for PermissionPolicyTable {
    fn default() -> Self {
        let mut table = Self {
            global_default: ToolPermissionMode::Confirm,
            tool_rules: HashMap::new(),
            session_grants: HashSet::new(),
            session_denials: HashSet::new(),
        };

        // Standard defaults: read tools allow by default, mutating tools confirm by default
        table.set_tool_default("read_file", ToolPermissionMode::Allow);
        table.set_tool_default("list_directory", ToolPermissionMode::Allow);
        table.set_tool_default("grep", ToolPermissionMode::Allow);
        table.set_tool_default("find_path", ToolPermissionMode::Allow);
        table.set_tool_default("diagnostics", ToolPermissionMode::Allow);

        table.set_tool_default("write_file", ToolPermissionMode::Confirm);
        table.set_tool_default("edit_file", ToolPermissionMode::Confirm);
        table.set_tool_default("delete_path", ToolPermissionMode::Confirm);
        table.set_tool_default("terminal", ToolPermissionMode::Confirm);

        table
    }
}

impl PermissionPolicyTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_tool_default(&mut self, tool: &str, mode: ToolPermissionMode) {
        self.tool_rules
            .entry(tool.to_string())
            .or_insert_with(|| ToolRuleSet::new(tool))
            .default = Some(mode);
    }

    pub fn add_allow_pattern(&mut self, tool: &str, pattern: &str) -> Result<(), regex::Error> {
        let rule = CompiledRule::new(pattern, false)?;
        self.tool_rules
            .entry(tool.to_string())
            .or_insert_with(|| ToolRuleSet::new(tool))
            .always_allow
            .push(rule);
        Ok(())
    }

    pub fn add_deny_pattern(&mut self, tool: &str, pattern: &str) -> Result<(), regex::Error> {
        let rule = CompiledRule::new(pattern, false)?;
        self.tool_rules
            .entry(tool.to_string())
            .or_insert_with(|| ToolRuleSet::new(tool))
            .always_deny
            .push(rule);
        Ok(())
    }

    pub fn add_confirm_pattern(&mut self, tool: &str, pattern: &str) -> Result<(), regex::Error> {
        let rule = CompiledRule::new(pattern, false)?;
        self.tool_rules
            .entry(tool.to_string())
            .or_insert_with(|| ToolRuleSet::new(tool))
            .always_confirm
            .push(rule);
        Ok(())
    }

    pub fn grant_session_allow(&mut self, grant_key: &str) {
        self.session_grants.insert(grant_key.to_string());
    }

    pub fn grant_session_deny(&mut self, grant_key: &str) {
        self.session_denials.insert(grant_key.to_string());
    }

    /// Evaluates permission decision for an input using the strict priority order:
    /// 1. Hardcoded security denials
    /// 2. Session denials (AlwaysDeny in current session)
    /// 3. Configured always_deny rules
    /// 4. Session grants (AlwaysAllow in current session)
    /// 5. Configured always_confirm rules
    /// 6. Configured always_allow rules
    /// 7. Tool-specific default
    /// 8. Global default
    pub fn evaluate(&self, tool_name: &str, input: &str) -> ToolPermissionDecision {
        // 1. Hardcoded security rules check (unbypassable)
        if let Some(reason) = HardcodedSecurityRules::check_command(input) {
            return ToolPermissionDecision::Deny {
                reason: format!("Hardcoded security rule: {}", reason),
            };
        }

        let grant_key = format!("{}:{}", tool_name, input);

        // 2. Session denials (active AlwaysDeny)
        if self.session_denials.contains(&grant_key)
            || self.session_denials.contains(tool_name)
            || self.session_denials.contains(&format!("{}:", tool_name))
        {
            return ToolPermissionDecision::Deny {
                reason: "Denied by active session policy (AlwaysDeny)".to_string(),
            };
        }

        // 3. Rule matching for tool
        if let Some(rules) = self.tool_rules.get(tool_name) {
            // Check always_deny
            for rule in &rules.always_deny {
                if rule.is_match(input) {
                    return ToolPermissionDecision::Deny {
                        reason: format!("Matches deny rule pattern: '{}'", rule.pattern),
                    };
                }
            }

            // 4. Session grants (active AlwaysAllow)
            if self.session_grants.contains(&grant_key)
                || self.session_grants.contains(tool_name)
                || self.session_grants.contains(&format!("{}:", tool_name))
            {
                return ToolPermissionDecision::Allow;
            }

            // 5. Check always_confirm
            for rule in &rules.always_confirm {
                if rule.is_match(input) {
                    return ToolPermissionDecision::Confirm {
                        reason: format!("Matches confirm rule pattern: '{}'", rule.pattern),
                        suggested_action: format!("Allow execution of '{}'", input),
                    };
                }
            }

            // 6. Check always_allow
            for rule in &rules.always_allow {
                if rule.is_match(input) {
                    return ToolPermissionDecision::Allow;
                }
            }

            // 7. Tool default
            if let Some(tool_default) = rules.default {
                return match tool_default {
                    ToolPermissionMode::Allow => ToolPermissionDecision::Allow,
                    ToolPermissionMode::Confirm => ToolPermissionDecision::Confirm {
                        reason: format!("Tool '{}' requires confirmation by default", tool_name),
                        suggested_action: format!("Approve {} call", tool_name),
                    },
                    ToolPermissionMode::Deny => ToolPermissionDecision::Deny {
                        reason: format!("Tool '{}' is denied by default", tool_name),
                    },
                };
            }
        }

        // 8. Global default
        match self.global_default {
            ToolPermissionMode::Allow => ToolPermissionDecision::Allow,
            ToolPermissionMode::Confirm => ToolPermissionDecision::Confirm {
                reason: "Requires user confirmation by global policy".to_string(),
                suggested_action: format!("Approve {} call", tool_name),
            },
            ToolPermissionMode::Deny => ToolPermissionDecision::Deny {
                reason: "Denied by global default policy".to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_tools_allow_by_default() {
        let table = PermissionPolicyTable::new();
        let dec = table.evaluate("read_file", "src/main.rs");
        assert_eq!(dec, ToolPermissionDecision::Allow);
    }

    #[test]
    fn test_mutating_tools_confirm_by_default() {
        let table = PermissionPolicyTable::new();
        let dec = table.evaluate("terminal", "cargo build");
        assert!(dec.is_confirmation_required());
    }

    #[test]
    fn test_hardcoded_security_rules_override_everything() {
        let mut table = PermissionPolicyTable::new();
        // Even if user adds an allow pattern for rm
        table.add_allow_pattern("terminal", r"\brm\b").unwrap();

        // Destructive rm -rf / is STILL denied unconditionally!
        let dec = table.evaluate("terminal", "rm -rf /");
        assert!(dec.is_denied());
    }

    #[test]
    fn test_always_allow_pattern_works() {
        let mut table = PermissionPolicyTable::new();
        table.add_allow_pattern("terminal", r"^cargo\s+(test|check)\b").unwrap();

        let dec = table.evaluate("terminal", "cargo test --workspace");
        assert_eq!(dec, ToolPermissionDecision::Allow);
    }
}
