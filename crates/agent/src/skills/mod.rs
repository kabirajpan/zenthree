pub mod spec;
pub mod discovery;
pub mod catalog;

pub use spec::{Skill, SkillMetadata, SkillScope, MAX_SKILL_FILE_SIZE, SKILL_FILE_NAME};
pub use discovery::{SkillDiscovery, BUILTIN_CREATE_SKILL_MD};
pub use catalog::{SkillCatalog, SkillSummary, MAX_CATALOG_DESCRIPTIONS_SIZE};

use std::collections::HashMap;
use std::path::Path;

/// Unified skills registry managing discovery, caching, and catalog prompt formatting.
#[derive(Debug, Clone, Default)]
pub struct SkillsRegistry {
    skills: HashMap<String, Skill>,
}

impl SkillsRegistry {
    /// Discovers all available skills for the specified workspace roots and user global directory.
    pub fn new(workspace_roots: &[impl AsRef<Path>]) -> Self {
        let skills = SkillDiscovery::discover_skills(workspace_roots, true);
        Self { skills }
    }

    /// Discovers skills allowing a custom global directory (for testing and isolation).
    pub fn with_custom_global(
        workspace_roots: &[impl AsRef<Path>],
        custom_global: Option<&Path>,
    ) -> Self {
        let skills = SkillDiscovery::discover_skills_with_global(workspace_roots, custom_global, true);
        Self { skills }
    }

    /// Retrieves a skill by name.
    pub fn get_skill(&self, name: &str) -> Option<&Skill> {
        self.skills.get(name)
    }

    /// Returns the full list of discovered skills.
    pub fn skills(&self) -> impl Iterator<Item = &Skill> {
        self.skills.values()
    }

    /// Produces the progressive disclosure catalog for system prompt injection.
    pub fn catalog(&self) -> SkillCatalog {
        SkillCatalog::from_skills(self.skills.values())
    }

    /// Formats the `<available_skills>` XML block for the system prompt.
    pub fn format_prompt_block(&self) -> String {
        self.catalog().format_prompt_block()
    }

    pub fn len(&self) -> usize {
        self.skills.len()
    }

    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_skills_registry_initialization() {
        let empty_roots: Vec<PathBuf> = Vec::new();
        let registry = SkillsRegistry::new(&empty_roots);

        // At minimum, the built-in `create-skill` is available
        assert!(!registry.is_empty());
        assert!(registry.get_skill("create-skill").is_some());

        let prompt_block = registry.format_prompt_block();
        assert!(prompt_block.contains("<available_skills>"));
        assert!(prompt_block.contains("<name>create-skill</name>"));
    }
}
