use super::spec::Skill;
use serde::{Deserialize, Serialize};

/// Maximum total size for skill descriptions in system prompt (50KB, matching Zed standard).
pub const MAX_CATALOG_DESCRIPTIONS_SIZE: usize = 50 * 1024;

/// Lightweight summary of a skill included in the system prompt catalog.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillSummary {
    pub name: String,
    pub description: String,
    pub location: String,
}

/// The progressive disclosure catalog that formats available skills for LLM system instructions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SkillCatalog {
    pub skills: Vec<SkillSummary>,
}

impl SkillCatalog {
    /// Builds a catalog from a collection of skills, respecting the 50KB prompt budget.
    /// Excludes skills marked with `disable_model_invocation: true`.
    pub fn from_skills<'a>(skills: impl IntoIterator<Item = &'a Skill>) -> Self {
        let mut summaries = Vec::new();
        let mut current_bytes = 0;

        for skill in skills {
            if !skill.is_model_invocable() {
                continue;
            }

            let desc_len = skill.description().len();
            if current_bytes + desc_len > MAX_CATALOG_DESCRIPTIONS_SIZE {
                break;
            }

            let location = if skill.scope == super::spec::SkillScope::Builtin {
                format!("builtin:{}", skill.name())
            } else {
                skill.file_path.to_string_lossy().to_string()
            };

            summaries.push(SkillSummary {
                name: skill.name().to_string(),
                description: skill.description().to_string(),
                location,
            });

            current_bytes += desc_len;
        }

        // Sort catalog deterministically by name
        summaries.sort_by(|a, b| a.name.cmp(&b.name));

        Self { skills: summaries }
    }

    /// Formats the catalog into the exact XML block specified by Zed and modern agents for system prompt injection.
    pub fn format_prompt_block(&self) -> String {
        if self.skills.is_empty() {
            return String::new();
        }

        let mut out = String::new();
        out.push_str("## Agent Skills\n\n");
        out.push_str("You have access to the following Skills - modular capabilities that provide specialized instructions for specific tasks. When a user's request matches a Skill's description, use the `skill` tool or `read_file` with the skill's location to retrieve the full instructions before taking action.\n\n");
        out.push_str("<available_skills>\n");

        for skill in &self.skills {
            out.push_str("  <skill>\n");
            out.push_str(&format!("    <name>{}</name>\n", escape_xml(&skill.name)));
            out.push_str(&format!("    <description>{}</description>\n", escape_xml(&skill.description)));
            out.push_str(&format!("    <location>{}</location>\n", &skill.location));
            out.push_str("  </skill>\n");
        }

        out.push_str("</available_skills>\n");
        out
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }
}

/// Helper to escape XML special characters in name and description.
fn escape_xml(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::spec::{SkillScope, Skill};

    #[test]
    fn test_catalog_prompt_formatting() {
        let skill1 = Skill::parse(
            "---\nname: rust-best-practices\ndescription: Idiomatic Rust.\n---\nbody",
            "/path/to/rust-best-practices",
            SkillScope::Workspace,
        ).unwrap();

        let catalog = SkillCatalog::from_skills([&skill1]);
        let prompt_block = catalog.format_prompt_block();

        assert!(prompt_block.contains("## Agent Skills"));
        assert!(prompt_block.contains("<available_skills>"));
        assert!(prompt_block.contains("<name>rust-best-practices</name>"));
        assert!(prompt_block.contains("<description>Idiomatic Rust.</description>"));
        assert!(prompt_block.contains("</available_skills>"));
    }

    #[test]
    fn test_excludes_disabled_invocation() {
        let skill = Skill::parse(
            "---\nname: hidden-skill\ndescription: Hidden from model.\ndisable-model-invocation: true\n---\nbody",
            "/path",
            SkillScope::Global,
        ).unwrap();

        let catalog = SkillCatalog::from_skills([&skill]);
        assert!(catalog.is_empty());
        assert!(catalog.format_prompt_block().is_empty());
    }
}
