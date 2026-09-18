use std::path::{Path, PathBuf};
use crate::environment::host_detection::HostEnvironment;
use crate::rules::render_rules_prompt;
use crate::rules::scanner::RuleScanner;
use crate::skills::catalog::SkillCatalog;
use crate::skills::discovery::SkillDiscovery;
use crate::tools::{format_few_shot_tool_examples, format_tool_definitions_prompt};

/// Master system persona instructions for the autonomous coding agent.
pub const BASE_AGENT_INSTRUCTIONS: &str = "\
You are the Zenthree autonomous software engineering agent running inside the Zenthree editor.\n\
You help users complete software engineering tasks by understanding their codebase, making careful changes, and explaining your work clearly.\n\n\
## Operating Principles\n\
- Default to a tone that is concise, direct, and friendly. Prioritize actionable execution over verbose narration.\n\
- For tasks in an existing codebase, do exactly what the user asks with surgical precision. Fix the problem at the root cause rather than applying surface-level patches.\n\
- Keep changes minimal, focused, and consistent with the style and conventions of the existing codebase.\n\
- Multi-language & Framework Respect: Always strictly adhere to the user's requested language, framework, runtime, and package manager (e.g. React, Vue, Bun, Node, Python, Rust, Go). Never switch to or generate an unrelated language or toolchain simply because other files exist in the background.\n\
- Do not commit changes or create git branches unless the user explicitly requests it.\n\
- Never claim validation or tests passed unless you actually ran them with a tool and verified they passed.\n\n\
## Tool Use\n\
- Follow available tool schemas exactly and provide every required argument.\n\
- When multiple tool calls are independent, make them in parallel to maximize efficiency. If a tool call depends on a previous result, call them sequentially.\n\
- Before a group of related tool calls, send a brief one-sentence preamble explaining what you are about to do.\n\
- Do not re-read files after calling `write_file` or `edit_file` unless specifically needed; the tool execution output reports the result directly.";

/// Builder for dynamically generating the complete, context-aware master system prompt.
#[derive(Debug, Clone, Default)]
pub struct SystemPromptBuilder {
    workspace_root: Option<PathBuf>,
    custom_instructions: Option<String>,
    mcp_prompt_block: Option<String>,
    active_files: Vec<(PathBuf, String)>,
}

impl SystemPromptBuilder {
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        Self {
            workspace_root,
            custom_instructions: None,
            mcp_prompt_block: None,
            active_files: Vec::new(),
        }
    }

    /// Sets additional user or session instructions.
    pub fn with_custom_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.custom_instructions = Some(instructions.into());
        self
    }

    /// Injects active/open editor buffers into the prompt context.
    pub fn with_active_file(mut self, path: impl Into<PathBuf>, content_snippet: impl Into<String>) -> Self {
        self.active_files.push((path.into(), content_snippet.into()));
        self
    }

    /// Injects documentation of registered external MCP tools into the prompt.
    pub fn with_mcp_tools(mut self, mcp_block: impl Into<String>) -> Self {
        self.mcp_prompt_block = Some(mcp_block.into());
        self
    }

    /// Assembles the complete system prompt across all architectural layers.
    pub fn build(&self) -> String {
        let mut prompt = String::with_capacity(8192);

        // 1. Base Agent Persona & Principles
        prompt.push_str(BASE_AGENT_INSTRUCTIONS);
        prompt.push_str("\n\n");

        if let Some(ref custom) = self.custom_instructions {
            prompt.push_str("## Additional Session Instructions\n");
            prompt.push_str(custom);
            prompt.push_str("\n\n");
        }

        // 2. Host Environment (Phase 1)
        let host_env = HostEnvironment::detect();
        prompt.push_str(&host_env.format_prompt_block());
        prompt.push_str("\n\n");

        // 3. Workspace Rules (Phase 3)
        if let Some(ref root) = self.workspace_root {
            let rules = RuleScanner::new().scan_workspace(root);
            if let Some(rules_block) = render_rules_prompt(&rules) {
                if !rules_block.trim().is_empty() {
                    prompt.push_str(&rules_block);
                    prompt.push_str("\n\n");
                }
            }
        }

        // 4. Progressive Disclosure Skills Catalog (Phase 3)
        let roots: Vec<&Path> = self.workspace_root.as_deref().into_iter().collect();
        let skills_map = SkillDiscovery::discover_skills(&roots, true);
        let skills: Vec<&crate::skills::Skill> = skills_map.values().collect();
        let catalog = SkillCatalog::from_skills(skills.into_iter());
        let skills_block = catalog.format_prompt_block();
        if !skills_block.trim().is_empty() {
            prompt.push_str(&skills_block);
            prompt.push_str("\n\n");
        }

        // 5. Tool Definitions & Formats
        prompt.push_str(&format_tool_definitions_prompt());
        prompt.push_str("\n\n");

        // 6. External MCP Tools (Phase 8)
        if let Some(ref mcp_block) = self.mcp_prompt_block {
            if !mcp_block.trim().is_empty() {
                prompt.push_str(mcp_block);
                prompt.push_str("\n\n");
            }
        }

        // 7. Few-Shot Tool Usage Examples (Phase 4)
        prompt.push_str(&format_few_shot_tool_examples());
        prompt.push_str("\n\n");

        // 8. Active Open Files & Working Buffers
        if !self.active_files.is_empty() {
            prompt.push_str("## Active Working Buffers\n");
            prompt.push_str("The user currently has the following files open in their editor:\n\n");
            for (path, content) in &self.active_files {
                prompt.push_str(&format!(
                    "<active_file path=\"{}\">\n{}\n</active_file>\n\n",
                    path.display(),
                    content
                ));
            }
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_prompt_builder_composition() {
        let temp_dir = TempDir::new().unwrap();
        let ws = temp_dir.path().to_path_buf();

        let builder = SystemPromptBuilder::new(Some(ws))
            .with_custom_instructions("Always write tests before code.")
            .with_active_file("src/main.rs", "fn main() {}");

        let prompt = builder.build();

        // Persona check
        assert!(prompt.contains("Zenthree autonomous software engineering agent"));
        assert!(prompt.contains("surgical precision"));

        // Custom instructions check
        assert!(prompt.contains("Always write tests before code."));

        // Host environment check
        assert!(prompt.contains("## System Environment"));

        // Available tools check
        assert!(prompt.contains("## Available Engineering Tools & Calling Formats"));
        assert!(prompt.contains("write_file"));
        assert!(prompt.contains("terminal"));

        // Few shot examples check
        assert!(prompt.contains("<tool_call_examples>"));

        // Active files check
        assert!(prompt.contains("<active_file path=\"src/main.rs\">"));
        assert!(prompt.contains("fn main() {}"));
    }
}
