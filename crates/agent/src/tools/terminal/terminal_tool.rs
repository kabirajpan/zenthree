use crate::environment::{AgentEnvironment, ProcessExecutionOptions};
use crate::permissions::{HardcodedSecurityRules, PermissionEngine, ToolPermissionDecision};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Input parameters for the `terminal` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalInput {
    pub command: String,
    #[serde(default)]
    pub cd: Option<String>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub head_lines: Option<usize>,
    #[serde(default)]
    pub tail_lines: Option<usize>,
}

/// Output returned by the `terminal` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TerminalOutput {
    pub command: String,
    pub exit_code: Option<i32>,
    pub output: String,
    pub timed_out: bool,
    pub duration_ms: u64,
}

/// Executes a shell command safely, evaluating hardcoded security denials, permissions, and anti-hang sanitization.
pub fn execute_terminal(
    env: &AgentEnvironment,
    permission_engine: Option<&PermissionEngine>,
    command: &str,
    cd: Option<&str>,
    timeout_ms: Option<u64>,
    head_lines: Option<usize>,
    tail_lines: Option<usize>,
) -> Result<TerminalOutput, String> {
    // 1. Unbypassable hardcoded security denial check
    if let Some(reason) = HardcodedSecurityRules::check_command(command) {
        return Err(format!("Hardcoded security denial: {}", reason));
    }

    // 2. Permission Engine evaluation (if provided)
    if let Some(engine) = permission_engine {
        match engine.evaluate("terminal", command) {
            ToolPermissionDecision::Deny { reason } => {
                return Err(format!("Permission denied: {}", reason));
            }
            ToolPermissionDecision::Confirm { reason, .. } => {
                log::info!("Command requires confirmation: {}", reason);
            }
            ToolPermissionDecision::Allow => {}
        }
    }

    // 3. Resolve working directory inside workspace jail
    let working_dir = if let Some(sub_dir) = cd {
        env.jail
            .validate_read(Path::new(sub_dir))
            .map_err(|e| format!("Security denial: {}", e))?
    } else {
        env.primary_root()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."))
    };

    // 4. Configure options and execute via ProcessRunner
    let timeout = Duration::from_millis(timeout_ms.unwrap_or(60_000));
    let opts = ProcessExecutionOptions {
        command: command.to_string(),
        cwd: working_dir,
        timeout,
        head_lines,
        tail_lines,
        env_overrides: std::collections::HashMap::new(),
    };

    let result = env.runner.execute(opts, None, None)?;

    Ok(TerminalOutput {
        command: command.to_string(),
        exit_code: result.exit_code,
        output: result.combined_output,
        timed_out: result.timed_out,
        duration_ms: result.duration.as_millis() as u64,
    })
}

/// Returns the JSON Schema specification for `terminal`.
pub fn terminal_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "terminal",
        "description": "Executes a shell command in the project workspace with timeout and security protection.\n\nANTI-HANG INSTRUCTIONS:\n- For read-only git calls, always insert `--no-pager` (e.g. `git --no-pager log -n 5`).\n- Commands that may launch an editor must be prefixed with `GIT_EDITOR=true`.\n- Never generate shell substitutions: `$VAR`, `$(...)`, or backticks.\n\n<example>\n{\"tool\": \"terminal\", \"command\": \"cargo check -p agent\"}\n</example>",
        "parameters": {
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "The command string to execute."
                },
                "cd": {
                    "type": "string",
                    "description": "Optional directory within the project workspace to execute the command from."
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds (default 60000)."
                },
                "head_lines": {
                    "type": "integer",
                    "description": "Limit returned output to first N lines."
                },
                "tail_lines": {
                    "type": "integer",
                    "description": "Limit returned output to last N lines."
                }
            },
            "required": ["command"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_execute_terminal_denies_hardcoded_wipe() {
        let dir = tempdir().unwrap();
        let env = AgentEnvironment::new([dir.path()]);

        let res = execute_terminal(&env, None, "rm -rf /", None, None, None, None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Hardcoded security denial"));
    }

    #[test]
    fn test_execute_terminal_benign_command() {
        let dir = tempdir().unwrap();
        let env = AgentEnvironment::new([dir.path()]);

        let res = execute_terminal(&env, None, "echo 'hello terminal'", None, None, None, None).unwrap();
        assert_eq!(res.exit_code, Some(0));
        assert!(res.output.contains("hello terminal"));
        assert!(!res.timed_out);
    }
}
