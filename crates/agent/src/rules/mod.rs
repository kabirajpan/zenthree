pub mod scanner;
pub mod context_injection;

pub use scanner::{RuleDocument, RuleScope, RuleScanner, MAX_RULE_FILE_BYTES, KNOWN_RULE_FILENAMES};
pub use context_injection::{render_rules_prompt, render_rules_prompt_with_budget, MAX_AGGREGATE_RULES_BYTES};

use std::path::{Path, PathBuf};

/// Coordinates scanning, caching, and prompt injection for workspace and user rules.
#[derive(Debug, Clone)]
pub struct RulesRegistry {
    workspace_root: PathBuf,
    global_rules_dir: Option<PathBuf>,
}

impl RulesRegistry {
    pub fn new(workspace_root: impl AsRef<Path>, global_rules_dir: Option<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.as_ref().to_path_buf(),
            global_rules_dir,
        }
    }

    /// Discovers all active rule documents across workspace and global scopes.
    pub fn discover_rules(&self) -> Vec<RuleDocument> {
        let scanner = RuleScanner::new();
        let mut rules = Vec::new();

        // 1. Workspace rules (higher priority for workspace-specific context)
        rules.extend(scanner.scan_workspace(&self.workspace_root));

        // 2. Global rules (if configured)
        if let Some(ref global_dir) = self.global_rules_dir {
            rules.extend(scanner.scan_global(global_dir));
        }

        rules
    }

    /// Renders the prompt injection block for active rules.
    pub fn generate_rules_prompt(&self) -> Option<String> {
        let rules = self.discover_rules();
        render_rules_prompt(&rules)
    }

    /// Renders the prompt injection block with an explicit byte budget limit.
    pub fn generate_rules_prompt_with_budget(&self, budget_bytes: usize) -> Option<String> {
        let rules = self.discover_rules();
        render_rules_prompt_with_budget(&rules, budget_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_rules_registry_workflow() {
        let ws_dir = tempdir().unwrap();
        let global_dir = tempdir().unwrap();

        fs::write(ws_dir.path().join("AGENTS.md"), "Workspace agents rule.").unwrap();
        fs::write(global_dir.path().join("global.rules"), "Global dev rule.").unwrap();

        let registry = RulesRegistry::new(ws_dir.path(), Some(global_dir.path().to_path_buf()));
        let rules = registry.discover_rules();

        assert_eq!(rules.len(), 2);
        let prompt = registry.generate_rules_prompt().expect("Prompt must be rendered");
        assert!(prompt.contains("Workspace agents rule."));
        assert!(prompt.contains("Global dev rule."));
    }
}
