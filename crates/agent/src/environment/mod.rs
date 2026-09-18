pub mod workspace_jail;
pub mod protected_paths;
pub mod secret_scrubber;
pub mod env_sanitizer;
pub mod process_runner;
pub mod host_detection;

pub use workspace_jail::{WorkspaceJail, SecurityViolation};
pub use protected_paths::ProtectedPathGuard;
pub use secret_scrubber::{SecretScrubber, REDACTED_SECRET};
pub use env_sanitizer::EnvironmentSanitizer;
pub use process_runner::{ProcessRunner, ProcessExecutionOptions, ProcessExecutionResult};
pub use host_detection::{HostEnvironment, ToolchainInfo};

use crate::types::cancellation::CancellationToken;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Unified agent environment providing workspace isolation, invariant protection,
/// secret scrubbing, and safe subprocess execution.
#[derive(Debug, Clone)]
pub struct AgentEnvironment {
    pub jail: WorkspaceJail,
    pub scrubber: SecretScrubber,
    pub runner: ProcessRunner,
    pub host: HostEnvironment,
}

impl AgentEnvironment {
    /// Creates a new agent environment jailed to the specified workspace roots.
    pub fn new(roots: impl IntoIterator<Item = impl AsRef<Path>>) -> Self {
        Self {
            jail: WorkspaceJail::new(roots),
            scrubber: SecretScrubber::new(),
            runner: ProcessRunner::new(),
            host: HostEnvironment::detect(),
        }
    }

    /// Validates a path for reading. Returns error if outside jail or protected.
    pub fn validate_read(&self, path: &Path) -> Result<PathBuf, SecurityViolation> {
        self.jail.validate_read(path)
    }

    /// Validates a path for writing. Strictly enforces workspace jail and invariant protections (.git/, secrets).
    pub fn validate_write(&self, path: &Path) -> Result<PathBuf, SecurityViolation> {
        let resolved = self.jail.validate_write(path)?;

        // Invariant guard check: strictly forbid writes to .git, .env, or credential files
        if ProtectedPathGuard::is_write_blocked(&resolved) {
            return Err(SecurityViolation::InvalidPath {
                reason: format!("Writing to protected path '{}' is strictly prohibited by security rules", resolved.display()),
            });
        }

        Ok(resolved)
    }

    /// Executes a shell command inside the environment with timeouts and line-clipping.
    pub fn run_command(
        &self,
        cmd: &str,
        timeout: Duration,
        head_lines: Option<usize>,
        tail_lines: Option<usize>,
        cancel: Option<&CancellationToken>,
    ) -> Result<ProcessExecutionResult, String> {
        let cwd = self.primary_root()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        let opts = ProcessExecutionOptions {
            command: cmd.to_string(),
            cwd,
            timeout,
            head_lines,
            tail_lines,
            env_overrides: std::collections::HashMap::new(),
        };

        self.runner.execute(opts, cancel, None)
    }

    /// Scrubs any secrets or private keys from output strings.
    pub fn scrub_output(&self, text: &str) -> String {
        self.scrubber.scrub(text)
    }

    /// Primary workspace root directory.
    pub fn primary_root(&self) -> Option<&Path> {
        self.jail.primary_root()
    }

    /// Formats detected host environment into system prompt markdown.
    pub fn format_prompt_block(&self) -> String {
        self.host.format_prompt_block()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_environment_write_to_git_blocked() {
        let temp_dir = std::env::temp_dir().join("zenthree_test_env_all");
        let _ = fs::create_dir_all(&temp_dir);

        let env = AgentEnvironment::new(vec![&temp_dir]);

        // Writing regular file is allowed
        let valid_file = temp_dir.join("src/lib.rs");
        assert!(env.validate_write(&valid_file).is_ok());

        // Writing to .git inside workspace is strictly blocked by ProtectedPathGuard
        let git_file = temp_dir.join(".git/config");
        assert!(env.validate_write(&git_file).is_err());

        // Writing to .env is blocked
        let env_file = temp_dir.join(".env");
        assert!(env.validate_write(&env_file).is_err());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_environment_run_command_and_prompt() {
        let env = AgentEnvironment::new(vec!["."]);
        let prompt_block = env.format_prompt_block();
        assert!(prompt_block.contains("## System Environment"));

        let result = env.run_command("echo 123", Duration::from_secs(5), None, None, None).unwrap();
        assert!(result.combined_output.contains("123"));
    }
}
