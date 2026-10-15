use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool allowing the agent to delegate subtasks to an isolated subagent.
/// Enforces a maximum recursion/nesting depth to prevent run-away spawning loops.
pub struct StartSubagentTool {
    current_depth: usize,
    max_depth: usize,
}

impl Default for StartSubagentTool {
    fn default() -> Self {
        Self::new()
    }
}

impl StartSubagentTool {
    pub fn new() -> Self {
        Self {
            current_depth: 0,
            max_depth: 2,
        }
    }

    pub fn with_depth(current_depth: usize, max_depth: usize) -> Self {
        Self {
            current_depth,
            max_depth,
        }
    }

    pub fn current_depth(&self) -> usize {
        self.current_depth
    }

    pub fn max_depth(&self) -> usize {
        self.max_depth
    }
}

impl Tool for StartSubagentTool {
    fn name(&self) -> &str {
        "start_subagent"
    }

    fn description(&self) -> &str {
        "Delegates a scoped, independent task to an isolated subagent. \
         The subagent executes in its own conversation workspace and returns a summary result upon completion. \
         Guarded by a maximum nesting depth limit."
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "task": {
                    "type": "string",
                    "description": "Clear, detailed task description and instructions for the subagent to execute."
                },
                "task_name": {
                    "type": "string",
                    "description": "Short human-readable title or label for the subagent run."
                },
                "max_turns": {
                    "type": "integer",
                    "description": "Maximum tool-call iterations allowed for the subagent (default: 10)."
                }
            },
            "required": ["task"]
        })
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        if self.current_depth >= self.max_depth {
            return Err(ToolError::ExecutionFailed(format!(
                "Recursion guard: Maximum subagent nesting depth limit ({}) reached. Cannot spawn deeper subagents.",
                self.max_depth
            )));
        }

        let task = args["task"].as_str().ok_or_else(|| {
            ToolError::InvalidArgs("Missing required string parameter 'task'".into())
        })?;

        let task_name = args["task_name"].as_str().unwrap_or("Subagent Task");
        let max_turns = args["max_turns"].as_u64().unwrap_or(10) as usize;

        // Instantiate isolated provider for subagent
        let provider = match crate::provider::create_default_provider() {
            Some(p) => p,
            None => {
                return Err(ToolError::ExecutionFailed(
                    "No LLM provider credentials configured to launch subagent. Please set GROQ_API_KEY or GEMINI_API_KEY.".into(),
                ));
            }
        };

        // Subagent gets minimal/essential tools and its own child StartSubagentTool with incremented depth
        let mut tools = crate::tools::ToolRegistry::essentials();
        tools.register(StartSubagentTool::with_depth(self.current_depth + 1, self.max_depth));

        let mut config = crate::orchestrator::AgentConfig::default();
        config.max_iterations = max_turns;

        let mut child_agent = crate::agentic_loop::Agent::new(provider, tools).with_config(config);
        let mut events = Vec::new();

        let run_result = child_agent.run(
            task,
            &mut |ev| events.push(ev),
            &mut |_approval_req| crate::types::approval::ApprovalDecision::Approve,
        );

        match run_result {
            Ok(summary) => Ok(json!({
                "subagent_status": "completed",
                "task_name": task_name,
                "nesting_depth": self.current_depth + 1,
                "summary": summary
            })),
            Err(e) => Err(ToolError::ExecutionFailed(format!(
                "Subagent '{}' encountered an execution error: {}",
                task_name, e
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subagent_depth_guard() {
        let tool = StartSubagentTool::with_depth(2, 2);
        let res = tool.execute(json!({ "task": "Check code" }));
        assert!(res.is_err());
        let err_str = res.unwrap_err().to_string();
        assert!(err_str.contains("Maximum subagent nesting depth limit (2) reached"));
    }

    #[test]
    fn test_subagent_missing_task_arg() {
        let tool = StartSubagentTool::with_depth(0, 2);
        let res = tool.execute(json!({}));
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("Missing required string parameter 'task'"));
    }
}
