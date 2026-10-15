use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::tasks::{SpawnOutcome, TaskManager};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for running terminal commands (build, test, lint, background daemons).
#[derive(Clone, Default)]
pub struct RunTerminalTool {
    workspace: Option<PathBuf>,
    task_manager: Option<TaskManager>,
}

impl RunTerminalTool {
    pub fn new() -> Self {
        Self {
            workspace: None,
            task_manager: None,
        }
    }

    pub fn with_workspace(mut self, workspace: impl Into<PathBuf>) -> Self {
        self.workspace = Some(workspace.into());
        self
    }

    pub fn with_task_manager(mut self, task_manager: TaskManager) -> Self {
        self.task_manager = Some(task_manager);
        self
    }
}

impl Tool for RunTerminalTool {
    fn name(&self) -> &str {
        "run_terminal"
    }

    fn description(&self) -> &str {
        "Run builds, test suites, linters, or background daemons in the terminal. Supports synchronous execution and automatic backgrounding via WaitMsBeforeAsync and IsDaemon."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "CommandLine": {
                    "type": "string",
                    "description": "Shell command to execute"
                },
                "Cwd": {
                    "type": "string",
                    "description": "Optional working directory"
                },
                "WaitMsBeforeAsync": {
                    "type": "integer",
                    "description": "Milliseconds to wait synchronously before pushing the command to a background task (default: 3000ms)"
                },
                "IsDaemon": {
                    "type": "boolean",
                    "description": "Set to true for long-running support processes (dev servers, file watchers) that should run indefinitely in the background"
                }
            },
            "required": ["CommandLine"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Destructive
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let command_str = args.get("CommandLine")
            .or_else(|| args.get("command"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'CommandLine'".to_string()))?;

        let cwd_str = args.get("Cwd")
            .or_else(|| args.get("cwd"))
            .and_then(|v| v.as_str());

        let is_daemon = args.get("IsDaemon")
            .or_else(|| args.get("is_daemon"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let wait_ms = args.get("WaitMsBeforeAsync")
            .or_else(|| args.get("wait_ms"))
            .and_then(|v| v.as_u64())
            .unwrap_or(3000);

        let resolved_cwd = if let Some(cwd_p) = cwd_str {
            let p = Path::new(cwd_p);
            if let Some(ref ws) = self.workspace {
                if p.is_absolute() { p.to_path_buf() } else { ws.join(p) }
            } else {
                p.to_path_buf()
            }
        } else if let Some(ref ws) = self.workspace {
            ws.clone()
        } else {
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        };

        if let Some(ref ws) = self.workspace {
            let canon_ws = ws.canonicalize().unwrap_or_else(|_| ws.clone());
            let canon_cwd = resolved_cwd.canonicalize().unwrap_or_else(|_| resolved_cwd.clone());
            if !canon_cwd.starts_with(&canon_ws) {
                return Err(ToolError::ExecutionFailed(format!(
                    "Access denied: Working directory '{}' is outside the project workspace boundary ('{}').",
                    resolved_cwd.display(),
                    ws.display()
                )));
            }
        }

        let mgr = self.task_manager.clone().unwrap_or_default();
        let outcome = mgr.spawn(command_str, &resolved_cwd, is_daemon, wait_ms)
            .map_err(|e| ToolError::ExecutionFailed(e))?;

        match outcome {
            SpawnOutcome::Background { message, .. } => Ok(Value::String(message)),
            SpawnOutcome::Synchronous { exit_code, stdout, stderr } => {
                let mut result = format!("Exit code: {}\n", exit_code);
                if !stdout.trim().is_empty() {
                    result.push_str(&format!("--- STDOUT ---\n{}\n", stdout.trim_end()));
                }
                if !stderr.trim().is_empty() {
                    result.push_str(&format!("--- STDERR ---\n{}\n", stderr.trim_end()));
                }
                Ok(Value::String(result))
            }
        }
    }
}
