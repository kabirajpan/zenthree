use crate::agents::SubAgentRole;
use crate::types::ExploredItem;
use super::mission::{AgentMission, AgentTask, TaskStatus};

pub fn create_mission_for_prompt(prompt: &str, explored_items: &[ExploredItem]) -> AgentMission {
    let trimmed = prompt.trim();
    let lower = trimmed.to_lowercase();
    let is_refactor = lower.contains("refactor") || lower.contains("optimize");
    let is_explain = lower.contains("explain");
    let is_plan = lower.contains("plan") || lower.contains("architect") || lower.contains("blueprint");

    let mut tasks = Vec::new();

    // Task 1: Context & Reading (Explorer)
    let read_title = if is_plan {
        "Analyze workspace architecture, dependencies & conventions".to_string()
    } else if !explored_items.is_empty() {
        let file_list: Vec<&str> = explored_items.iter().take(2).map(|i| i.file_name.as_str()).collect();
        format!("Analyze workspace context ({})", file_list.join(", "))
    } else {
        "Analyze workspace context & symbols".to_string()
    };
    tasks.push(AgentTask {
        id: "task_1_explore".to_string(),
        title: read_title,
        status: TaskStatus::Completed,
        target_file: explored_items.first().map(|i| i.file_name.clone()),
        diff_summary: None,
        sub_agent: Some(SubAgentRole::Explorer),
    });

    // Task 2: Toolchain & Compiler Detection (Detector)
    tasks.push(AgentTask {
        id: "task_2_detect".to_string(),
        title: "Detect project toolchain, compiler check & test runner".to_string(),
        status: TaskStatus::Completed,
        target_file: None,
        diff_summary: None,
        sub_agent: Some(SubAgentRole::Detector),
    });

    // Task 3: Code Execution / Generation (Builder)
    let exec_title = if is_plan {
        "Formulate comprehensive architectural implementation blueprint".to_string()
    } else if lower.contains("create") || lower.contains("make") || lower.contains("add") {
        let desc = if trimmed.len() > 30 { &trimmed[..30] } else { trimmed };
        format!("Scaffold structure & implement application for `{}`", desc)
    } else if is_refactor {
        "Apply architectural refactoring".to_string()
    } else if is_explain {
        "Formulate comprehensive code analysis".to_string()
    } else {
        let desc = if trimmed.len() > 30 { &trimmed[..30] } else { trimmed };
        format!("Implement changes for `{}`", desc)
    };
    tasks.push(AgentTask {
        id: "task_3_build".to_string(),
        title: exec_title,
        status: TaskStatus::InProgress,
        target_file: None,
        diff_summary: None,
        sub_agent: Some(SubAgentRole::Builder),
    });

    // Task 4: Verification & self-check (Tester)
    let test_title = if is_plan {
        "Formulate automated test strategy & manual verification plan".to_string()
    } else {
        "Verify syntax, run compiler check & test suite".to_string()
    };
    tasks.push(AgentTask {
        id: "task_4_test".to_string(),
        title: test_title,
        status: TaskStatus::Pending,
        target_file: None,
        diff_summary: None,
        sub_agent: Some(SubAgentRole::Tester),
    });

    AgentMission {
        id: format!(
            "mission_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        ),
        goal: trimmed.to_string(),
        tasks,
        active_task_index: Some(2),
        is_active: true,
        is_paused: false,
        status_message: "Executing Task 3 of 4 [⚡ Builder]...".to_string(),
        self_healing_attempts: 0,
    }
}
