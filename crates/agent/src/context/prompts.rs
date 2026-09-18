use crate::orchestrator::SystemPromptBuilder;

/// System prompt instructing the autonomous agent with modern tools, rules, and protocols.
pub fn build_autonomous_agent_prompt() -> String {
    SystemPromptBuilder::new(None).build()
}

/// Discussion prompt configured with guidance instructions and modern protocols.
pub fn build_discussion_guide_prompt() -> String {
    SystemPromptBuilder::new(None)
        .with_custom_instructions("Discussion Mode: Answer questions concisely, explain concepts clearly, and guide the user without making unsolicited file modifications unless requested.")
        .build()
}
