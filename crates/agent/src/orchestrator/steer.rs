use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

/// Mid-flight steering signal emitted by user interactions while the agent is running.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SteeringSignal {
    /// Proceed normally with execution.
    Continue,
    /// Immediately interrupt and cancel the current turn execution.
    Cancel { reason: String },
    /// Redirect the agent mid-flight with new instructions or corrections.
    Redirect { user_feedback: String },
}

/// Thread-safe steering controller that allows UI threads or event loops
/// to steer, interrupt, or redirect an in-flight agent turn.
#[derive(Debug, Clone)]
pub struct SteeringController {
    signal: Arc<Mutex<SteeringSignal>>,
}

impl Default for SteeringController {
    fn default() -> Self {
        Self::new()
    }
}

impl SteeringController {
    pub fn new() -> Self {
        Self {
            signal: Arc::new(Mutex::new(SteeringSignal::Continue)),
        }
    }

    /// Sets the current steering signal (e.g. from user cancellation button or prompt input).
    pub fn emit(&self, signal: SteeringSignal) {
        if let Ok(mut lock) = self.signal.lock() {
            *lock = signal;
        }
    }

    /// Checks the current steering state.
    pub fn check(&self) -> SteeringSignal {
        self.signal
            .lock()
            .map(|s| s.clone())
            .unwrap_or(SteeringSignal::Continue)
    }

    /// Resets the controller back to `Continue`.
    pub fn reset(&self) {
        if let Ok(mut lock) = self.signal.lock() {
            *lock = SteeringSignal::Continue;
        }
    }

    /// Convenience check whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        matches!(self.check(), SteeringSignal::Cancel { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_steering_signals() {
        let controller = SteeringController::new();
        assert_eq!(controller.check(), SteeringSignal::Continue);
        assert!(!controller.is_cancelled());

        // Emit redirect
        controller.emit(SteeringSignal::Redirect {
            user_feedback: "Use port 9090 instead".to_string(),
        });
        assert_eq!(
            controller.check(),
            SteeringSignal::Redirect {
                user_feedback: "Use port 9090 instead".to_string(),
            }
        );

        // Emit cancel
        controller.emit(SteeringSignal::Cancel {
            reason: "User clicked Stop".to_string(),
        });
        assert!(controller.is_cancelled());

        // Reset
        controller.reset();
        assert_eq!(controller.check(), SteeringSignal::Continue);
    }
}
