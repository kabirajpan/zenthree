use serde::{Deserialize, Serialize};
use crate::agents::SubAgentRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploredItem {
    pub file_name: String,
    pub line_range: Option<String>,
    pub action_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditedFileSummary {
    pub file_name: String,
    pub lines_added: usize,
    pub lines_removed: usize,
    #[serde(default)]
    pub original_code: Option<String>,
    #[serde(default)]
    pub proposed_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiMessage {
    pub is_user: bool,
    pub content: String,
    pub code_snippet: Option<String>,
    pub thoughts: Option<String>,
    #[serde(default)]
    pub deliberation: Option<connectome::DecisionResult>,
    #[serde(default)]
    pub explored_items: Vec<ExploredItem>,
    #[serde(default)]
    pub edited_files: Vec<EditedFileSummary>,
    #[serde(default)]
    pub is_working: bool,
    #[serde(default)]
    pub sub_agent: Option<SubAgentRole>,
}

impl Default for AiMessage {
    fn default() -> Self {
        Self {
            is_user: false,
            content: String::new(),
            code_snippet: None,
            thoughts: None,
            deliberation: None,
            explored_items: Vec::new(),
            edited_files: Vec::new(),
            is_working: false,
            sub_agent: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: content.into(),
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new("user", content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new("assistant", content)
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new("system", content)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserIntent {
    DiscussionAndGuide,
    WorkAndProject,
    CodeGeneration,
    CodeExplanation,
    Refactoring,
    BugFix,
    ArchitecturePlanning,
    GeneralChat,
}

impl UserIntent {
    pub fn is_discussion(&self) -> bool {
        matches!(self, Self::DiscussionAndGuide | Self::CodeExplanation | Self::GeneralChat)
    }

    pub fn is_work(&self) -> bool {
        matches!(
            self,
            Self::WorkAndProject
                | Self::CodeGeneration
                | Self::Refactoring
                | Self::BugFix
                | Self::ArchitecturePlanning
        )
    }
}

pub fn classify_user_intent(text: &str) -> UserIntent {
    let lower = text.trim().to_lowercase();
    if lower.contains("explain")
        || lower.contains("how does")
        || lower.contains("what is")
        || lower.contains("walk through")
        || lower.contains("understand")
    {
        UserIntent::CodeExplanation
    } else if lower.contains("refactor")
        || lower.contains("clean up")
        || lower.contains("optimize")
        || lower.contains("restructure")
        || lower.contains("modularize")
    {
        UserIntent::Refactoring
    } else if lower.contains("fix")
        || lower.contains("bug")
        || lower.contains("error")
        || lower.contains("failing")
        || lower.contains("panic")
        || lower.contains("broken")
    {
        UserIntent::BugFix
    } else if lower.contains("plan")
        || lower.contains("architect")
        || lower.contains("design")
        || lower.contains("roadmap")
        || lower.contains("step by step")
    {
        UserIntent::ArchitecturePlanning
    } else if lower.contains("create")
        || lower.contains("add")
        || lower.contains("build")
        || lower.contains("implement")
        || lower.contains("make")
        || lower.contains("write")
    {
        UserIntent::CodeGeneration
    } else {
        UserIntent::GeneralChat
    }
}
