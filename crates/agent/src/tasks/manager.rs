use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use super::task::{BackgroundTask, TaskStatus};

/// Outcome of spawning a process: either completed synchronously or backgrounded.
#[derive(Debug)]
pub enum SpawnOutcome {
    Synchronous {
        exit_code: i32,
        stdout: String,
        stderr: String,
    },
    Background {
        task_id: String,
        log_path: PathBuf,
        message: String,
    },
}

/// Task summary for list queries.
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskSummary {
    pub task_id: String,
    pub command: String,
    pub status: String,
    pub is_daemon: bool,
    pub elapsed_seconds: u64,
    pub log_path: String,
}

/// Central task manager supervising asynchronous background tasks.
#[derive(Clone)]
pub struct TaskManager {
    tasks: Arc<Mutex<HashMap<String, BackgroundTask>>>,
    counter: Arc<AtomicUsize>,
    log_dir: PathBuf,
    notifications: Arc<Mutex<Vec<String>>>,
}

impl Default for TaskManager {
    fn default() -> Self {
        let base_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".zene")
            .join("tasks");
        Self::with_log_dir(base_dir)
    }
}

impl TaskManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_log_dir(log_dir: PathBuf) -> Self {
        let _ = std::fs::create_dir_all(&log_dir);
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            counter: Arc::new(AtomicUsize::new(1001)),
            log_dir,
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Spawns a command, checking if it completes within `wait_ms`. If not, or if `is_daemon` is true,
    /// pushes it into background task supervision and returns immediately.
    pub fn spawn(
        &self,
        command_str: &str,
        cwd: &Path,
        is_daemon: bool,
        wait_ms: u64,
    ) -> Result<SpawnOutcome, String> {
        let task_id = format!("task-{}", self.counter.fetch_add(1, Ordering::SeqCst));
        let task = BackgroundTask::spawn(
            task_id.clone(),
            command_str,
            cwd.to_path_buf(),
            is_daemon,
            self.log_dir.clone(),
        )?;

        // If explicit daemon, background immediately without waiting
        if is_daemon {
            let log_path = task.log_path.clone();
            let msg = format!(
                "Daemon process started in background with task id: {}\nTask logs: {}",
                task_id, log_path.display()
            );
            self.tasks.lock().unwrap().insert(task_id.clone(), task);
            return Ok(SpawnOutcome::Background {
                task_id,
                log_path,
                message: msg,
            });
        }

        // Wait synchronously up to wait_ms before sending to background
        let poll_interval = Duration::from_millis(50);
        let max_wait = Duration::from_millis(wait_ms);
        let start = Instant::now();

        while start.elapsed() < max_wait {
            match task.current_status() {
                TaskStatus::Completed { exit_code } => {
                    let log_content = task.tail_logs(usize::MAX);
                    return Ok(SpawnOutcome::Synchronous {
                        exit_code,
                        stdout: log_content,
                        stderr: String::new(),
                    });
                }
                TaskStatus::Failed { error } => {
                    return Err(format!("Command failed: {}", error));
                }
                TaskStatus::Killed => {
                    return Err("Command was killed".to_string());
                }
                TaskStatus::Running => {
                    std::thread::sleep(poll_interval);
                }
            }
        }

        // Timed out of synchronous wait window: register in active background tasks!
        let log_path = task.log_path.clone();
        let message = format!(
            "Tool is running as a background task with task id: {}\nTask Description: {}\nTask logs are available at: {}\nStatus: Running",
            task_id, command_str, log_path.display()
        );

        // Setup notification wakeup when process finishes
        let notifications_arc = self.notifications.clone();
        let task_clone = task.clone();
        let id_clone = task_id.clone();

        std::thread::spawn(move || {
            loop {
                std::thread::sleep(Duration::from_millis(250));
                match task_clone.current_status() {
                    TaskStatus::Completed { exit_code } => {
                        let tail = task_clone.tail_logs(15);
                        let notif = format!(
                            "[Message] Task id \"{}\" finished with result:\nExit code: {}\nLog tail:\n{}",
                            id_clone, exit_code, tail
                        );
                        notifications_arc.lock().unwrap().push(notif);
                        break;
                    }
                    TaskStatus::Failed { error } => {
                        let notif = format!("[Message] Task id \"{}\" failed with error: {}", id_clone, error);
                        notifications_arc.lock().unwrap().push(notif);
                        break;
                    }
                    TaskStatus::Killed => {
                        let notif = format!("[Message] Task id \"{}\" was terminated.", id_clone);
                        notifications_arc.lock().unwrap().push(notif);
                        break;
                    }
                    TaskStatus::Running => {}
                }
            }
        });

        self.tasks.lock().unwrap().insert(task_id.clone(), task);

        Ok(SpawnOutcome::Background {
            task_id,
            log_path,
            message,
        })
    }

    /// List all currently monitored tasks.
    pub fn list(&self) -> Vec<TaskSummary> {
        let tasks = self.tasks.lock().unwrap();
        let mut list = Vec::new();
        for (id, t) in tasks.iter() {
            let status_str = match t.current_status() {
                TaskStatus::Running => "running".to_string(),
                TaskStatus::Completed { exit_code } => format!("completed (exit code {})", exit_code),
                TaskStatus::Failed { error } => format!("failed ({})", error),
                TaskStatus::Killed => "killed".to_string(),
            };
            list.push(TaskSummary {
                task_id: id.clone(),
                command: t.command.clone(),
                status: status_str,
                is_daemon: t.is_daemon,
                elapsed_seconds: t.started_at.elapsed().as_secs(),
                log_path: t.log_path.display().to_string(),
            });
        }
        list.sort_by(|a, b| a.task_id.cmp(&b.task_id));
        list
    }

    /// Check status and get log output of a specific task.
    pub fn status(&self, task_id: &str) -> Option<String> {
        let tasks = self.tasks.lock().unwrap();
        let t = tasks.get(task_id)?;
        let st = t.current_status();
        let status_desc = match st {
            TaskStatus::Running => "Running".to_string(),
            TaskStatus::Completed { exit_code } => format!("Completed with exit code {}", exit_code),
            TaskStatus::Failed { error } => format!("Failed: {}", error),
            TaskStatus::Killed => "Terminated".to_string(),
        };
        let log_tail = t.tail_logs(20);
        Some(format!(
            "Task ID: {}\nCommand: {}\nStatus: {}\nElapsed: {}s\nLog file: {}\n\nRecent logs:\n{}",
            task_id,
            t.command,
            status_desc,
            t.started_at.elapsed().as_secs(),
            t.log_path.display(),
            log_tail
        ))
    }

    /// Send input to a running task's stdin.
    pub fn send_input(&self, task_id: &str, input: &str) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap();
        let t = tasks.get(task_id).ok_or_else(|| format!("Task '{}' not found", task_id))?;
        t.send_input(input)
    }

    /// Terminate a background task.
    pub fn kill(&self, task_id: &str) -> Result<(), String> {
        let tasks = self.tasks.lock().unwrap();
        let t = tasks.get(task_id).ok_or_else(|| format!("Task '{}' not found", task_id))?;
        t.kill()
    }

    /// Drain all pending reactive wakeup messages.
    pub fn drain_notifications(&self) -> Vec<String> {
        let mut notifs = self.notifications.lock().unwrap();
        let out = notifs.clone();
        notifs.clear();
        out
    }
}
