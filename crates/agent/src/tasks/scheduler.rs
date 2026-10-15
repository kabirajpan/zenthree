use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Condition under which a one-shot timer terminates early.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimerCondition {
    /// Always fire after specified duration.
    Never,
    /// Cancel early if any message is received from any sender.
    Any,
    /// Cancel early if a specific sender ID (e.g. task ID) sends an update.
    SenderId(String),
}

impl Default for TimerCondition {
    fn default() -> Self {
        Self::Never
    }
}

/// Dynamic timer and cron schedule manager.
#[derive(Clone)]
pub struct Scheduler {
    notifications: Arc<Mutex<Vec<String>>>,
}

impl Scheduler {
    pub fn new(notifications: Arc<Mutex<Vec<String>>>) -> Self {
        Self { notifications }
    }

    /// Schedule a one-shot timer.
    pub fn schedule_timer(
        &self,
        duration_seconds: u64,
        prompt: String,
        _condition: TimerCondition,
    ) -> String {
        let notifs = self.notifications.clone();
        let prompt_clone = prompt.clone();

        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_secs(duration_seconds));
            let notif = format!("[Reminder] {}", prompt_clone);
            notifs.lock().unwrap().push(notif);
        });

        format!(
            "One-shot timer scheduled for {} second(s). Will notify: '{}'",
            duration_seconds, prompt
        )
    }

    /// Schedule a recurring interval job.
    pub fn schedule_interval(
        &self,
        interval_seconds: u64,
        prompt: String,
        max_iterations: Option<usize>,
    ) -> String {
        let notifs = self.notifications.clone();
        let prompt_clone = prompt.clone();

        std::thread::spawn(move || {
            let mut iter = 0;
            loop {
                std::thread::sleep(Duration::from_secs(interval_seconds));
                let notif = format!("[Scheduled Event] {}", prompt_clone);
                notifs.lock().unwrap().push(notif);
                iter += 1;
                if let Some(max) = max_iterations {
                    if iter >= max {
                        break;
                    }
                }
            }
        });

        format!(
            "Recurring schedule set for every {} second(s). Notification prompt: '{}'",
            interval_seconds, prompt
        )
    }
}
