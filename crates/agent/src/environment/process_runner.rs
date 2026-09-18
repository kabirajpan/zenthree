use super::env_sanitizer::EnvironmentSanitizer;
use super::secret_scrubber::SecretScrubber;
use crate::types::cancellation::CancellationToken;
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Execution parameters for running a command in the environment.
#[derive(Debug, Clone)]
pub struct ProcessExecutionOptions {
    pub command: String,
    pub cwd: PathBuf,
    pub timeout: Duration,
    pub head_lines: Option<usize>,
    pub tail_lines: Option<usize>,
    pub env_overrides: HashMap<String, String>,
}

impl Default for ProcessExecutionOptions {
    fn default() -> Self {
        Self {
            command: String::new(),
            cwd: PathBuf::from("."),
            timeout: Duration::from_secs(60),
            head_lines: None,
            tail_lines: None,
            env_overrides: HashMap::new(),
        }
    }
}

/// The result of a process execution.
#[derive(Debug, Clone, Default)]
pub struct ProcessExecutionResult {
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub combined_output: String,
    pub clipped_output: String,
    pub duration: Duration,
    pub timed_out: bool,
    pub cancelled: bool,
}

/// Sandboxed process runner with timeouts, cancellation, secret scrubbing, and output clipping.
#[derive(Debug, Clone, Default)]
pub struct ProcessRunner {
    scrubber: SecretScrubber,
}

impl ProcessRunner {
    /// Creates a new process runner instance.
    pub fn new() -> Self {
        Self {
            scrubber: SecretScrubber::new(),
        }
    }

    /// Executes a shell command synchronously with timeout, cancellation, and streaming callback.
    pub fn execute(
        &self,
        opts: ProcessExecutionOptions,
        cancel_token: Option<&CancellationToken>,
        mut on_chunk: Option<&mut dyn FnMut(&str)>,
    ) -> Result<ProcessExecutionResult, String> {
        let start_time = Instant::now();
        let sanitized_cmd = EnvironmentSanitizer::sanitize_git_command(&opts.command);

        // Configure shell execution
        let mut cmd = if cfg!(windows) {
            let mut c = Command::new("cmd.exe");
            c.arg("/C").arg(&sanitized_cmd);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&sanitized_cmd);
            c
        };

        cmd.current_dir(&opts.cwd);
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        // Inject sanitized anti-hang environment variables
        for (k, v) in EnvironmentSanitizer::sanitized_env_vars() {
            cmd.env(k, v);
        }

        // Apply custom overrides
        for (k, v) in &opts.env_overrides {
            cmd.env(k, v);
        }

        let mut child = cmd.spawn().map_err(|e| {
            format!("Failed to spawn process for command '{}': {}", sanitized_cmd, e)
        })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        let stdout_lines = Arc::new(Mutex::new(Vec::new()));
        let stderr_lines = Arc::new(Mutex::new(Vec::new()));
        let combined_lines = Arc::new(Mutex::new(Vec::new()));

        let stdout_collector = {
            let stdout_lines = Arc::clone(&stdout_lines);
            let combined_lines = Arc::clone(&combined_lines);
            std::thread::spawn(move || {
                if let Some(pipe) = stdout {
                    let reader = BufReader::new(pipe);
                    for line in reader.lines().flatten() {
                        if let Ok(mut out) = stdout_lines.lock() {
                            out.push(line.clone());
                        }
                        if let Ok(mut comb) = combined_lines.lock() {
                            comb.push(line);
                        }
                    }
                }
            })
        };

        let stderr_collector = {
            let stderr_lines = Arc::clone(&stderr_lines);
            let combined_lines = Arc::clone(&combined_lines);
            std::thread::spawn(move || {
                if let Some(pipe) = stderr {
                    let reader = BufReader::new(pipe);
                    for line in reader.lines().flatten() {
                        if let Ok(mut err) = stderr_lines.lock() {
                            err.push(line.clone());
                        }
                        if let Ok(mut comb) = combined_lines.lock() {
                            comb.push(line);
                        }
                    }
                }
            })
        };

        let mut timed_out = false;
        let mut cancelled = false;
        let mut exit_code = None;

        // Poll process with timeout and cancellation check
        loop {
            if let Some(token) = cancel_token {
                if token.is_cancelled() {
                    cancelled = true;
                    let _ = child.kill();
                    break;
                }
            }

            if start_time.elapsed() >= opts.timeout {
                timed_out = true;
                let _ = child.kill();
                break;
            }

            match child.try_wait() {
                Ok(Some(status)) => {
                    exit_code = status.code();
                    break;
                }
                Ok(None) => {
                    std::thread::sleep(Duration::from_millis(50));
                }
                Err(e) => {
                    return Err(format!("Error waiting for process: {}", e));
                }
            }
        }

        let _ = stdout_collector.join();
        let _ = stderr_collector.join();

        let raw_stdout = stdout_lines.lock().unwrap().join("\n");
        let raw_stderr = stderr_lines.lock().unwrap().join("\n");
        let raw_combined = combined_lines.lock().unwrap().join("\n");

        // Secret scrub raw output strings
        let scrubbed_stdout = self.scrubber.scrub(&raw_stdout);
        let scrubbed_stderr = self.scrubber.scrub(&raw_stderr);
        let scrubbed_combined = self.scrubber.scrub(&raw_combined);

        // Apply line streaming callback if provided
        if let Some(ref mut cb) = on_chunk {
            cb(&scrubbed_combined);
        }

        // Apply head and tail output clipping for the LLM
        let clipped = Self::clip_lines(
            &scrubbed_combined,
            opts.head_lines,
            opts.tail_lines,
        );

        Ok(ProcessExecutionResult {
            exit_code,
            stdout: scrubbed_stdout,
            stderr: scrubbed_stderr,
            combined_output: scrubbed_combined,
            clipped_output: clipped,
            duration: start_time.elapsed(),
            timed_out,
            cancelled,
        })
    }

    /// Clips output lines according to head_lines and tail_lines bounds, preserving token budget.
    pub fn clip_lines(
        output: &str,
        head_lines: Option<usize>,
        tail_lines: Option<usize>,
    ) -> String {
        if head_lines.is_none() && tail_lines.is_none() {
            return output.to_string();
        }

        let lines: Vec<&str> = output.lines().collect();
        let total = lines.len();

        let head = head_lines.unwrap_or(0);
        let tail = tail_lines.unwrap_or(0);

        // If total lines are within bounds, return everything
        if total <= head + tail {
            return output.to_string();
        }

        let mut result: Vec<String> = Vec::new();
        if head > 0 {
            result.extend(lines[..head.min(total)].iter().map(|s| s.to_string()));
        }

        let omitted = total.saturating_sub(head + tail);
        if omitted > 0 {
            result.push(format!("\n[... {} lines omitted for context preservation ...]\n", omitted));
        }

        if tail > 0 && total > head {
            let tail_start = total.saturating_sub(tail).max(head);
            result.extend(lines[tail_start..].iter().map(|s| s.to_string()));
        }

        result.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_runner_echo() {
        let runner = ProcessRunner::new();
        let opts = ProcessExecutionOptions {
            command: "echo 'hello zenthree'".to_string(),
            cwd: PathBuf::from("."),
            timeout: Duration::from_secs(5),
            head_lines: None,
            tail_lines: None,
            env_overrides: HashMap::new(),
        };

        let res = runner.execute(opts, None, None).unwrap();
        assert_eq!(res.exit_code, Some(0));
        assert!(res.combined_output.contains("hello zenthree"));
        assert!(!res.timed_out);
        assert!(!res.cancelled);
    }

    #[test]
    fn test_process_runner_clips_lines() {
        let output = (1..=100).map(|i| format!("line {}", i)).collect::<Vec<_>>().join("\n");
        let clipped = ProcessRunner::clip_lines(&output, Some(5), Some(5));

        assert!(clipped.contains("line 1"));
        assert!(clipped.contains("line 5"));
        assert!(clipped.contains("line 96"));
        assert!(clipped.contains("line 100"));
        assert!(clipped.contains("lines omitted"));
    }

    #[test]
    fn test_process_runner_timeout() {
        let runner = ProcessRunner::new();
        let cmd = if cfg!(windows) {
            "powershell -Command Start-Sleep -Seconds 5"
        } else {
            "sleep 5"
        };

        let opts = ProcessExecutionOptions {
            command: cmd.to_string(),
            cwd: PathBuf::from("."),
            timeout: Duration::from_millis(200),
            head_lines: None,
            tail_lines: None,
            env_overrides: HashMap::new(),
        };

        let res = runner.execute(opts, None, None).unwrap();
        assert!(res.timed_out);
    }
}
