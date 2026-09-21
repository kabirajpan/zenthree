use std::collections::BTreeMap;

use super::{Skill, SkillError};

#[derive(Debug, Default, Clone)]
pub struct SkillRegistry {
    skills: BTreeMap<String, Skill>,
}

impl SkillRegistry {
    pub fn register(&mut self, skill: Skill) -> Result<(), SkillError> {
        if self.skills.contains_key(&skill.name) {
            return Err(SkillError::DuplicateName(skill.name));
        }
        self.skills.insert(skill.name.clone(), skill);
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    pub fn list(&self) -> impl Iterator<Item = &Skill> {
        self.skills.values()
    }

    pub fn contains(&self, name: &str) -> bool {
        self.skills.contains_key(name)
    }

    pub fn unregister(&mut self, name: &str) -> Option<Skill> {
        self.skills.remove(name)
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(name: &str) -> Skill {
        Skill {
            name: name.into(),
            description: "description".into(),
            version: None,
            tags: Vec::new(),
            path: name.into(),
            instructions: "instructions".into(),
        }
    }

    #[test]
    fn manages_skills_without_overwriting_duplicates() {
        let mut registry = SkillRegistry::default();
        registry.register(skill("debugging")).unwrap();
        assert!(registry.contains("debugging"));
        assert_eq!(registry.get("debugging").unwrap().name, "debugging");
        assert_eq!(registry.list().count(), 1);
        assert!(matches!(
            registry.register(skill("debugging")),
            Err(SkillError::DuplicateName(_))
        ));
        assert!(registry.unregister("debugging").is_some());
        assert!(!registry.contains("debugging"));
    }
}
