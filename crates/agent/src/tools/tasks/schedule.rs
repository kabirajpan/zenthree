use serde_json::{json, Value};
use crate::tasks::{Scheduler, TimerCondition};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for setting one-shot timers and recurring cron wakeups.
#[derive(Clone)]
pub struct ScheduleTool {
    scheduler: Scheduler,
}

impl ScheduleTool {
    pub fn new(scheduler: Scheduler) -> Self {
        Self { scheduler }
    }
}

impl Tool for ScheduleTool {
    fn name(&self) -> &str {
        "schedule"
    }

    fn description(&self) -> &str {
        "Schedule a one-shot timer or recurring schedule that reactively wakes up the agent without polling."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "DurationSeconds": {
                    "type": "integer",
                    "description": "Duration in seconds for a one-shot timer"
                },
                "IntervalSeconds": {
                    "type": "integer",
                    "description": "Interval in seconds for recurring schedule"
                },
                "Prompt": {
                    "type": "string",
                    "description": "The message or reminder to notify the agent when the timer expires"
                },
                "TimerCondition": {
                    "type": "string",
                    "enum": ["never", "any"],
                    "description": "Early cancellation condition: 'never' (default) or 'any' (cancel if message received)"
                },
                "MaxIterations": {
                    "type": "integer",
                    "description": "Optional maximum number of iterations for recurring schedule"
                }
            },
            "required": ["Prompt"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let prompt = args.get("Prompt")
            .or_else(|| args.get("prompt"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'Prompt'".to_string()))?
            .to_string();

        let duration = args.get("DurationSeconds")
            .or_else(|| args.get("duration_seconds"))
            .and_then(|v| v.as_u64());

        let interval = args.get("IntervalSeconds")
            .or_else(|| args.get("interval_seconds"))
            .and_then(|v| v.as_u64());

        if let Some(dur) = duration {
            let cond_str = args.get("TimerCondition")
                .or_else(|| args.get("timer_condition"))
                .and_then(|v| v.as_str())
                .unwrap_or("never");

            let cond = match cond_str {
                "any" => TimerCondition::Any,
                _ => TimerCondition::Never,
            };

            let res = self.scheduler.schedule_timer(dur, prompt, cond);
            Ok(Value::String(res))
        } else if let Some(inter) = interval {
            let max_iter = args.get("MaxIterations")
                .or_else(|| args.get("max_iterations"))
                .and_then(|v| v.as_u64())
                .map(|v| v as usize);

            let res = self.scheduler.schedule_interval(inter, prompt, max_iter);
            Ok(Value::String(res))
        } else {
            Err(ToolError::InvalidArgs(
                "Either 'DurationSeconds' (for one-shot timer) or 'IntervalSeconds' (for recurring schedule) must be specified.".to_string(),
            ))
        }
    }
}
