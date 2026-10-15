pub mod manager;
pub mod scheduler;
pub mod task;

pub use manager::{SpawnOutcome, TaskManager, TaskSummary};
pub use scheduler::{Scheduler, TimerCondition};
pub use task::{BackgroundTask, TaskStatus};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_task_manager_synchronous_completion() {
        let dir = tempdir().unwrap();
        let mgr = TaskManager::with_log_dir(dir.path().to_path_buf());

        // Fast echo command finishes well within 2000ms
        let res = mgr.spawn("echo 'hello from sync task'", dir.path(), false, 2000);
        assert!(res.is_ok());

        match res.unwrap() {
            SpawnOutcome::Synchronous { exit_code, stdout, .. } => {
                assert_eq!(exit_code, 0);
                assert!(stdout.contains("hello from sync task"));
            }
            SpawnOutcome::Background { .. } => {
                panic!("Expected synchronous execution, got background task");
            }
        }
    }

    #[test]
    fn test_task_manager_background_transition_and_kill() {
        let dir = tempdir().unwrap();
        let mgr = TaskManager::with_log_dir(dir.path().to_path_buf());

        // Sleep 10s with wait_ms = 100ms should transition to background immediately
        let res = mgr.spawn("sleep 10", dir.path(), false, 100);
        assert!(res.is_ok());

        let task_id = match res.unwrap() {
            SpawnOutcome::Background { task_id, .. } => task_id,
            SpawnOutcome::Synchronous { .. } => panic!("Expected background task"),
        };

        let list = mgr.list();
        assert!(!list.is_empty());
        assert!(list.iter().any(|t| t.task_id == task_id));

        let status_desc = mgr.status(&task_id);
        assert!(status_desc.is_some());
        assert!(status_desc.unwrap().contains(&task_id));

        // Kill the task
        let kill_res = mgr.kill(&task_id);
        assert!(kill_res.is_ok());
    }

    #[test]
    fn test_scheduler_one_shot_timer() {
        let notifications = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let scheduler = Scheduler::new(notifications.clone());

        let res = scheduler.schedule_timer(1, "Wake up and run tests".to_string(), TimerCondition::Never);
        assert!(res.contains("One-shot timer scheduled"));

        std::thread::sleep(std::time::Duration::from_millis(1200));

        let notifs = notifications.lock().unwrap();
        assert_eq!(notifs.len(), 1);
        assert!(notifs[0].contains("Wake up and run tests"));
    }
}
