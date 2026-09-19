use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillMetadata {
    pub name: String,
    pub description: String,
    pub path: String,
    pub absolute_path: PathBuf,
}

/// Discovers skills in the workspace and user config directory.
/// Follows the open standard: looks for `<skill_folder>/SKILL.md`.
pub fn discover_skills(workspace_root: &Path) -> Vec<SkillMetadata> {
    let mut skills = Vec::new();

    let candidate_dirs = vec![
        workspace_root.join(".zenthree").join("skills"),
        workspace_root.join(".agents").join("skills"),
        workspace_root.join("skills"),
    ];

    for base_dir in candidate_dirs {
        if !base_dir.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&base_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let skill_md = path.join("SKILL.md");
                    let skill_md_alt = path.join("skill.md");
                    let target_md = if skill_md.is_file() {
                        Some(skill_md)
                    } else if skill_md_alt.is_file() {
                        Some(skill_md_alt)
                    } else {
                        None
                    };

                    if let Some(target) = target_md {
                        let dir_name = path.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown")
                            .to_string();

                        let rel_path = target.strip_prefix(workspace_root)
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| target.to_string_lossy().to_string());

                        let (name, description) = parse_skill_metadata(&target, &dir_name);

                        // Prevent duplicate skill names
                        if !skills.iter().any(|s: &SkillMetadata| s.name == name) {
                            skills.push(SkillMetadata {
                                name,
                                description,
                                path: rel_path,
                                absolute_path: target,
                            });
                        }
                    }
                }
            }
        }
    }

    skills
}

/// Parses YAML frontmatter or falls back to first heading / line in SKILL.md
fn parse_skill_metadata(skill_md_path: &Path, default_name: &str) -> (String, String) {
    let content = match std::fs::read_to_string(skill_md_path) {
        Ok(c) => c,
        Err(_) => return (default_name.to_string(), format!("Skill for {}", default_name)),
    };

    let trimmed = content.trim();
    if trimmed.starts_with("---") {
        let after_first = &trimmed[3..];
        if let Some(end_idx) = after_first.find("---") {
            let frontmatter = &after_first[..end_idx];
            let mut name = default_name.to_string();
            let mut desc = String::new();

            for line in frontmatter.lines() {
                let l = line.trim();
                if let Some(rest) = l.strip_prefix("name:") {
                    name = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                } else if let Some(rest) = l.strip_prefix("description:") {
                    desc = rest.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }

            if desc.is_empty() {
                desc = format!("Skill for {}", name);
            }
            return (name, desc);
        }
    }

    // Fallback: search for first markdown heading
    for line in content.lines() {
        let l = line.trim();
        if let Some(heading) = l.strip_prefix("# ") {
            return (default_name.to_string(), heading.trim().to_string());
        }
    }

    (default_name.to_string(), format!("Workflow instructions for {}", default_name))
}

/// Renders a concise progressive-disclosure skill index for the LLM context.
pub fn render_skills_summary(skills: &[SkillMetadata]) -> String {
    if skills.is_empty() {
        return String::new();
    }

    let mut out = String::from("### Available Skills:\n");
    out.push_str("If a task matches an available skill, read its `SKILL.md` using `read_file` before executing.\n\n");
    for s in skills {
        out.push_str(&format!("- `{}` (`{}`): {}\n", s.name, s.path, s.description));
    }
    out.push('\n');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_and_render_skills() {
        let tmp = std::env::temp_dir().join(format!("zenthree_skills_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let skill_dir = tmp.join(".zenthree").join("skills").join("db-migration");
        let _ = std::fs::create_dir_all(&skill_dir);

        let skill_content = "---\nname: db-migration\ndescription: Database schema migration and rollback playbook\n---\n# DB Migration\nInstructions...";
        let _ = std::fs::write(skill_dir.join("SKILL.md"), skill_content);

        let discovered = discover_skills(&tmp);
        assert_eq!(discovered.len(), 1);
        assert_eq!(discovered[0].name, "db-migration");
        assert_eq!(discovered[0].description, "Database schema migration and rollback playbook");
        assert_eq!(discovered[0].path, ".zenthree/skills/db-migration/SKILL.md");

        let rendered = render_skills_summary(&discovered);
        assert!(rendered.contains("### Available Skills:"));
        assert!(rendered.contains("`db-migration`"));
        assert!(rendered.contains("Database schema migration and rollback playbook"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
