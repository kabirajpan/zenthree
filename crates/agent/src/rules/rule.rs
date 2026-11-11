use std::path::PathBuf;

/// The scope/source hierarchy level of a discovered rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuleScope {
    /// Global user-level instructions (e.g., ~/.config/zene/rules/*.md). Lowest priority.
    Global,
    /// Modular project rules in subdirectories (e.g., .agents/rules/*.md, .gemini/rules/*.md).
    WorkspaceModular,
    /// Primary workspace root rule file (AGENTS.md, GEMINI.md, .cursorrules). Highest priority.
    WorkspaceRoot,
}

/// A parsed project rule or custom instruction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
    pub scope: RuleScope,
}

impl Rule {
    pub fn new(name: impl Into<String>, path: PathBuf, content: impl Into<String>, scope: RuleScope) -> Self {
        Self {
            name: name.into(),
            path,
            content: content.into(),
            scope,
        }
    }
}
