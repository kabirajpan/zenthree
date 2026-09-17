use std::io::Read;
use std::path::Path;
use super::AgentToolResult;

/// Checks if a command contains dangerous or destructive operations that require explicit user permission.
pub fn is_dangerous_command(cmd: &str) -> Option<&'static str> {
    let lower = cmd.trim().to_lowercase();

    // Privilege escalation
    if lower.starts_with("sudo ") || lower == "sudo"
        || lower.starts_with("doas ") || lower == "doas"
        || lower.starts_with("su ") || lower == "su"
    {
        return Some("requires root / superuser privileges");
    }

    // Dangerous system filesystem destruction
    if lower.contains("rm -rf /") || lower.contains("rm -fr /")
        || lower.contains("rm -rf ~") || lower.contains("rm -rf $home")
        || lower.contains("rm -rf *") || lower.contains("rm -fr *")
        || lower.contains("rm -rf .") || lower.contains("rm -fr .")
    {
        return Some("recursive deletion of root, home, or entire workspace directory");
    }

    // Disk formatting and raw storage device manipulation
    if lower.contains("mkfs")
        || lower.contains("dd if=")
        || lower.starts_with("fdisk ") || lower == "fdisk"
        || lower.starts_with("parted ") || lower == "parted"
        || lower.starts_with("gdisk ") || lower == "gdisk"
    {
        return Some("disk formatting or raw storage device partition modification");
    }

    // Power operations / system control
    if lower == "shutdown" || lower.starts_with("shutdown ")
        || lower == "reboot" || lower.starts_with("reboot ")
        || lower == "poweroff" || lower.starts_with("poweroff ")
        || lower.starts_with("init 0") || lower.starts_with("init 6")
    {
        return Some("system power modification or reboot operation");
    }

    // Malicious shell exploits
    if lower.contains(":(){ :|:& };:") {
        return Some("fork bomb denial of service script");
    }

    // Destructive unrecoverable Git clean
    if lower.contains("git clean -fdx") || lower.contains("git clean -f -d -x") {
        return Some("irreversible deletion of untracked files and working tree state");
    }

    None
}

/// Checks if a command is a continuous dev server or watcher that stays running in the background.
/// Batch/setup/build/install/test commands must NEVER be classified as continuous servers.
pub fn is_continuous_server_command(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();
    let is_batch = lower.contains("create")
        || lower.contains("init")
        || lower.contains("install")
        || lower.contains("add ")
        || lower.contains("build")
        || lower.contains("test")
        || lower.contains("check")
        || lower.contains("lint")
        || lower.contains("clean");

    if is_batch {
        return false;
    }

    lower.contains("run dev")
        || lower.ends_with(" dev")
        || lower.contains("npm start")
        || lower.contains("bun start")
        || lower.contains("yarn start")
        || lower.contains("pnpm start")
        || lower.contains("http.server")
        || lower.contains("vite preview")
        || (lower.contains("vite") && !lower.contains("create"))
}

#[derive(Debug, Clone)]
pub struct ActiveAiProcess {
    pub command: String,
    pub pid: u32,
    pub is_server: bool,
}

static ACTIVE_PROCESSES: std::sync::Mutex<Vec<ActiveAiProcess>> = std::sync::Mutex::new(Vec::new());

pub fn prune_active_processes() {
    if let Ok(mut procs) = ACTIVE_PROCESSES.lock() {
        procs.retain(|p| {
            let check = std::process::Command::new("kill")
                .arg("-0")
                .arg(p.pid.to_string())
                .output();
            match check {
                Ok(out) => out.status.success(),
                Err(_) => false,
            }
        });
    }
}

pub fn kill_active_process(pid: u32) -> bool {
    let _ = std::process::Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .output();
    if let Ok(mut procs) = ACTIVE_PROCESSES.lock() {
        procs.retain(|p| p.pid != pid);
    }
    true
}

pub fn execute_run_command(workspace: &Path, cmd: &str) -> AgentToolResult {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: "Command cannot be empty".to_string(),
        };
    }

    if let Some(reason) = is_dangerous_command(trimmed) {
        return AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: format!(
                "Permission required: Command `{}` was blocked because it {}. Please ask the user for explicit permission before running this command.",
                trimmed, reason
            ),
        };
    }

    prune_active_processes();
    let lower = trimmed.to_lowercase();
    let is_dev_server = is_continuous_server_command(trimmed);

    // Rule 1 & 2: Check if any command of the same type or another dev server is already running
    if let Ok(procs) = ACTIVE_PROCESSES.lock() {
        for active in procs.iter() {
            let active_trimmed = active.command.trim();
            let is_same_cmd = active_trimmed == trimmed || active_trimmed.to_lowercase() == lower;

            if is_same_cmd {
                return AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: false,
                    output: format!(
                        "Command `{}` is already actively running in the background (PID {}). Duplicate execution was disabled: only one instance can run.",
                        trimmed, active.pid
                    ),
                };
            }

            if is_dev_server && active.is_server {
                return AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: false,
                    output: format!(
                        "Disabled `{}`: A dev server `{}` is already actively running in the background (PID {}). Stop the existing server before starting another.",
                        trimmed, active.command, active.pid
                    ),
                };
            }
        }
    }

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
    let mut cmd_builder = std::process::Command::new(&shell);
    cmd_builder.arg("-c").arg(trimmed).current_dir(workspace);

    // Augment PATH with common tool locations (bun, cargo, local bins)
    let home = std::env::var("HOME").unwrap_or_default();
    let path_var = std::env::var("PATH").unwrap_or_default();
    let bun_bin = format!("{}/.bun/bin", home);
    let cargo_bin = format!("{}/.cargo/bin", home);
    let local_bin = format!("{}/.local/bin", home);
    let mut augmented = path_var;
    if !augmented.contains(&bun_bin) {
        augmented = format!("{}:{}", bun_bin, augmented);
    }
    if !augmented.contains(&cargo_bin) {
        augmented = format!("{}:{}", cargo_bin, augmented);
    }
    if !augmented.contains(&local_bin) {
        augmented = format!("{}:{}", local_bin, augmented);
    }
    cmd_builder.env("PATH", augmented);

    match cmd_builder
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            let start = std::time::Instant::now();
            let timeout = if is_dev_server {
                std::time::Duration::from_secs(4)
            } else {
                std::time::Duration::from_secs(120)
            };

            let mut exited_status = None;
            while start.elapsed() < timeout {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        exited_status = Some(status);
                        break;
                    }
                    Ok(None) => {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    Err(_) => break,
                }
            }

            if let Some(status) = exited_status {
                let mut stdout_str = String::new();
                let mut stderr_str = String::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_string(&mut stdout_str);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_string(&mut stderr_str);
                }

                let mut combined = String::new();
                if !stdout_str.trim().is_empty() {
                    combined.push_str(&stdout_str);
                }
                if !stderr_str.trim().is_empty() {
                    if !combined.is_empty() {
                        combined.push('\n');
                    }
                    combined.push_str(&stderr_str);
                }

                if combined.is_empty() {
                    combined = if status.success() {
                        "Command exited successfully with code 0 (no output)".to_string()
                    } else {
                        format!("Command exited with status {}", status)
                    };
                }

                AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: status.success(),
                    output: combined.trim_end().to_string(),
                }
            } else if is_dev_server {
                // Continuous dev server still running as expected
                let pid = child.id();
                if let Ok(mut procs) = ACTIVE_PROCESSES.lock() {
                    procs.push(ActiveAiProcess {
                        command: trimmed.to_string(),
                        pid,
                        is_server: true,
                    });
                }

                AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: true,
                    output: format!("Command `{}` started successfully and is running in the background (PID {}).", trimmed, pid),
                }
            } else {
                // Non-server command timed out after 120s - terminate it so it does not hang
                let _ = child.kill();
                AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: false,
                    output: format!("Command `{}` timed out after 120 seconds and was terminated.", trimmed),
                }
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: format!("Failed to spawn command `{}`: {}", trimmed, e),
        },
    }
}
