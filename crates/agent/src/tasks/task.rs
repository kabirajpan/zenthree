use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Status of a background supervised process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Running,
    Completed { exit_code: i32 },
    Failed { error: String },
    Killed,
}

/// Representation of an asynchronous background task.
#[derive(Clone)]
pub struct BackgroundTask {
    pub id: String,
    pub command: String,
    pub cwd: PathBuf,
    pub is_daemon: bool,
    pub log_path: PathBuf,
    pub started_at: Instant,
    pub status: Arc<Mutex<TaskStatus>>,
    child: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
}

impl BackgroundTask {
    pub fn spawn(
        id: String,
        command_str: &str,
        cwd: PathBuf,
        is_daemon: bool,
        log_dir: PathBuf,
    ) -> Result<Self, String> {
        let _ = std::fs::create_dir_all(&log_dir);
        let log_path = log_dir.join(format!("{}.log", id));
        let log_file = File::create(&log_path)
            .map_err(|e| format!("Failed to create log file '{}': {}", log_path.display(), e))?;

        let log_file_err = log_file.try_clone()
            .map_err(|e| format!("Failed to clone log file handle: {}", e))?;

        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", command_str]);
            c
        } else {
            let mut c = Command::new("sh");
            c.args(["-c", command_str]);
            c
        };

        cmd.current_dir(&cwd);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::from(log_file));
        cmd.stderr(Stdio::from(log_file_err));

        let mut child = cmd.spawn()
            .map_err(|e| format!("Failed to spawn command '{}': {}", command_str, e))?;

        let stdin = child.stdin.take();
        let status = Arc::new(Mutex::new(TaskStatus::Running));

        let task = Self {
            id: id.clone(),
            command: command_str.to_string(),
            cwd,
            is_daemon,
            log_path,
            started_at: Instant::now(),
            status: status.clone(),
            child: Arc::new(Mutex::new(Some(child))),
            stdin: Arc::new(Mutex::new(stdin)),
        };

        // Spawn background supervisor thread to monitor child process exit
        let child_arc = task.child.clone();
        let status_arc = status.clone();
        let task_id_clone = id.clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));

                let mut child_guard = child_arc.lock().unwrap();
                if let Some(ref mut child) = *child_guard {
                    match child.try_wait() {
                        Ok(Some(exit_status)) => {
                            let mut st = status_arc.lock().unwrap();
                            if *st == TaskStatus::Running {
                                let code = exit_status.code().unwrap_or(-1);
                                *st = TaskStatus::Completed { exit_code: code };
                            }
                            break;
                        }
                        Ok(None) => {
                            // Still running, check if explicitly marked as Killed
                            let st = status_arc.lock().unwrap();
                            if *st == TaskStatus::Killed {
                                let _ = child.kill();
                                break;
                            }
                        }
                        Err(e) => {
                            let mut st = status_arc.lock().unwrap();
                            *st = TaskStatus::Failed {
                                error: format!("Failed to wait on task '{}': {}", task_id_clone, e),
                            };
                            break;
                        }
                    }
                } else {
                    break;
                }
            }
        });

        Ok(task)
    }

    /// Read latest status, checking if process has terminated.
    pub fn current_status(&self) -> TaskStatus {
        let st = self.status.lock().unwrap();
        st.clone()
    }

    /// Write input string to the child process's stdin.
    pub fn send_input(&self, input: &str) -> Result<(), String> {
        let mut stdin_guard = self.stdin.lock().unwrap();
        if let Some(ref mut stdin) = *stdin_guard {
            stdin.write_all(input.as_bytes())
                .map_err(|e| format!("Failed to write to stdin of task '{}': {}", self.id, e))?;
            stdin.flush()
                .map_err(|e| format!("Failed to flush stdin of task '{}': {}", self.id, e))?;
            Ok(())
        } else {
            Err(format!("Task '{}' has closed its stdin pipe", self.id))
        }
    }

    /// Terminate the child process.
    pub fn kill(&self) -> Result<(), String> {
        let mut st = self.status.lock().unwrap();
        *st = TaskStatus::Killed;

        let mut child_guard = self.child.lock().unwrap();
        if let Some(ref mut child) = *child_guard {
            let _ = child.kill();
            Ok(())
        } else {
            Err(format!("Task '{}' has already terminated", self.id))
        }
    }

    /// Get tail snippet from the log file.
    pub fn tail_logs(&self, max_lines: usize) -> String {
        if !self.log_path.exists() {
            return "(No log output produced yet)".to_string();
        }

        match std::fs::read_to_string(&self.log_path) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                if lines.len() <= max_lines {
                    content
                } else {
                    let start = lines.len() - max_lines;
                    format!("... [truncated {} lines]\n{}", start, lines[start..].join("\n"))
                }
            }
            Err(e) => format!("(Error reading log file: {})", e),
        }
    }
}
