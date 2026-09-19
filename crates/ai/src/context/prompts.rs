use crate::tools::ToolRegistry;

/// Constructs the clean autonomous agent system prompt with the Tool Registry and syntax examples.
pub fn build_autonomous_agent_prompt(_deliberation_confidence: f32) -> String {
    format!(
        "You are ZENE, an autonomous software engineering agent in the Zenthree code editor with direct workspace access.\n\n\
        Internal Deliberation:\n\
        Before calling tools, writing files, or answering, write your internal reasoning inside `<thought> ... </thought>`.\n\n\
        {}\n\
        ### File Operations:\n\n\
        To create a new file or completely rewrite an existing file:\n\
        ### File: relative/path/to/file.ext\n\
        ```<language>\n\
        <complete file content>\n\
        ```\n\n\
        To surgically edit an existing file with search and replace:\n\
        ### File: relative/path/to/file.ext\n\
        <<<<<<< SEARCH\n\
        <exact existing lines to replace>\n\
        =======\n\
        <new replacement lines>\n\
        >>>>>>> REPLACE\n\n\
        When answering questions, explaining architecture, or discussing ideas, respond directly in Markdown without emitting tool calls or file blocks.",
        ToolRegistry::render_for_prompt()
    )
}

/// Constructs the conversational discussion guide prompt with the Tool Registry for project inspection.
pub fn build_discussion_guide_prompt() -> String {
    format!(
        "You are ZENE, an intelligent software engineering partner in the Zenthree code editor.\n\
        You are having a discussion with the user to answer questions, brainstorm ideas, explain concepts, or review code.\n\n\
        {}\n\
        Respond in clear, helpful Markdown. If you need to inspect real files, directory structure, or git state, use the available tools.",
        ToolRegistry::render_for_prompt()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autonomous_prompt_contains_tool_registry_and_file_examples() {
        let prompt = build_autonomous_agent_prompt(95.0);
        assert!(prompt.contains("You are ZENE"));
        assert!(prompt.contains("<thought> ... </thought>"));
        assert!(prompt.contains("### Tool Registry:"));
        assert!(prompt.contains("`run_command`"));
        assert!(prompt.contains("`read_file`"));
        assert!(prompt.contains("`search_code`"));
        assert!(prompt.contains("`edit_file`"));
        assert!(prompt.contains("### File: relative/path/to/file.ext"));
        assert!(prompt.contains("<<<<<<< SEARCH"));
        assert!(prompt.contains(">>>>>>> REPLACE"));
    }

    #[test]
    fn test_discussion_prompt_contains_tool_registry() {
        let prompt = build_discussion_guide_prompt();
        assert!(prompt.contains("You are ZENE"));
        assert!(prompt.contains("### Tool Registry:"));
        assert!(prompt.contains("`read_file`"));
        assert!(prompt.contains("`search_code`"));
    }
}
