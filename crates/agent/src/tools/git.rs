use std::path::Path;
use super::AgentToolResult;

pub fn execute_git_status(workspace: &Path) -> AgentToolResult {
    let output = std::process::Command::new("git")
        .args(["status", "--short", "--branch"])
        .current_dir(workspace)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let ok = out.status.success();
            let text = if ok {
                if stdout.trim().is_empty() {
                    "Working tree clean (no modified or untracked files)".to_string()
                } else {
                    stdout.trim().to_string()
                }
            } else {
                format!("git status failed: {}", stderr.trim())
            };
            AgentToolResult {
                tool_name: "git_status".to_string(),
                ok,
                output: text,
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "git_status".to_string(),
            ok: false,
            output: format!("Failed to run git status: {}", e),
        },
    }
}

pub fn execute_git_diff(workspace: &Path, staged_only: bool) -> AgentToolResult {
    let mut cmd = std::process::Command::new("git");
    cmd.arg("diff");
    if staged_only {
        cmd.arg("--staged");
    }
    cmd.current_dir(workspace);

    match cmd.output() {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let ok = out.status.success();
            let text = if ok {
                if stdout.trim().is_empty() {
                    "No changes found in diff".to_string()
                } else {
                    let s = stdout.trim();
                    if s.len() > 6000 {
                        format!("{}...\n[Diff truncated: {} total chars]", &s[..6000], s.len())
                    } else {
                        s.to_string()
                    }
                }
            } else {
                format!("git diff failed: {}", stderr.trim())
            };
            AgentToolResult {
                tool_name: "git_diff".to_string(),
                ok,
                output: text,
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "git_diff".to_string(),
            ok: false,
            output: format!("Failed to run git diff: {}", e),
        },
    }
}
