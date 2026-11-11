use serde_json::{json, Value};
use crate::tasks::TaskManager;
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for managing supervised background tasks (list, status, send_input, kill).
#[derive(Clone)]
pub struct ManageTaskTool {
    task_manager: TaskManager,
}

impl ManageTaskTool {
    pub fn new(task_manager: TaskManager) -> Self {
        Self { task_manager }
    }
}

impl Default for ManageTaskTool {
    fn default() -> Self {
        Self {
            task_manager: TaskManager::default(),
        }
    }
}

impl Tool for ManageTaskTool {
    fn name(&self) -> &str {
        "manage_task"
    }

    fn description(&self) -> &str {
        "Manage background tasks. Use this tool to list running tasks, check execution status and logs, send stdin input, or terminate tasks cleanly."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "Action": {
                    "type": "string",
                    "enum": ["list", "status", "send_input", "kill"],
                    "description": "Action to perform: 'list' (all tasks), 'status' (details & logs), 'send_input' (send stdin), 'kill' (terminate)"
                },
                "TaskId": {
                    "type": "string",
                    "description": "The task ID to manage (required for 'status', 'send_input', 'kill')"
                },
                "Input": {
                    "type": "string",
                    "description": "The string input to write to stdin (required for 'send_input')"
                }
            },
            "required": ["Action"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Mutating
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let action = args.get("Action")
            .or_else(|| args.get("action"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'Action'".to_string()))?;

        match action {
            "list" => {
                let list = self.task_manager.list();
                if list.is_empty() {
                    Ok(Value::String("No background tasks currently active or recorded.".to_string()))
                } else {
                    let mut out = format!("Active & Supervised Background Tasks ({}):\n", list.len());
                    for t in list {
                        out.push_str(&format!(
                            "  • ID: {}\n    Command: {}\n    Status: {}\n    Elapsed: {}s\n    Logs: {}\n",
                            t.task_id, t.command, t.status, t.elapsed_seconds, t.log_path
                        ));
                    }
                    Ok(Value::String(out))
                }
            }
            "status" => {
                let task_id = args.get("TaskId")
                    .or_else(|| args.get("task_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'TaskId'".to_string()))?;

                let info = self.task_manager.status(task_id)
                    .ok_or_else(|| ToolError::ExecutionFailed(format!("Task '{}' not found", task_id)))?;

                Ok(Value::String(info))
            }
            "send_input" => {
                let task_id = args.get("TaskId")
                    .or_else(|| args.get("task_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'TaskId'".to_string()))?;

                let input = args.get("Input")
                    .or_else(|| args.get("input"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'Input'".to_string()))?;

                self.task_manager.send_input(task_id, input)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(Value::String(format!("Successfully sent input to task '{}'", task_id)))
            }
            "kill" => {
                let task_id = args.get("TaskId")
                    .or_else(|| args.get("task_id"))
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'TaskId'".to_string()))?;

                self.task_manager.kill(task_id)
                    .map_err(|e| ToolError::ExecutionFailed(e))?;

                Ok(Value::String(format!("Successfully terminated background task '{}'", task_id)))
            }
            other => Err(ToolError::InvalidArgs(format!(
                "Unknown Action '{}'. Supported actions are: 'list', 'status', 'send_input', 'kill'.",
                other
            ))),
        }
    }
}
