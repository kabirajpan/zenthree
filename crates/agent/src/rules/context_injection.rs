use super::scanner::RuleDocument;

/// Maximum aggregate byte size for all rules in the prompt (40 KB).
pub const MAX_AGGREGATE_RULES_BYTES: usize = 40 * 1024;

/// Generates the `<workspace_rules>` prompt block from discovered rule documents.
/// Returns `None` if there are no active rules to inject.
pub fn render_rules_prompt(rules: &[RuleDocument]) -> Option<String> {
    render_rules_prompt_with_budget(rules, MAX_AGGREGATE_RULES_BYTES)
}

/// Generates the prompt block with a configurable byte budget.
pub fn render_rules_prompt_with_budget(rules: &[RuleDocument], budget_bytes: usize) -> Option<String> {
    if rules.is_empty() {
        return None;
    }

    let mut out = String::new();
    out.push_str("<workspace_rules>\n");
    out.push_str("The following user and workspace-specific rules and instructions must be adhered to at all times:\n\n");

    let mut current_bytes = out.len();
    let mut included_any = false;

    for rule in rules {
        let rule_header = format!("<rule source=\"{}\">\n", rule.name);
        let rule_footer = "\n</rule>\n\n";
        let overhead = rule_header.len() + rule_footer.len();

        if current_bytes + overhead >= budget_bytes {
            out.push_str("<!-- [Remaining rules omitted: aggregate budget reached] -->\n");
            break;
        }

        let remaining_budget = budget_bytes - (current_bytes + overhead);
        let content_to_add = if rule.content.len() > remaining_budget {
            let truncated = &rule.content[..remaining_budget];
            format!("{}\n<!-- [Truncated due to aggregate budget] -->", truncated)
        } else {
            rule.content.clone()
        };

        out.push_str(&rule_header);
        out.push_str(&content_to_add);
        out.push_str(rule_footer);

        current_bytes += overhead + content_to_add.len();
        included_any = true;
    }

    if !included_any {
        return None;
    }

    // Strip trailing newlines and close tag
    let trimmed = out.trim_end();
    let mut final_out = trimmed.to_string();
    final_out.push_str("\n</workspace_rules>");

    Some(final_out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rules::scanner::RuleScope;
    use std::path::PathBuf;

    #[test]
    fn test_render_rules_empty() {
        assert!(render_rules_prompt(&[]).is_none());
    }

    #[test]
    fn test_render_rules_standard() {
        let rules = vec![
            RuleDocument {
                name: "AGENTS.md".to_string(),
                source_path: PathBuf::from("/ws/AGENTS.md"),
                scope: RuleScope::Workspace,
                content: "# Guidelines\nDo not mutate cargo.lock directly.".to_string(),
                truncated: false,
            },
            RuleDocument {
                name: ".cursorrules".to_string(),
                source_path: PathBuf::from("/ws/.cursorrules"),
                scope: RuleScope::Workspace,
                content: "Keep functions small.".to_string(),
                truncated: false,
            },
        ];

        let prompt = render_rules_prompt(&rules).expect("Should render prompt");
        assert!(prompt.starts_with("<workspace_rules>"));
        assert!(prompt.contains("<rule source=\"AGENTS.md\">"));
        assert!(prompt.contains("Do not mutate cargo.lock directly."));
        assert!(prompt.contains("<rule source=\".cursorrules\">"));
        assert!(prompt.contains("Keep functions small."));
        assert!(prompt.ends_with("</workspace_rules>"));
    }

    #[test]
    fn test_render_rules_budget_cutoff() {
        let rules = vec![
            RuleDocument {
                name: "AGENTS.md".to_string(),
                source_path: PathBuf::from("/ws/AGENTS.md"),
                scope: RuleScope::Workspace,
                content: "Small rule.".to_string(),
                truncated: false,
            },
            RuleDocument {
                name: "SECOND.md".to_string(),
                source_path: PathBuf::from("/ws/SECOND.md"),
                scope: RuleScope::Workspace,
                content: "A".repeat(1000),
                truncated: false,
            },
        ];

        // Small budget that only fits AGENTS.md
        let prompt = render_rules_prompt_with_budget(&rules, 200).expect("Should render prompt");
        assert!(prompt.contains("<rule source=\"AGENTS.md\">"));
        assert!(prompt.contains("aggregate budget reached") || prompt.contains("Truncated due to aggregate budget"));
    }
}
