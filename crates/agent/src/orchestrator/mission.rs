use serde::{Deserialize, Serialize};
use crate::agents::SubAgentRole;
use crate::types::EditedFileSummary;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Pending,
    ReadingChecklist,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub target_file: Option<String>,
    pub diff_summary: Option<EditedFileSummary>,
    #[serde(default)]
    pub sub_agent: Option<SubAgentRole>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMission {
    pub id: String,
    pub goal: String,
    pub tasks: Vec<AgentTask>,
    pub active_task_index: Option<usize>,
    pub is_active: bool,
    pub is_paused: bool,
    pub status_message: String,
    pub self_healing_attempts: usize,
}

impl AgentMission {
    /// Returns index of next task with Pending status
    pub fn next_runnable_task_index(&self) -> Option<usize> {
        self.tasks.iter().position(|t| t.status == TaskStatus::Pending)
    }

    /// Marks the specified task failed with a diagnostic message and advances to next runnable task
    pub fn mark_task_failed_and_continue(&mut self, idx: usize, error_msg: String) {
        if let Some(t) = self.tasks.get_mut(idx) {
            t.status = TaskStatus::Failed(error_msg);
        }
        if let Some(next_idx) = self.next_runnable_task_index() {
            self.active_task_index = Some(next_idx);
            if let Some(next_task) = self.tasks.get_mut(next_idx) {
                next_task.status = TaskStatus::InProgress;
                self.status_message = format!("Task {} failed, proceeding with: {}", idx + 1, next_task.title);
            }
            self.is_active = true;
        } else {
            self.active_task_index = None;
            self.is_active = false;
            self.status_message = "All tasks processed (completed with some failures)".to_string();
        }
    }

    /// Advances to the next pending task, marking current task completed
    pub fn complete_task_and_advance(&mut self, idx: usize) {
        if let Some(t) = self.tasks.get_mut(idx) {
            t.status = TaskStatus::Completed;
        }
        let total_tasks = self.tasks.len();
        if let Some(next_idx) = self.next_runnable_task_index() {
            self.active_task_index = Some(next_idx);
            if let Some(next_task) = self.tasks.get_mut(next_idx) {
                next_task.status = TaskStatus::InProgress;
                self.status_message = format!("Executing Task {} of {}: {}", next_idx + 1, total_tasks, next_task.title);
            }
            self.is_active = true;
        } else {
            self.active_task_index = None;
            self.is_active = false;
            self.status_message = "✓ All tasks completed successfully".to_string();
        }
    }
}
