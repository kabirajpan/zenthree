use std::process::Command;
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for running terminal commands (build, test, lint, etc.).
pub struct RunTerminalTool;

impl Tool for RunTerminalTool {
    fn name(&self) -> &str {
        "run_terminal"
    }

    fn description(&self) -> &str {
        "Run a build, test, lint, or shell command in the terminal and capture its exit status, stdout, and stderr."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "cwd": {
                    "type": "string",
                    "description": "Optional working directory"
                }
            },
            "required": ["command"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Destructive
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let command_str = args.get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'command'".to_string()))?;

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command_str]);
            c
        };

        if let Some(cwd) = args.get("cwd").and_then(|v| v.as_str()) {
            cmd.current_dir(cwd);
        }

        let output = cmd.output()
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to spawn command '{}': {}", command_str, e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let code = output.status.code().unwrap_or(-1);

        let mut result = format!("Exit code: {}\n", code);
        if !stdout.trim().is_empty() {
            result.push_str(&format!("--- STDOUT ---\n{}\n", stdout.trim_end()));
        }
        if !stderr.trim().is_empty() {
            result.push_str(&format!("--- STDERR ---\n{}\n", stderr.trim_end()));
        }

        Ok(Value::String(result))
    }
}
