pub mod discovery;
pub mod formatter;
pub mod rule;

pub use discovery::discover_rules;
pub use formatter::{format_rules_prompt, MAX_RULE_CHARS, MAX_TOTAL_RULES_CHARS};
pub use rule::{Rule, RuleScope};

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_rules_discovery_and_precedence() {
        let temp_dir = std::env::temp_dir().join("zene_test_rules_discovery");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // 1. Create Workspace Root AGENTS.md
        let agents_md_path = temp_dir.join("AGENTS.md");
        fs::write(&agents_md_path, "# Team AGENTS Guidelines\nAlways write tests.").unwrap();

        // 2. Create Workspace Modular Rule in .agents/rules/
        let modular_dir = temp_dir.join(".agents").join("rules");
        fs::create_dir_all(&modular_dir).unwrap();
        let modular_rule_path = modular_dir.join("code_style.md");
        fs::write(&modular_rule_path, "Use rustfmt with 100 max width.").unwrap();

        // 3. Discover rules
        let rules = discover_rules(&temp_dir);
        assert_eq!(rules.len(), 2);

        // Highest precedence (WorkspaceRoot) must be first
        assert_eq!(rules[0].name, "AGENTS.md");
        assert_eq!(rules[0].scope, RuleScope::WorkspaceRoot);
        assert!(rules[0].content.contains("Always write tests"));

        assert_eq!(rules[1].name, "code_style.md");
        assert_eq!(rules[1].scope, RuleScope::WorkspaceModular);
        assert!(rules[1].content.contains("Use rustfmt"));

        // 4. Test Formatting
        let formatted = format_rules_prompt(&rules);
        assert!(formatted.contains("<custom_instructions>"));
        assert!(formatted.contains("AGENTS.md (Workspace Root)"));
        assert!(formatted.contains("code_style.md (Workspace Modular Rule)"));
        assert!(formatted.contains("</custom_instructions>"));

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_empty_workspace_rules_formatting() {
        let empty_rules: Vec<Rule> = Vec::new();
        let prompt = format_rules_prompt(&empty_rules);
        assert!(prompt.is_empty());
    }
}
