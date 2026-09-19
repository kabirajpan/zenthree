use std::sync::{Arc, Mutex};
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PlanStep {
    pub id: usize,
    pub description: String,
    pub status: String,
}

#[derive(Clone, Default)]
pub struct PlanState {
    pub steps: Vec<PlanStep>,
}

/// Tool for externalizing a structured multi-step plan before execution.
pub struct CreatePlanTool {
    state: Arc<Mutex<PlanState>>,
}

impl CreatePlanTool {
    pub fn new(state: Arc<Mutex<PlanState>>) -> Self {
        Self { state }
    }
}

impl Tool for CreatePlanTool {
    fn name(&self) -> &str {
        "create_plan"
    }

    fn description(&self) -> &str {
        "Create an explicit multi-step plan before performing complex modifications."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "steps": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Sequential action steps required to complete the task"
                }
            },
            "required": ["steps"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let steps_arr = args.get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'steps' (must be an array of strings)".to_string()))?;

        let mut state = self.state.lock().map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
        state.steps.clear();

        let mut output = String::from("Plan created:\n");
        for (i, step_val) in steps_arr.iter().enumerate() {
            let desc = step_val.as_str().unwrap_or("").to_string();
            state.steps.push(PlanStep {
                id: i + 1,
                description: desc.clone(),
                status: "pending".to_string(),
            });
            output.push_str(&format!("  [ ] Step {}: {}\n", i + 1, desc));
        }

        Ok(Value::String(output))
    }
}

/// Tool for updating the status of an ongoing plan step.
pub struct UpdatePlanStepTool {
    state: Arc<Mutex<PlanState>>,
}

impl UpdatePlanStepTool {
    pub fn new(state: Arc<Mutex<PlanState>>) -> Self {
        Self { state }
    }
}

impl Tool for UpdatePlanStepTool {
    fn name(&self) -> &str {
        "update_plan_step"
    }

    fn description(&self) -> &str {
        "Update the status of a specific step in the current plan ('pending', 'in_progress', 'completed', 'failed')."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "step_id": {
                    "type": "integer",
                    "description": "1-indexed step number"
                },
                "status": {
                    "type": "string",
                    "enum": ["pending", "in_progress", "completed", "failed"],
                    "description": "Current status of this step"
                }
            },
            "required": ["step_id", "status"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let step_id = args.get("step_id")
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'step_id'".to_string()))?;

        let status = args.get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'status'".to_string()))?;

        let mut state = self.state.lock().map_err(|e| ToolError::ExecutionFailed(e.to_string()))?;
        if step_id == 0 || step_id > state.steps.len() {
            return Err(ToolError::ExecutionFailed(format!("Step {} does not exist (total steps: {})", step_id, state.steps.len())));
        }

        let step = &mut state.steps[step_id - 1];
        step.status = status.to_string();

        let mark = match status {
            "completed" => "[x]",
            "in_progress" => "[>]",
            "failed" => "[!]",
            _ => "[ ]",
        };

        Ok(Value::String(format!("{} Step {}: {} -> {}", mark, step_id, step.description, status)))
    }
}
