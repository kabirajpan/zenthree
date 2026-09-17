use std::path::Path;
use super::AgentToolResult;

pub fn execute_run_command(workspace: &Path, cmd: &str) -> AgentToolResult {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: "Command cannot be empty".to_string(),
        };
    }

    let lower = trimmed.to_lowercase();
    if lower.contains("rm -rf")
        || lower.contains("mkfs")
        || lower.contains("dd if=")
        || lower.contains(":(){ :|:& };:")
        || lower.contains("shutdown")
        || lower.contains("reboot")
        || lower.starts_with("sudo")
    {
        return AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: format!("Command blocked for safety reasons: `{}`", trimmed),
        };
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(trimmed)
        .current_dir(workspace)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let success = out.status.success();

            let mut combined = String::new();
            if !stdout.trim().is_empty() {
                combined.push_str(&stdout);
            }
            if !stderr.trim().is_empty() {
                if !combined.is_empty() {
                    combined.push('\n');
                }
                combined.push_str(&stderr);
            }

            if combined.is_empty() {
                combined = if success {
                    "Command exited successfully with code 0 (no output)".to_string()
                } else {
                    format!("Command exited with status {}", out.status)
                };
            }

            AgentToolResult {
                tool_name: "run_command".to_string(),
                ok: success,
                output: combined.trim_end().to_string(),
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: format!("Failed to spawn command `{}`: {}", trimmed, e),
        },
    }
}
