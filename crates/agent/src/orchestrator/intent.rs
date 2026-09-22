use serde::{Deserialize, Serialize};

/// Categorizes the user's intent to decide tool provisioning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentIntent {
    /// Pure conversational exchange, brainstorming, advice, architecture Q&A.
    /// Loaded tools: 0 (saves ~3,750 tokens per message).
    Discussion,

    /// Code exploration, discovery, finding where logic lives.
    /// Loaded tools: search, read_file.
    Exploration,

    /// Concrete code modification, bug fixing, terminal execution.
    /// Loaded tools: targeted 3-5 action tools on-demand.
    Action(ActionCategory),
}

/// Action sub-categories defining targeted tool packages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActionCategory {
    /// Bug fixing, error diagnostics, runtime failure investigation.
    Debug,
    /// Adding new features, writing new code, implementing specifications.
    Implement,
    /// Restructuring code, renaming symbols, improving cleanliness.
    Refactor,
    /// Running tests, builds, compiler checks, verification.
    Verify,
    /// Diff inspection, git status, code quality review.
    Review,
    /// General multi-step execution.
    General,
}

impl AgentIntent {
    /// Returns the initial set of tool names needed for this intent.
    pub fn initial_tools(&self) -> Vec<&'static str> {
        match self {
            AgentIntent::Discussion => Vec::new(),
            AgentIntent::Exploration => vec!["search", "search_codebase", "find_files", "read_file", "list_directory"],
            AgentIntent::Action(cat) => match cat {
                ActionCategory::Debug => vec![
                    "search",
                    "search_codebase",
                    "find_files",
                    "read_file",
                    "edit_file",
                    "get_diagnostics",
                    "run_terminal",
                ],
                ActionCategory::Implement => vec![
                    "search",
                    "search_codebase",
                    "find_files",
                    "read_file",
                    "edit_file",
                    "write_file",
                    "get_diagnostics",
                ],
                ActionCategory::Refactor => vec![
                    "search",
                    "search_codebase",
                    "find_files",
                    "read_file",
                    "edit_file",
                    "rename_file",
                    "get_diagnostics",
                ],
                ActionCategory::Verify => vec![
                    "run_terminal",
                    "get_diagnostics",
                    "read_file",
                ],
                ActionCategory::Review => vec![
                    "git_status",
                    "git_diff",
                    "get_diagnostics",
                    "read_file",
                ],
                ActionCategory::General => vec![
                    "search",
                    "search_codebase",
                    "find_files",
                    "read_file",
                    "edit_file",
                    "get_diagnostics",
                    "run_terminal",
                ],
            },
        }
    }

    /// Fast heuristic classifier as a baseline / fallback when needed.
    pub fn classify_heuristic(prompt: &str) -> Self {
        let lower = prompt.to_lowercase();
        let trimmed = lower.trim();

        // 1. Check for pure conversational / discussion keywords
        let is_discussion = trimmed.starts_with("what do you think")
            || trimmed.starts_with("can you explain")
            || trimmed.starts_with("how should we")
            || trimmed.starts_with("why is")
            || trimmed.starts_with("difference between")
            || trimmed.contains("should i use")
            || trimmed.contains("pros and cons");

        let is_action_keyword = trimmed.contains("fix")
            || trimmed.contains("edit")
            || trimmed.contains("modify")
            || trimmed.contains("run cargo")
            || trimmed.contains("run test")
            || trimmed.contains("run the")
            || trimmed.starts_with("run ");

        if is_discussion && !is_action_keyword {
            return AgentIntent::Discussion;
        }

        // 2. Check for exploration
        let is_exploration = trimmed.starts_with("where is")
            || trimmed.starts_with("which file")
            || trimmed.starts_with("find")
            || trimmed.starts_with("locate")
            || trimmed.contains("where does");

        if is_exploration && !trimmed.contains("fix") && !trimmed.contains("modify") && !trimmed.contains("replace") {
            return AgentIntent::Exploration;
        }

        // 3. Action sub-categories
        if trimmed.contains("fix") || trimmed.contains("bug") || trimmed.contains("error") || trimmed.contains("panic") || trimmed.contains("issue") {
            return AgentIntent::Action(ActionCategory::Debug);
        }

        if trimmed.contains("cargo test") || trimmed.contains("verify") || trimmed.contains("test ") || trimmed.starts_with("test") || trimmed.contains("run cargo") || trimmed.starts_with("run ") {
            return AgentIntent::Action(ActionCategory::Verify);
        }

        if trimmed.contains("review") || trimmed.contains("diff") || trimmed.contains("status") {
            return AgentIntent::Action(ActionCategory::Review);
        }

        if trimmed.contains("refactor") || trimmed.contains("rename") || trimmed.contains("cleanup") {
            return AgentIntent::Action(ActionCategory::Refactor);
        }

        if trimmed.contains("add") || trimmed.contains("create") || trimmed.contains("implement") || trimmed.contains("build") || trimmed.contains("write") {
            return AgentIntent::Action(ActionCategory::Implement);
        }

        // Default to General Action if unsure
        AgentIntent::Action(ActionCategory::General)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::ToolRegistry;

    #[test]
    fn test_discussion_intent_has_zero_tools() {
        let intent = AgentIntent::Discussion;
        assert!(intent.initial_tools().is_empty());

        let classified = AgentIntent::classify_heuristic("Can you explain how async runtime works?");
        assert_eq!(classified, AgentIntent::Discussion);
        assert!(classified.initial_tools().is_empty());
    }

    #[test]
    fn test_exploration_intent_tools() {
        let intent = AgentIntent::classify_heuristic("Where is the user session token defined?");
        assert_eq!(intent, AgentIntent::Exploration);
        assert!(intent.initial_tools().contains(&"search_codebase"));
        assert!(intent.initial_tools().contains(&"read_file"));
    }

    #[test]
    fn test_action_debug_intent_tools() {
        let intent = AgentIntent::classify_heuristic("Please fix this compiler error");
        assert_eq!(intent, AgentIntent::Action(ActionCategory::Debug));
        assert!(intent.initial_tools().contains(&"edit_file"));
        assert!(intent.initial_tools().contains(&"get_diagnostics"));
    }

    #[test]
    fn test_selective_schemas_and_manifest() {
        let registry = ToolRegistry::all();
        let manifest = registry.manifest();
        assert!(manifest.contains("read_file"));
        assert!(manifest.contains("edit_file"));

        // Discussion has 0 schemas
        let empty_schemas = registry.openai_schemas_for(&[]);
        assert!(empty_schemas.is_empty());

        // Selective schemas only return requested tools
        let selected = registry.openai_schemas_for(&["read_file", "edit_file"]);
        assert_eq!(selected.len(), 2);
    }
}

