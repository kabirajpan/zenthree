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

/// Checks if a command is a project scaffolding or generator command that creates initial project boilerplate.
pub fn is_scaffolding_command(cmd: &str) -> bool {
    let lower = cmd.trim().to_lowercase();
    let is_scaffold_action = lower.contains("create")
        || lower.contains("init")
        || lower.contains("startproject")
        || lower.contains("new ")
        || lower.starts_with("new ");

    let is_scaffold_tool = lower.contains("vite")
        || lower.contains("react")
        || lower.contains("vue")
        || lower.contains("svelte")
        || lower.contains("next")
        || lower.contains("nuxt")
        || lower.contains("astro")
        || lower.contains("cargo new")
        || lower.contains("cargo init")
        || lower.contains("go mod init")
        || lower.contains("npm init")
        || lower.contains("npm create")
        || lower.contains("bun create")
        || lower.contains("pnpm create")
        || lower.contains("yarn create")
        || lower.contains("flutter create")
        || lower.contains("django-admin")
        || lower.contains("rails new")
        || lower.contains("dotnet new")
        || lower.contains("composer create-project")
        || lower.contains("poetry new")
        || lower.contains("poetry init")
        || lower.contains("create-vite")
        || lower.contains("create-react-app")
        || lower.contains("create-next-app");

    is_scaffold_action && is_scaffold_tool
}

/// Scans the workspace directory to discover scaffolded project directories and key entry files.
pub fn detect_scaffolded_project_files(workspace: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let manifests = [
        "package.json", "Cargo.toml", "pyproject.toml", "go.mod", "pubspec.yaml",
        "CMakeLists.txt", "pom.xml", "build.gradle", "deno.json",
    ];
    let key_files = [
        "App.jsx", "App.tsx", "App.vue", "App.svelte", "main.jsx", "main.tsx",
        "index.html", "main.rs", "main.py", "app.py", "main.go", "main.dart",
        "index.js", "index.ts",
    ];

    if let Ok(entries) = std::fs::read_dir(workspace) {
        for entry in entries.flatten() {
            let path = entry.path();
            let fname = entry.file_name().to_string_lossy().to_string();
            if fname == "target" || fname == "node_modules" || fname == ".git" {
                continue;
            }
            if path.is_dir() {
                let has_manifest = manifests.iter().any(|m| path.join(m).is_file());
                if has_manifest {
                    let mut stack = vec![(path.clone(), 0usize)];
                    while let Some((dir, depth)) = stack.pop() {
                        if depth > 4 || found.len() >= 25 { break; }
                        if let Ok(sub_entries) = std::fs::read_dir(&dir) {
                            for se in sub_entries.flatten() {
                                let sp = se.path();
                                let sname = se.file_name().to_string_lossy().to_string();
                                if sname == "node_modules" || sname == "target" || sname == ".git" {
                                    continue;
                                }
                                if sp.is_dir() {
                                    stack.push((sp, depth + 1));
                                } else if sp.is_file() {
                                    let rel = sp.strip_prefix(workspace).unwrap_or(&sp).to_string_lossy().to_string();
                                    if manifests.iter().any(|m| sname.eq_ignore_ascii_case(m))
                                        || key_files.iter().any(|k| sname.eq_ignore_ascii_case(k))
                                    {
                                        found.push(rel);
                                    }
                                }
                            }
                        }
                    }
                }
            } else if path.is_file() {
                if manifests.iter().any(|m| fname.eq_ignore_ascii_case(m))
                    || key_files.iter().any(|k| fname.eq_ignore_ascii_case(k))
                {
                    found.push(fname);
                }
            }
        }
    }
    found
}

pub const PROJECT_MANIFEST_NAMES: &[&str] = &[
    "package.json", "Cargo.toml", "pyproject.toml", "go.mod", "CMakeLists.txt",
    "pubspec.yaml", "pom.xml", "build.gradle", "build.gradle.kts", "deno.json",
    "composer.json", "Gemfile", "Makefile", "justfile",
];

/// Walks upward from start_path looking for the nearest project manifest file.
/// Stops when it hits workspace_root or filesystem root.
pub fn find_nearest_manifest_dir(start_path: &Path, workspace_root: &Path) -> Option<std::path::PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent().map(|p| p.to_path_buf())
    } else {
        Some(start_path.to_path_buf())
    };

    while let Some(dir) = current {
        for manifest in PROJECT_MANIFEST_NAMES {
            if dir.join(manifest).is_file() {
                return Some(dir);
            }
        }
        if dir == workspace_root || !dir.starts_with(workspace_root) {
            break;
        }
        current = dir.parent().map(|p| p.to_path_buf());
    }

    None
}

/// Resolves the effective execution working directory for terminal commands:
/// 1. Explicit requested_cwd (relative to workspace_root or absolute).
/// 2. Nearest manifest from active_target file/folder.
/// 3. If workspace has no root manifest, but contains a single subproject with a manifest, use that.
/// 4. Fallback: workspace_root.
pub fn resolve_command_cwd(
    workspace_root: &Path,
    requested_cwd: Option<&str>,
    active_target: Option<&Path>,
) -> std::path::PathBuf {
    if let Some(req) = requested_cwd {
        let trimmed = req.trim();
        if !trimmed.is_empty() {
            let p = Path::new(trimmed);
            let resolved = if p.is_absolute() {
                p.to_path_buf()
            } else {
                workspace_root.join(p)
            };
            if resolved.is_dir() {
                return resolved;
            }
        }
    }

    if let Some(target) = active_target {
        if let Some(dir) = find_nearest_manifest_dir(target, workspace_root) {
            return dir;
        }
    }

    // Single subproject heuristic: if workspace has no root manifest,
    // but contains a single subproject with a manifest (e.g. "react-calculator/"), use that!
    let has_root_manifest = PROJECT_MANIFEST_NAMES.iter().any(|m| workspace_root.join(m).is_file());
    if !has_root_manifest {
        if let Ok(entries) = std::fs::read_dir(workspace_root) {
            let mut subprojects = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if name == "node_modules" || name == "target" || name == ".git" {
                    continue;
                }
                if path.is_dir() {
                    let has_manifest = PROJECT_MANIFEST_NAMES.iter().any(|m| path.join(m).is_file());
                    if has_manifest {
                        subprojects.push(path);
                    }
                }
            }
            if subprojects.len() == 1 {
                return subprojects.remove(0);
            }
        }
    }

    workspace_root.to_path_buf()
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
            #[cfg(windows)]
            {
                let check = std::process::Command::new("tasklist")
                    .args(["/FI", &format!("PID eq {}", p.pid)])
                    .output();
                match check {
                    Ok(out) => {
                        let text = String::from_utf8_lossy(&out.stdout);
                        text.contains(&p.pid.to_string())
                    }
                    Err(_) => false,
                }
            }
            #[cfg(not(windows))]
            {
                let check = std::process::Command::new("kill")
                    .arg("-0")
                    .arg(p.pid.to_string())
                    .output();
                match check {
                    Ok(out) => out.status.success(),
                    Err(_) => false,
                }
            }
        });
    }
}

pub fn kill_active_process(pid: u32) -> bool {
    #[cfg(windows)]
    {
        let _ = std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(not(windows))]
    {
        let _ = std::process::Command::new("kill")
            .arg("-9")
            .arg(pid.to_string())
            .output();
    }
    if let Ok(mut procs) = ACTIVE_PROCESSES.lock() {
        procs.retain(|p| p.pid != pid);
    }
    true
}

/// Parses a command that starts with `cd ` or `cd` to extract the target directory and optional chained command.
/// Examples:
/// - `cd subfolder` -> Some(("subfolder", None))
/// - `cd subfolder && npm test` -> Some(("subfolder", Some("npm test")))
/// - `cd subfolder; npm test` -> Some(("subfolder", Some("npm test")))
pub fn parse_cd_statement(cmd: &str) -> Option<(String, Option<String>)> {
    let trimmed = cmd.trim();
    if trimmed == "cd" {
        return Some((".".to_string(), None));
    }
    if let Some(rest) = trimmed.strip_prefix("cd ") {
        let rest = rest.trim();
        if let Some(pos) = rest.find("&&").or_else(|| rest.find(';')) {
            let target = rest[..pos].trim().trim_matches(|c| c == '\'' || c == '"').to_string();
            let remainder = rest[pos..].trim_start_matches(|c| c == '&' || c == ';' || c == ' ').trim().to_string();
            let rem_opt = if remainder.is_empty() { None } else { Some(remainder) };
            return Some((target, rem_opt));
        } else {
            let target = rest.trim_matches(|c| c == '\'' || c == '"').to_string();
            return Some((target, None));
        }
    }
    None
}

pub fn execute_run_command(workspace: &Path, cmd: &str) -> AgentToolResult {
    execute_run_command_with_cwd(workspace, cmd, None)
}

pub fn execute_run_command_with_cwd(workspace: &Path, cmd: &str, cwd: Option<&str>) -> AgentToolResult {
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

    let effective_cwd = resolve_command_cwd(workspace, cwd, None);
    let rel_cwd_label = if effective_cwd != workspace {
        let rel = effective_cwd.strip_prefix(workspace).unwrap_or(&effective_cwd).to_string_lossy();
        Some(format!("[Working Directory: {}]\n", rel))
    } else {
        None
    };

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

    #[cfg(windows)]
    let mut cmd_builder = {
        let comspec = std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string());
        let mut builder = std::process::Command::new(comspec);
        builder.arg("/C").arg(trimmed).current_dir(&effective_cwd);
        builder
    };

    #[cfg(not(windows))]
    let mut cmd_builder = {
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/bash".to_string());
        let mut builder = std::process::Command::new(&shell);
        builder.arg("-c").arg(trimmed).current_dir(&effective_cwd);
        builder
    };

    // Augment PATH with common tool locations (bun, cargo, local bins)
    #[cfg(windows)]
    {
        let user_profile = std::env::var("USERPROFILE").unwrap_or_default();
        let path_var = std::env::var("PATH").unwrap_or_default();
        let cargo_bin = format!("{}\\.cargo\\bin", user_profile);
        let mut augmented = path_var;
        if !augmented.contains(&cargo_bin) {
            augmented = format!("{};{}", cargo_bin, augmented);
        }
        cmd_builder.env("PATH", augmented);
    }

    #[cfg(not(windows))]
    {
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
    }

    cmd_builder.env("CI", "true");
    cmd_builder.env("TERM", "dumb");
    cmd_builder.env("DEBIAN_FRONTEND", "noninteractive");
    cmd_builder.stdin(std::process::Stdio::null());

    match cmd_builder
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(mut child) => {
            let stdout = child.stdout.take();
            let stderr = child.stderr.take();

            let output_buffer = std::sync::Arc::new(std::sync::Mutex::new(String::new()));
            let out_clone1 = std::sync::Arc::clone(&output_buffer);
            let out_clone2 = std::sync::Arc::clone(&output_buffer);

            // Read stdout concurrently to prevent pipe buffer deadlocks
            let h_out = std::thread::spawn(move || {
                if let Some(mut r) = stdout {
                    let mut buf = [0u8; 4096];
                    while let Ok(n) = r.read(&mut buf) {
                        if n == 0 { break; }
                        let text = String::from_utf8_lossy(&buf[..n]);
                        if let Ok(mut lock) = out_clone1.lock() {
                            if lock.len() < 30_000 {
                                lock.push_str(&text);
                            }
                        }
                    }
                }
            });

            // Read stderr concurrently
            let h_err = std::thread::spawn(move || {
                if let Some(mut r) = stderr {
                    let mut buf = [0u8; 4096];
                    while let Ok(n) = r.read(&mut buf) {
                        if n == 0 { break; }
                        let text = String::from_utf8_lossy(&buf[..n]);
                        if let Ok(mut lock) = out_clone2.lock() {
                            if lock.len() < 30_000 {
                                lock.push_str(&text);
                            }
                        }
                    }
                }
            });

            let start = std::time::Instant::now();
            let timeout = if is_dev_server {
                std::time::Duration::from_secs(3)
            } else {
                std::time::Duration::from_secs(45)
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
                let _ = h_out.join();
                let _ = h_err.join();

                let combined = output_buffer.lock().map(|l| l.clone()).unwrap_or_default();
                let mut final_output = if combined.trim().is_empty() {
                    if status.success() {
                        "Command exited successfully with code 0 (no output)".to_string()
                    } else {
                        format!("Command exited with status {}", status)
                    }
                } else {
                    combined
                };

                if status.success() && is_scaffolding_command(trimmed) {
                    let mut files = detect_scaffolded_project_files(workspace);
                    if files.is_empty() && effective_cwd != workspace {
                        files = detect_scaffolded_project_files(&effective_cwd);
                    }
                    let mut scaffold_notice = String::from("\n\n[PROJECT SCAFFOLDING DETECTED]\n");
                    if !files.is_empty() {
                        scaffold_notice.push_str("Discovered generated project files:\n");
                        for f in &files {
                            scaffold_notice.push_str(&format!("  - {}\n", f));
                        }
                    }
                    scaffold_notice.push_str(
                        "\nCRITICAL MANDATORY NEXT STEPS (DO NOT STOP HERE):\n\
                        1. The initial project structure was scaffolded, but this is only the empty starter template!\n\
                        2. If dependencies are not yet installed, run the installation command (e.g. `cd <folder> && npm install` or `cargo build`).\n\
                        3. OVERWRITE the entry component or source file (e.g. `### File: <folder>/src/App.jsx` or `<folder>/src/main.rs`) with the FULL, REAL, BEAUTIFULLY STYLED implementation of the requested project (e.g. calculator buttons, state logic, arithmetic, styling).\n\
                        4. Do NOT leave the project as default starter boilerplate! Provide the complete application."
                    );
                    final_output.push_str(&scaffold_notice);
                }

                if let Some(ref label) = rel_cwd_label {
                    final_output = format!("{}{}", label, final_output);
                }

                AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: status.success(),
                    output: final_output.trim_end().to_string(),
                }
            } else if is_dev_server {
                // Continuous dev server still running as expected
                let pid = child.id();
                let accumulated = output_buffer.lock().map(|l| l.clone()).unwrap_or_default();
                if let Ok(mut procs) = ACTIVE_PROCESSES.lock() {
                    procs.push(ActiveAiProcess {
                        command: trimmed.to_string(),
                        pid,
                        is_server: true,
                    });
                }

                let out_msg = if accumulated.trim().is_empty() {
                    format!("Command `{}` started successfully and is running in the background (PID {}).", trimmed, pid)
                } else {
                    format!("Command `{}` started in background (PID {}).\nInitial terminal output:\n{}", trimmed, pid, accumulated.trim())
                };
                let full_out = if let Some(ref label) = rel_cwd_label {
                    format!("{}{}", label, out_msg)
                } else {
                    out_msg
                };

                AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: true,
                    output: full_out,
                }
            } else {
                // Non-server command timed out - terminate it so it does not hang
                let _ = child.kill();
                let accumulated = output_buffer.lock().map(|l| l.clone()).unwrap_or_default();
                let out_msg = if accumulated.trim().is_empty() {
                    format!("Command `{}` timed out after 45 seconds and was terminated.", trimmed)
                } else {
                    format!("Command `{}` timed out after 45 seconds. Terminal output captured:\n{}", trimmed, accumulated.trim())
                };
                let full_out = if let Some(ref label) = rel_cwd_label {
                    format!("{}{}", label, out_msg)
                } else {
                    out_msg
                };
                AgentToolResult {
                    tool_name: "run_command".to_string(),
                    ok: false,
                    output: full_out,
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
