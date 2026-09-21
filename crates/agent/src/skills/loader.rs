use std::path::Path;

use super::{Skill, SkillError, SkillParser};

pub struct SkillLoader;

impl SkillLoader {
    pub fn load(path: impl AsRef<Path>) -> Result<Skill, SkillError> {
        SkillParser::parse_file(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_skill_file() {
        let error = SkillLoader::load("missing/SKILL.md").unwrap_err();
        assert!(matches!(error, SkillError::Io(_)));
    }
}
