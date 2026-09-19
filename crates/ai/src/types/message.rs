use serde::{Deserialize, Serialize};
use crate::agents::SubAgentRole;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExploredItem {
    pub file_name: String,
    pub line_range: Option<String>,
    pub action_type: String,
    #[serde(default)]
    pub thought_content: Option<String>,
    #[serde(default)]
    pub duration_secs: Option<u64>,
}

impl ExploredItem {
    pub fn new(file_name: impl Into<String>, line_range: Option<String>, action_type: impl Into<String>) -> Self {
        Self {
            file_name: file_name.into(),
            line_range,
            action_type: action_type.into(),
            thought_content: None,
            duration_secs: None,
        }
    }

    pub fn new_thought(content: impl Into<String>, duration_secs: u64) -> Self {
        Self {
            file_name: String::new(),
            line_range: None,
            action_type: "Thought".to_string(),
            thought_content: Some(content.into()),
            duration_secs: Some(duration_secs),
        }
    }
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
    #[serde(default)]
    pub elapsed_secs: Option<u64>,
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
            elapsed_secs: None,
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
    let words: Vec<&str> = lower.split_whitespace().collect();
    if words.is_empty() {
        return UserIntent::GeneralChat;
    }

    // 1. Greetings and casual chit-chat
    let is_greeting = words[0] == "hi"
        || words[0] == "hello"
        || words[0] == "hey"
        || words[0] == "greetings"
        || lower.starts_with("good morning")
        || lower.starts_with("good afternoon")
        || lower.starts_with("good evening")
        || lower.starts_with("how are you")
        || lower.starts_with("who are you")
        || lower.starts_with("what can you do");

    if is_greeting && words.len() <= 6 {
        return UserIntent::GeneralChat;
    }

    // 2. Explicit action commands override general discussion
    // e.g. "Can we talk about fixing this bug? Fix it now." -> action takes precedence
    let has_explicit_action_command = lower.contains("fix it")
        || lower.contains("fix this")
        || lower.contains("write code")
        || lower.contains("write the code")
        || lower.contains("apply the fix")
        || lower.contains("patch it")
        || lower.contains("implement this");

    // 3. Strong conversational, brainstorming, advisory, and non-code signals
    let is_discussion = !has_explicit_action_command && (
        lower.contains("discuss")
            || lower.contains("talk about")
            || lower.contains("what do you think")
            || lower.contains("what's your thought")
            || lower.contains("opinion on")
            || lower.contains("thoughts on")
            || lower.contains("should we")
            || lower.contains("should i")
            || lower.contains("pros and cons")
            || lower.contains("compare")
            || lower.contains("brainstorm")
            || lower.contains("ideas for")
            || lower.contains("recommend")
            || lower.contains("why is")
            || lower.contains("why does")
            || lower.contains("tell me about")
            || lower.contains("difference between")
            || lower.contains("can we talk")
    );

    if is_discussion {
        return UserIntent::DiscussionAndGuide;
    }

    // 4. Action-oriented "how to": if user asks "how to fix ...", "how to add ...",
    // "how to implement ...", or "how to write ...", route to BugFix or CodeGeneration!
    if lower.contains("how to fix") || lower.contains("how do i fix") || lower.contains("how can i fix") {
        return UserIntent::BugFix;
    }
    if lower.contains("how to add")
        || lower.contains("how to create")
        || lower.contains("how to implement")
        || lower.contains("how to build")
        || lower.contains("how to write")
    {
        return UserIntent::CodeGeneration;
    }

    // 5. Explanation and conceptual understanding (pure explanations)
    if lower.contains("explain")
        || lower.contains("how does")
        || lower.contains("what is")
        || lower.contains("walk through")
        || lower.contains("understand")
        || lower.contains("teach me")
        || lower.contains("how to")
    {
        return UserIntent::CodeExplanation;
    }

    // 6. Refactoring
    if lower.contains("refactor")
        || lower.contains("clean up")
        || lower.contains("optimize")
        || lower.contains("restructure")
        || lower.contains("modularize")
    {
        return UserIntent::Refactoring;
    }

    // 7. Bug fixing
    if lower.contains("fix")
        || lower.contains("bug")
        || lower.contains("error")
        || lower.contains("failing")
        || lower.contains("panic")
        || lower.contains("broken")
        || lower.contains("crash")
    {
        return UserIntent::BugFix;
    }

    // 8. Architecture Planning
    if lower.contains("plan")
        || lower.contains("architect")
        || lower.contains("design")
        || lower.contains("roadmap")
        || lower.contains("step by step")
    {
        return UserIntent::ArchitecturePlanning;
    }

    // 9. Direct Code Generation / Modification actions
    let is_code_gen = lower.contains("create a")
        || lower.contains("create new")
        || lower.contains("add a")
        || lower.contains("add new")
        || lower.contains("build a")
        || lower.contains("implement")
        || lower.contains("write code")
        || lower.contains("write a")
        || lower.contains("make a")
        || lower.contains("code for")
        || lower.starts_with("create")
        || lower.starts_with("add")
        || lower.starts_with("write")
        || lower.starts_with("make")
        || lower.starts_with("build");

    if is_code_gen {
        return UserIntent::CodeGeneration;
    }

    UserIntent::GeneralChat
}
