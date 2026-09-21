use std::path::{Path, PathBuf};

use super::{SkillError, SkillLoader, SkillRegistry};

#[derive(Debug, Default)]
pub struct DiscoveryReport {
    pub registry: SkillRegistry,
    pub errors: Vec<(PathBuf, SkillError)>,
}

pub fn discover(workspace: impl AsRef<Path>) -> Result<DiscoveryReport, SkillError> {
    let skills_dir = workspace
        .as_ref()
        .join(".zenthree")
        .join("agent")
        .join("zene")
        .join("skills");
    if !skills_dir.exists() {
        return Ok(DiscoveryReport::default());
    }

    let mut report = DiscoveryReport::default();
    for entry in std::fs::read_dir(&skills_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let skill_path = path.join("SKILL.md");
        if !skill_path.is_file() {
            report.errors.push((
                skill_path,
                SkillError::MissingSkill(path.display().to_string()),
            ));
            continue;
        }

        match SkillLoader::load(&skill_path) {
            Ok(skill) => {
                if let Err(error) = report.registry.register(skill) {
                    report.errors.push((skill_path, error));
                }
            }
            Err(error) => report.errors.push((skill_path, error)),
        }
    }

    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn workspace() -> PathBuf {
        let id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("agent-skills-{}", id));
        fs::create_dir_all(
            path.join(".zenthree")
                .join("agent")
                .join("zene")
                .join("skills"),
        )
        .unwrap();
        path
    }

    fn write_skill(root: &Path, name: &str, content: &str) {
        let dir = root
            .join(".zenthree")
            .join("agent")
            .join("zene")
            .join("skills")
            .join(name);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("SKILL.md"), content).unwrap();
    }

    #[test]
    fn discovers_valid_skills_and_continues_after_invalid_one() {
        let root = workspace();
        write_skill(&root, "debugging", "---\nname: debugging\ndescription: Debug.\n---\n# Debugging");
        write_skill(&root, "broken", "---\nname: broken\n---\n# Broken");
        let report = discover(&root).unwrap();
        assert!(report.registry.contains("debugging"));
        assert_eq!(report.errors.len(), 1);
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn missing_directory_is_empty_success() {
        let root = std::env::temp_dir().join("agent-skills-missing");
        let report = discover(root).unwrap();
        assert_eq!(report.registry.len(), 0);
    }

    #[test]
    fn reports_directory_without_skill_file() {
        let root = workspace();
        fs::create_dir_all(
            root.join(".zenthree")
                .join("agent")
                .join("zene")
                .join("skills")
                .join("empty"),
        )
        .unwrap();
        let report = discover(&root).unwrap();
        assert_eq!(report.errors.len(), 1);
        let _ = fs::remove_dir_all(root);
    }
}
