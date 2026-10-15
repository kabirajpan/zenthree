use crate::orchestrator::intent::{ActionCategory, AgentIntent};

/// Recognized user-facing slash commands from `AGENT_SKILLS_SLASH_COMMANDS_CUSTOMIZATION.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashCommand {
    /// Autonomous long-running execution mode
    Goal(String),
    /// Enforces architectural planning before code modification
    Plan(String),
    /// Targeted debugging and error diagnosis
    Fix(String),
    /// Socratic interview to resolve ambiguous requirements
    GrillMe(String),
    /// Manual context compaction
    Compact,
    /// Schedule a timer or recurring notification
    Schedule(String),
    /// Persist learned pattern or rule
    Learn(String),
    /// Background tasks dashboard
    Tasks,
    /// Show slash command help
    Help,
}

impl SlashCommand {
    /// Parse a line starting with '/' into a typed SlashCommand.
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if !trimmed.starts_with('/') {
            return None;
        }

        let without_slash = &trimmed[1..];
        let mut parts = without_slash.splitn(2, ' ');
        let cmd = parts.next()?.trim().to_lowercase();
        let arg = parts.next().map(|s| s.trim().to_string()).unwrap_or_default();

        match cmd.as_str() {
            "goal" => Some(SlashCommand::Goal(arg)),
            "plan" => Some(SlashCommand::Plan(arg)),
            "fix" => Some(SlashCommand::Fix(arg)),
            "grill-me" | "grillme" | "grill" => Some(SlashCommand::GrillMe(arg)),
            "compact" => Some(SlashCommand::Compact),
            "schedule" => Some(SlashCommand::Schedule(arg)),
            "learn" => Some(SlashCommand::Learn(arg)),
            "tasks" | "task" => Some(SlashCommand::Tasks),
            "help" => Some(SlashCommand::Help),
            _ => None,
        }
    }

    /// Associated intent override for this command, if applicable.
    pub fn intent_override(&self) -> Option<AgentIntent> {
        match self {
            SlashCommand::Goal(_) => Some(AgentIntent::Action(ActionCategory::General)),
            SlashCommand::Plan(_) => Some(AgentIntent::Action(ActionCategory::Implement)),
            SlashCommand::Fix(_) => Some(AgentIntent::Action(ActionCategory::Debug)),
            SlashCommand::GrillMe(_) => Some(AgentIntent::Discussion),
            _ => None,
        }
    }

    /// Enriches the prompt with directive meta-instructions when forwarded to the agent.
    pub fn enriched_prompt(&self) -> String {
        match self {
            SlashCommand::Goal(task) => {
                let body = if task.is_empty() { "Complete the active project goal" } else { task };
                format!(
                    "[DIRECTIVE: /goal - AUTONOMOUS CONTINUOUS EXECUTION]\n\
                    Execute continuously and thoroughly until all requirements are met and verified.\n\
                    Run compilation checks or test suites via terminal to ensure zero regressions.\n\
                    Task: {}",
                    body
                )
            }
            SlashCommand::Plan(task) => {
                let body = if task.is_empty() { "Formulate implementation plan for the active task" } else { task };
                format!(
                    "[DIRECTIVE: /plan - STRUCTURED PLANNING MODE]\n\
                    Before making any code edits or running terminal mutations, formulate a clear, step-by-step implementation plan.\n\
                    Use the 'create_plan' tool to establish structured milestones.\n\
                    Task: {}",
                    body
                )
            }
            SlashCommand::Fix(issue) => {
                let body = if issue.is_empty() { "Diagnose and fix errors in the codebase" } else { issue };
                format!(
                    "[DIRECTIVE: /fix - TARGETED DIAGNOSTIC & REPAIR MODE]\n\
                    1. Use 'get_diagnostics' or compiler tools to identify the exact error.\n\
                    2. Inspect the offending symbol using 'read_file'.\n\
                    3. Apply targeted minimal edits via 'edit_file' or 'multi_replace_file_content'.\n\
                    4. Re-run diagnostics or tests to confirm resolution.\n\
                    Issue: {}",
                    body
                )
            }
            SlashCommand::GrillMe(topic) => {
                let body = if topic.is_empty() { "the current project requirements" } else { topic };
                format!(
                    "[DIRECTIVE: /grill-me - INTERACTIVE INTERVIEW MODE]\n\
                    Grill the user with targeted, probing questions regarding architectural trade-offs, edge cases, and design decisions for: {}\n\
                    Do not modify code until the user has answered your questions.",
                    body
                )
            }
            SlashCommand::Schedule(args) => {
                format!(
                    "[DIRECTIVE: /schedule - AUTOMATION SCHEDULE]\n\
                    Configure timer or recurring event with schedule tool: {}",
                    args
                )
            }
            SlashCommand::Learn(rule) => {
                format!(
                    "[DIRECTIVE: /learn - KNOWLEDGE PERSISTENCE]\n\
                    Record and remember this architectural pattern or developer preference: {}",
                    rule
                )
            }
            _ => String::new(),
        }
    }
}

/// Generates system prompt XML documenting available slash commands so the model can recommend them.
pub fn format_slash_commands_prompt() -> String {
    "<slash_commands>\n\
Slash commands are user-facing shortcuts in the chat interface that automate workflows or trigger specialized agent behaviors.\n\
You cannot execute slash commands yourself; recommend them to the user when appropriate.\n\n\
Available slash commands you can recommend to the user:\n\
- /goal: Recommend when the user wants autonomous, end-to-end execution that verifies all requirements.\n\
- /plan: Recommend when a task is complex and requires structured step-by-step milestones before modifying code.\n\
- /fix: Recommend for targeted compiler error diagnostics and minimal repair.\n\
- /grill-me: Recommend when architectural requirements or design choices are ambiguous and need clarification.\n\
- /compact: Recommend when context history is very long and should be summarized.\n\
- /schedule: Recommend when the user wants recurring cron checks or one-shot timer reminders.\n\
- /tasks: Display the active background tasks dashboard.\n\
- /learn: Recommend to persist newly discovered rules or conventions into the project rules.\n\
</slash_commands>".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slash_commands() {
        assert_eq!(
            SlashCommand::parse("/goal build release"),
            Some(SlashCommand::Goal("build release".into()))
        );
        assert_eq!(
            SlashCommand::parse("/plan implement auth"),
            Some(SlashCommand::Plan("implement auth".into()))
        );
        assert_eq!(
            SlashCommand::parse("/fix type error"),
            Some(SlashCommand::Fix("type error".into()))
        );
        assert_eq!(
            SlashCommand::parse("/grill-me api design"),
            Some(SlashCommand::GrillMe("api design".into()))
        );
        assert_eq!(SlashCommand::parse("/compact"), Some(SlashCommand::Compact));
        assert_eq!(SlashCommand::parse("/tasks"), Some(SlashCommand::Tasks));
        assert_eq!(SlashCommand::parse("not a slash command"), None);
    }

    #[test]
    fn test_slash_command_directives() {
        let cmd = SlashCommand::Plan("add jwt auth".into());
        let enriched = cmd.enriched_prompt();
        assert!(enriched.contains("STRUCTURED PLANNING MODE"));
        assert!(enriched.contains("create_plan"));
        assert_eq!(
            cmd.intent_override(),
            Some(AgentIntent::Action(ActionCategory::Implement))
        );
    }
}
