use super::rule::{Rule, RuleScope};

/// Maximum characters allowed per individual rule file before truncating.
pub const MAX_RULE_CHARS: usize = 8000;

/// Maximum aggregate characters allowed across all rules combined in system prompt.
pub const MAX_TOTAL_RULES_CHARS: usize = 24000;

/// Formats discovered rules into a structured `<custom_instructions>` XML block for the system prompt.
pub fn format_rules_prompt(rules: &[Rule]) -> String {
    if rules.is_empty() {
        return String::new();
    }

    let mut output = String::new();
    output.push_str("<custom_instructions>\n");
    output.push_str("# Project Rules & Custom Instructions\n");
    output.push_str("The following project-specific guidelines, behavioral constraints, and architectural standards are active for this workspace. You MUST adhere to these instructions:\n\n");

    let mut total_chars = 0;

    for rule in rules {
        if total_chars >= MAX_TOTAL_RULES_CHARS {
            output.push_str("\n> [!NOTE]\n> Remaining project rules omitted to conserve context window.\n");
            break;
        }

        let scope_label = match rule.scope {
            RuleScope::WorkspaceRoot => "Workspace Root",
            RuleScope::WorkspaceModular => "Workspace Modular Rule",
            RuleScope::Global => "Global User Configuration",
        };

        output.push_str(&format!("## {} ({})\n", rule.name, scope_label));
        output.push_str(&format!("*Source: `{}`*\n\n", rule.path.display()));

        let mut content = rule.content.as_str();
        let mut truncated = false;
        if content.len() > MAX_RULE_CHARS {
            content = &content[..MAX_RULE_CHARS];
            truncated = true;
        }

        output.push_str(content);
        output.push('\n');

        if truncated {
            output.push_str("\n> [!WARNING]\n> Content of this rule was truncated to 8,000 characters.\n");
        }
        output.push_str("\n---\n\n");

        total_chars += content.len();
    }

    output.push_str("</custom_instructions>\n");
    output
}
