use std::path::Path;
use crate::types::ProjectEnvironment;
use crate::tools::environment::DetectorAgent;

pub fn filter_compiler_errors(output: &str) -> String {
    let mut lines = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Checking ")
            || trimmed.starts_with("Compiling ")
            || trimmed.starts_with("Finished ")
            || trimmed.starts_with("Downloading ")
            || trimmed.starts_with("Downloaded ")
            || trimmed.is_empty()
        {
            continue;
        }

        if trimmed.contains("error[")
            || trimmed.contains("error:")
            || trimmed.contains("-->")
            || trimmed.contains("error TS")
            || trimmed.contains("::: ")
            || trimmed.contains("help:")
            || trimmed.contains("warning:")
        {
            lines.push(trimmed.to_string());
            if lines.len() >= 20 {
                break;
            }
        }
    }

    if lines.is_empty() {
        for line in output.lines().filter(|l| !l.trim().is_empty()).take(10) {
            lines.push(line.trim().to_string());
        }
    }

    if lines.is_empty() {
        "Unknown compiler error".to_string()
    } else {
        lines.join("\n")
    }
}

pub fn run_project_compiler_check(workspace_root: &Path) -> Option<Result<(), String>> {
    let env = DetectorAgent::detect(workspace_root);
    run_project_compiler_check_with_env(workspace_root, &env)
}

pub fn run_project_compiler_check_with_env(
    workspace_root: &Path,
    env: &ProjectEnvironment,
) -> Option<Result<(), String>> {
    let cmd_parts = env.compiler_check_cmd.as_ref()?;
    if cmd_parts.is_empty() {
        return None;
    }

    let program = &cmd_parts[0];
    let args = &cmd_parts[1..];

    let mut command = std::process::Command::new(program);
    command.args(args).current_dir(workspace_root);

    let res = command.output();
    if let Ok(out) = res {
        if out.status.success() {
            Some(Ok(()))
        } else {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            Some(Err(filter_compiler_errors(&combined)))
        }
    } else {
        None
    }
}

pub fn run_project_tests(workspace_root: &Path) -> Option<Result<(), String>> {
    let env = DetectorAgent::detect(workspace_root);
    let cmd_parts = env.test_cmd.as_ref()?;
    if cmd_parts.is_empty() {
        return None;
    }

    let program = &cmd_parts[0];
    let args = &cmd_parts[1..];

    let mut command = std::process::Command::new(program);
    command.args(args).current_dir(workspace_root);

    let res = command.output();
    if let Ok(out) = res {
        if out.status.success() {
            Some(Ok(()))
        } else {
            let combined = format!(
                "{}\n{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
            Some(Err(combined))
        }
    } else {
        None
    }
}

pub fn build_self_healing_prompt(
    original_goal: &str,
    compiler_errors: &str,
    target_file: Option<&str>,
) -> String {
    let file_hint = if let Some(f) = target_file {
        format!("in `{}`", f)
    } else {
        "in the target files".to_string()
    };

    format!(
        "COMPILER ERROR SELF-HEALING REQUEST:\n\
        The previous code generated for `{}` failed compiler check.\n\n\
        Compiler Diagnostics:\n\
        ```\n\
        {}\n\
        ```\n\n\
        Instructions:\n\
        1. Analyze and fix all compiler errors shown above {}.\n\
        2. Provide the corrected complete code for each affected file using `### File: <relative_path>` tags and fenced code blocks.\n\
        3. Do not omit any code or use placeholders. Return working, clean code.",
        original_goal, compiler_errors, file_hint
    )
}
