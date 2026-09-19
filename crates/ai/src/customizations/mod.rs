pub mod skills;
pub mod rules;

use std::path::Path;
pub use skills::{SkillMetadata, discover_skills, render_skills_summary};
pub use rules::{ProjectRule, discover_rules, render_rules_summary};

#[derive(Debug, Clone, Default)]
pub struct CustomizationContext {
    pub skills: Vec<SkillMetadata>,
    pub rules: Vec<ProjectRule>,
}

impl CustomizationContext {
    pub fn discover(workspace_root: &Path) -> Self {
        Self {
            skills: discover_skills(workspace_root),
            rules: discover_rules(workspace_root),
        }
    }

    pub fn render_for_prompt(&self) -> String {
        let mut out = String::new();
        let rules_str = render_rules_summary(&self.rules);
        if !rules_str.is_empty() {
            out.push_str(&rules_str);
            out.push('\n');
        }
        let skills_str = render_skills_summary(&self.skills);
        if !skills_str.is_empty() {
            out.push_str(&skills_str);
            out.push('\n');
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_customization_context_empty() {
        let tmp = std::env::temp_dir().join(format!("zenthree_cust_empty_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let _ = std::fs::create_dir_all(&tmp);

        let ctx = CustomizationContext::discover(&tmp);
        assert!(ctx.skills.is_empty());
        assert!(ctx.rules.is_empty());
        assert!(ctx.render_for_prompt().is_empty());

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_customization_context_with_skills_and_rules() {
        let tmp = std::env::temp_dir().join(format!("zenthree_cust_full_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let skill_dir = tmp.join(".zenthree").join("skills").join("deploy");
        let rules_dir = tmp.join(".zenthree").join("rules");
        let _ = std::fs::create_dir_all(&skill_dir);
        let _ = std::fs::create_dir_all(&rules_dir);

        let _ = std::fs::write(skill_dir.join("SKILL.md"), "---\nname: deploy\ndescription: Deployment playbook\n---\nRun docker-compose.");
        let _ = std::fs::write(rules_dir.join("style.md"), "Write clean code.");

        let ctx = CustomizationContext::discover(&tmp);
        assert_eq!(ctx.skills.len(), 1);
        assert_eq!(ctx.rules.len(), 1);

        let rendered = ctx.render_for_prompt();
        assert!(rendered.contains("### Project Rules & Constraints:"));
        assert!(rendered.contains("Write clean code."));
        assert!(rendered.contains("### Available Skills:"));
        assert!(rendered.contains("`deploy`"));
        assert!(rendered.contains("Deployment playbook"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
