use super::spec::{Skill, SkillScope, SKILL_FILE_NAME};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Built-in reference skill embedded directly into the binary.
pub const BUILTIN_CREATE_SKILL_MD: &str = r#"---
name: create-skill
description: Helps you create new agent skills conforming to agentskills.io. Use this to package reusable agent instructions.
---

# Creating an Agent Skill

Use this skill when you need to package specialized instructions, workflows, or rules into a reusable skill.

## SKILL.md Structure
Every skill must have a directory matching the skill name, containing a `SKILL.md` file with YAML frontmatter:

```markdown
---
name: my-skill-name
description: Clear, specific description of when the model should invoke this skill.
---

# Skill Title
Specific actionable instructions for the agent go here.
```

## Guidelines
1. **Name**: Lowercase letters, numbers, and single hyphens only (e.g. `rust-patterns`).
2. **Description**: Concise and actionable (what it does and when to call it).
3. **Directory**: Place under `.agents/skills/<name>/SKILL.md` for project scope or `~/.agents/skills/<name>/SKILL.md` for global personal scope.
"#;

/// Discovery engine that scans and indexes agent skills across all scopes.
#[derive(Debug, Clone, Default)]
pub struct SkillDiscovery;

impl SkillDiscovery {
    /// Discovers all available skills across Builtin, Global, and Workspace tiers.
    /// Workspace skills shadow Global skills, which shadow Built-in skills.
    pub fn discover_skills(
        workspace_roots: &[impl AsRef<Path>],
        include_global: bool,
    ) -> HashMap<String, Skill> {
        let global_dir = if include_global {
            dirs::home_dir().map(|h| h.join(".agents").join("skills"))
        } else {
            None
        };
        Self::discover_skills_with_global(workspace_roots, global_dir.as_deref(), true)
    }

    /// Discovers skills allowing a custom global directory (ideal for testing and isolation).
    pub fn discover_skills_with_global(
        workspace_roots: &[impl AsRef<Path>],
        custom_global: Option<&Path>,
        include_builtins: bool,
    ) -> HashMap<String, Skill> {
        let mut registry = HashMap::new();

        // 1. Built-in tier (lowest priority)
        if include_builtins {
            if let Ok(builtin) = Skill::parse(
                BUILTIN_CREATE_SKILL_MD,
                Path::new("builtin/create-skill"),
                SkillScope::Builtin,
            ) {
                registry.insert(builtin.name().to_string(), builtin);
            }
        }

        // 2. Global tier
        if let Some(global_dir) = custom_global {
            Self::scan_directory(global_dir, SkillScope::Global, &mut registry);
        }

        // 3. Workspace tier (<root>/.agents/skills/) - shadows previous tiers
        for root in workspace_roots {
            let ws_skills_dir = root.as_ref().join(".agents").join("skills");
            Self::scan_directory(&ws_skills_dir, SkillScope::Workspace, &mut registry);
        }

        registry
    }

    /// Scans a directory for subdirectories containing SKILL.md.
    fn scan_directory(
        base_dir: &Path,
        scope: SkillScope,
        registry: &mut HashMap<String, Skill>,
    ) {
        if !base_dir.is_dir() {
            return;
        }

        let entries = match fs::read_dir(base_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let skill_file = path.join(SKILL_FILE_NAME);
                if skill_file.is_file() {
                    if let Ok(content) = fs::read_to_string(&skill_file) {
                        if let Ok(skill) = Skill::parse(&content, &path, scope.clone()) {
                            // Insert or shadow
                            registry.insert(skill.name().to_string(), skill);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_builtin_create_skill_discovered() {
        let empty_roots: Vec<PathBuf> = Vec::new();
        let skills = SkillDiscovery::discover_skills(&empty_roots, false);

        assert!(skills.contains_key("create-skill"));
        let builtin = &skills["create-skill"];
        assert_eq!(builtin.scope, SkillScope::Builtin);
        assert!(builtin.description().contains("create new agent skills"));
    }

    #[test]
    fn test_workspace_shadows_builtin() {
        let temp_dir = std::env::temp_dir().join("zenthree_test_skill_shadow");
        let skill_dir = temp_dir.join(".agents").join("skills").join("create-skill");
        let _ = fs::create_dir_all(&skill_dir);

        let custom_skill = r#"---
name: create-skill
description: Custom workspace-specific skill creator.
---
# Custom Workspace Override
"#;
        fs::write(skill_dir.join("SKILL.md"), custom_skill).unwrap();

        let skills = SkillDiscovery::discover_skills(&[temp_dir.clone()], false);
        assert_eq!(skills["create-skill"].scope, SkillScope::Workspace);
        assert_eq!(skills["create-skill"].description(), "Custom workspace-specific skill creator.");

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
