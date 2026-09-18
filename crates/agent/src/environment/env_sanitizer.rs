use std::collections::HashMap;

/// Environment variables sanitizer that prevents subprocesses from hanging on interactive prompts,
/// pagers, or text editors (mirroring Zed's terminal sanitization rules).
#[derive(Debug, Clone, Default)]
pub struct EnvironmentSanitizer;

impl EnvironmentSanitizer {
    /// Returns default safe environment variables for subprocess execution.
    pub fn sanitized_env_vars() -> HashMap<String, String> {
        let mut vars = HashMap::new();

        // Prevent commands from paging output (e.g. git log, man, less)
        vars.insert("PAGER".to_string(), "cat".to_string());

        // Prevent commands from spawning interactive text editors
        vars.insert("GIT_EDITOR".to_string(), "true".to_string());
        vars.insert("EDITOR".to_string(), "true".to_string());
        vars.insert("VISUAL".to_string(), "true".to_string());

        // Non-interactive mode for common CLI toolchains
        vars.insert("CI".to_string(), "1".to_string());
        vars.insert("DEBIAN_FRONTEND".to_string(), "noninteractive".to_string());
        vars.insert("TERM".to_string(), "xterm-256color".to_string());

        // Node / npm / python non-interactive flags
        vars.insert("NPM_CONFIG_YES".to_string(), "true".to_string());
        vars.insert("PYTHONUNBUFFERED".to_string(), "1".to_string());

        vars
    }

    /// Sanitizes git command arguments to prevent hanging on pagers or locking repositories.
    pub fn sanitize_git_command(command: &str) -> String {
        let trimmed = command.trim();
        if !trimmed.starts_with("git ") && trimmed != "git" {
            return command.to_string();
        }

        let mut parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() <= 1 {
            return command.to_string();
        }

        let subcmd = parts[1];
        let read_only_git_subcmds = ["log", "diff", "show", "blame", "status", "branch", "tag"];

        // If command is a read-only git command, ensure `--no-pager` is inserted right after `git`
        if read_only_git_subcmds.contains(&subcmd) && !command.contains("--no-pager") {
            parts.insert(1, "--no-pager");
        }

        // If command is git status, prefer `--no-optional-locks` to avoid mutating repository metadata
        if subcmd == "status" && !command.contains("--no-optional-locks") {
            parts.insert(parts.len(), "--no-optional-locks");
        }

        parts.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitized_env_contains_anti_hang_guards() {
        let env = EnvironmentSanitizer::sanitized_env_vars();
        assert_eq!(env.get("PAGER").unwrap(), "cat");
        assert_eq!(env.get("GIT_EDITOR").unwrap(), "true");
        assert_eq!(env.get("EDITOR").unwrap(), "true");
        assert_eq!(env.get("CI").unwrap(), "1");
    }

    #[test]
    fn test_git_command_sanitization() {
        // Read-only git commands get --no-pager inserted
        let sanitized_log = EnvironmentSanitizer::sanitize_git_command("git log -n 5");
        assert!(sanitized_log.contains("git --no-pager log"));

        let sanitized_diff = EnvironmentSanitizer::sanitize_git_command("git diff HEAD~1");
        assert!(sanitized_diff.contains("git --no-pager diff"));

        // git status gets --no-optional-locks to avoid repo lock mutations
        let sanitized_status = EnvironmentSanitizer::sanitize_git_command("git status");
        assert!(sanitized_status.contains("--no-optional-locks"));

        // Non-git commands remain unmodified
        let cargo = EnvironmentSanitizer::sanitize_git_command("cargo test --workspace");
        assert_eq!(cargo, "cargo test --workspace");
    }
}
