use std::path::{Path, PathBuf};
use crate::state::{AiMessage, EditedFileSummary, ExploredItem};
use crate::agent::AgentMission;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
    #[serde(default)]
    pub message_count: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionArtifact {
    pub name: String,
    pub path: PathBuf,
    pub updated_at: u64,
    pub size_bytes: u64,
    pub icon: String,
}

pub fn format_relative_time(secs: u64) -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if secs == 0 || secs > now {
        return "recently".to_string();
    }
    let diff = now.saturating_sub(secs);
    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        format!("{}m ago", diff / 60)
    } else if diff < 86400 {
        format!("{}h ago", diff / 3600)
    } else {
        format!("{}d ago", diff / 86400)
    }
}

pub fn format_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentSession {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub dir_path: PathBuf,
    pub has_plan: bool,
    pub has_mission: bool,
    pub has_walkthrough: bool,
}

impl AgentSession {
    pub fn new(base_workspace: &Path, title: Option<&str>) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let id = format!("sess_{}", now);
        let session_title = title.unwrap_or("New Session").to_string();

        let dir_path = get_session_dir(base_workspace, &id);
        let _ = std::fs::create_dir_all(&dir_path);

        let mut session = Self {
            id,
            title: session_title,
            created_at: now,
            updated_at: now,
            dir_path,
            has_plan: false,
            has_mission: false,
            has_walkthrough: false,
        };

        // Create empty or starter files
        session.init_starter_files();
        session
    }

    pub fn init_starter_files(&mut self) {
        let plan_path = self.dir_path.join("implementation_plan.md");
        if !plan_path.exists() {
            let initial_plan = format!(
                "# Implementation Plan - {}\n\n*Waiting for agent deliberation and task breakdown...*\n",
                self.title
            );
            let _ = std::fs::write(&plan_path, initial_plan);
            self.has_plan = true;
        }

        let mission_path = self.dir_path.join("mission.md");
        if !mission_path.exists() {
            let initial_mission = format!(
                "# Agent Mission - {}\n\n**Status**: Initialized\n\n## Tasks\n- [ ] Initialize task context\n",
                self.title
            );
            let _ = std::fs::write(&mission_path, initial_mission);
            self.has_mission = true;
        }

        let walkthrough_path = self.dir_path.join("walkthrough.md");
        if !walkthrough_path.exists() {
            let initial_walkthrough = format!(
                "# Walkthrough - {}\n\n*Changes will be documented here once implemented and verified.*\n",
                self.title
            );
            let _ = std::fs::write(&walkthrough_path, initial_walkthrough);
            self.has_walkthrough = true;
        }

        let _ = std::fs::create_dir_all(self.artifacts_dir());
    }

    pub fn artifacts_dir(&self) -> PathBuf {
        self.dir_path.join("artifacts")
    }

    pub fn plan_file_path(&self) -> PathBuf {
        self.dir_path.join("implementation_plan.md")
    }

    pub fn mission_file_path(&self) -> PathBuf {
        self.dir_path.join("mission.md")
    }

    pub fn walkthrough_file_path(&self) -> PathBuf {
        self.dir_path.join("walkthrough.md")
    }

    pub fn messages_file_path(&self) -> PathBuf {
        self.dir_path.join("messages.json")
    }

    pub fn save_messages(&self, messages: &[AiMessage]) {
        let path = self.messages_file_path();
        if let Ok(json) = serde_json::to_string_pretty(messages) {
            let _ = std::fs::write(path, json);
        }
    }

    pub fn load_messages(&self) -> Option<Vec<AiMessage>> {
        let path = self.messages_file_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(messages) = serde_json::from_str::<Vec<AiMessage>>(&content) {
                    return Some(messages);
                }
            }
        }
        None
    }

    pub fn list_artifacts(&self) -> Vec<SessionArtifact> {
        let mut list = Vec::new();
        let plan_p = self.plan_file_path();
        if plan_p.exists() {
            let meta = std::fs::metadata(&plan_p).ok();
            let updated = meta.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(self.updated_at);
            let size = meta.map(|m| m.len()).unwrap_or(0);
            list.push(SessionArtifact {
                name: "implementation_plan.md".to_string(),
                path: plan_p,
                updated_at: updated,
                size_bytes: size,
                icon: "\u{ea7f}".to_string(),
            });
        }

        let mission_p = self.mission_file_path();
        if mission_p.exists() {
            let meta = std::fs::metadata(&mission_p).ok();
            let updated = meta.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(self.updated_at);
            let size = meta.map(|m| m.len()).unwrap_or(0);
            list.push(SessionArtifact {
                name: "mission.md".to_string(),
                path: mission_p,
                updated_at: updated,
                size_bytes: size,
                icon: "\u{f0e7}".to_string(),
            });
        }

        let walk_p = self.walkthrough_file_path();
        if walk_p.exists() {
            let meta = std::fs::metadata(&walk_p).ok();
            let updated = meta.as_ref()
                .and_then(|m| m.modified().ok())
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs())
                .unwrap_or(self.updated_at);
            let size = meta.map(|m| m.len()).unwrap_or(0);
            list.push(SessionArtifact {
                name: "walkthrough.md".to_string(),
                path: walk_p,
                updated_at: updated,
                size_bytes: size,
                icon: "\u{f15c}".to_string(),
            });
        }

        let art_dir = self.artifacts_dir();
        if let Ok(entries) = std::fs::read_dir(&art_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("artifact").to_string();
                    let meta = std::fs::metadata(&p).ok();
                    let updated = meta.as_ref()
                        .and_then(|m| m.modified().ok())
                        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                        .map(|d| d.as_secs())
                        .unwrap_or(self.updated_at);
                    let size = meta.map(|m| m.len()).unwrap_or(0);
                    list.push(SessionArtifact {
                        name,
                        path: p,
                        updated_at: updated,
                        size_bytes: size,
                        icon: "\u{ea7f}".to_string(),
                    });
                }
            }
        }

        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        list
    }

    /// Writes or updates the `implementation_plan.md` file on disk
    pub fn update_implementation_plan(&mut self, prompt: &str, explored: &[ExploredItem], plan_text: &str) {
        let path = self.plan_file_path();
        let mut content = String::new();
        content.push_str(&format!("# Implementation Plan: {}\n\n", self.title));
        content.push_str(&format!("**User Request**: {}\n\n", prompt));

        if !explored.is_empty() {
            content.push_str("## Analyzed Workspace Context\n");
            for item in explored {
                let range = item.line_range.as_deref().unwrap_or("");
                content.push_str(&format!("- **`{}`** {} ({})\n", item.file_name, range, item.action_type));
            }
            content.push_str("\n");
        }

        content.push_str("## Proposed Architecture & Changes\n\n");
        if plan_text.trim().is_empty() {
            content.push_str("Automatic multi-file generation & symbol synthesis in progress...\n");
        } else {
            content.push_str(plan_text);
            content.push_str("\n");
        }

        content.push_str("\n## Verification Plan\n- Automated syntax checks and compiler diagnostics\n- Workspace parity verification\n");

        let _ = std::fs::write(&path, content);
        self.has_plan = true;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
    }

    /// Writes or updates the `mission.md` file on disk from the AgentMission state
    pub fn update_mission_file(&mut self, mission: &AgentMission) {
        let path = self.mission_file_path();
        let total = mission.tasks.len();
        let completed = mission.tasks.iter().filter(|t| t.status == crate::agent::TaskStatus::Completed).count();
        let status_label = if mission.is_paused {
            "Paused"
        } else if !mission.is_active || completed == total {
            "Completed"
        } else {
            "In Progress"
        };

        let mut content = String::new();
        content.push_str(&format!("# Agent Mission: {}\n\n", mission.goal));
        content.push_str(&format!("- **Status**: {}\n", status_label));
        content.push_str(&format!("- **Progress**: {}/{} tasks completed ({:.0}%)\n", completed, total, if total > 0 { completed as f32 / total as f32 * 100.0 } else { 0.0 }));
        if mission.self_healing_attempts > 0 {
            content.push_str(&format!("- **Compiler Self-Healing**: Attempt {}/2\n", mission.self_healing_attempts));
        }
        content.push_str(&format!("- **Current State**: {}\n\n", mission.status_message));

        content.push_str("## Task Checklist\n\n");
        for task in &mission.tasks {
            let mark = match task.status {
                crate::agent::TaskStatus::Completed => "[x]",
                crate::agent::TaskStatus::InProgress => "[/]",
                crate::agent::TaskStatus::ReadingChecklist => "[*]",
                crate::agent::TaskStatus::Pending => "[ ]",
                crate::agent::TaskStatus::Failed(_) => "[!]",
            };
            content.push_str(&format!("- {} **{}**", mark, task.title));
            if let Some(ref target) = task.target_file {
                content.push_str(&format!(" (`{}`)", target));
            }
            if let crate::agent::TaskStatus::Failed(ref err) = task.status {
                content.push_str(&format!(" — *Error: {}*", err));
            }
            content.push_str("\n");
        }

        content.push_str("\n---\n*Generated autonomously by Zenthree Agent*\n");

        let _ = std::fs::write(&path, content);
        self.has_mission = true;
    }

    /// Writes or updates the `walkthrough.md` file on disk once changes are applied
    pub fn update_walkthrough_file(&mut self, goal: &str, edited_files: &[EditedFileSummary], verified: bool) {
        let path = self.walkthrough_file_path();
        let mut content = String::new();
        content.push_str(&format!("# Walkthrough: {}\n\n", self.title));
        content.push_str(&format!("**Goal**: {}\n\n", goal));

        content.push_str("## Changes Implemented\n\n");
        if edited_files.is_empty() {
            content.push_str("No files directly modified yet.\n\n");
        } else {
            for f in edited_files {
                content.push_str(&format!("### `{}`\n", f.file_name));
                content.push_str(&format!("- **Lines Added**: `+{}`\n", f.lines_added));
                content.push_str(&format!("- **Lines Removed**: `-{}`\n\n", f.lines_removed));
            }
        }

        content.push_str("## Verification & Status\n");
        if verified {
            content.push_str("✓ **All code modifications verified and accepted**\n- Compiler checks passed with 0 errors\n- Files persisted directly to disk\n");
        } else {
            content.push_str("⏳ Code changes pending verification / acceptance\n");
        }

        let _ = std::fs::write(&path, content);
        self.has_walkthrough = true;
    }

    /// Loads an existing session by ID from the session storage directory
    pub fn load(base_workspace: &Path, session_id: &str) -> Option<Self> {
        let dir_path = get_session_dir(base_workspace, session_id);
        if !dir_path.exists() {
            return None;
        }
        let plan_path = dir_path.join("implementation_plan.md");
        let mut title = session_id.to_string();
        if let Ok(content) = std::fs::read_to_string(&plan_path) {
            if let Some(first_line) = content.lines().next() {
                let t = first_line.trim_start_matches('#').trim();
                if let Some(stripped) = t.strip_prefix("Implementation Plan:") {
                    title = stripped.trim().to_string();
                } else if !t.is_empty() {
                    title = t.to_string();
                }
            }
        }
        let updated_at = std::fs::metadata(&dir_path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let has_plan = dir_path.join("implementation_plan.md").exists();
        let has_mission = dir_path.join("mission.md").exists();
        let has_walkthrough = dir_path.join("walkthrough.md").exists();

        Some(Self {
            id: session_id.to_string(),
            title,
            created_at: updated_at,
            updated_at,
            dir_path,
            has_plan,
            has_mission,
            has_walkthrough,
        })
    }
}

/// Computes the persistent directory path for storing session artifacts
pub fn get_session_dir(base_workspace: &Path, session_id: &str) -> PathBuf {
    if base_workspace.exists() && base_workspace.is_dir() {
        base_workspace.join(".zenthree").join("sessions").join(session_id)
    } else {
        dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("zenthree")
            .join("sessions")
            .join(session_id)
    }
}

/// Deletes a session directory and its contents from disk
pub fn delete_session(base_workspace: &Path, session_id: &str) -> bool {
    let dir = get_session_dir(base_workspace, session_id);
    if dir.exists() {
        let _ = std::fs::remove_dir_all(&dir);
        true
    } else {
        false
    }
}

/// Lists all saved session summaries from `.zenthree/sessions` sorted by last modified
pub fn list_saved_sessions(base_workspace: &Path) -> Vec<SessionSummary> {
    let root = if base_workspace.exists() && base_workspace.is_dir() {
        base_workspace.join(".zenthree").join("sessions")
    } else {
        dirs::cache_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("zenthree")
            .join("sessions")
    };

    let mut list = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let id = entry.file_name().to_string_lossy().to_string();
                let plan_path = entry.path().join("implementation_plan.md");
                let mut title = id.clone();
                if let Ok(content) = std::fs::read_to_string(&plan_path) {
                    if let Some(first_line) = content.lines().next() {
                        let t = first_line.trim_start_matches('#').trim();
                        if let Some(stripped) = t.strip_prefix("Implementation Plan:") {
                            title = stripped.trim().to_string();
                        } else if !t.is_empty() {
                            title = t.to_string();
                        }
                    }
                }
                let updated_at = std::fs::metadata(entry.path())
                    .and_then(|m| m.modified())
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let mut message_count = 0;
                let msgs_path = entry.path().join("messages.json");
                if msgs_path.exists() {
                    if let Ok(content) = std::fs::read_to_string(&msgs_path) {
                        if let Ok(parsed) = serde_json::from_str::<Vec<serde_json::Value>>(&content) {
                            message_count = parsed.len();
                        }
                    }
                }

                list.push(SessionSummary {
                    id,
                    title,
                    created_at: updated_at,
                    updated_at,
                    message_count,
                });
            }
        }
    }
    list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    list
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::{AgentMission, AgentTask, TaskStatus};

    fn make_test_tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "zenthree_test_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let _ = std::fs::create_dir_all(&dir);
        dir
    }

    #[test]
    fn test_agent_session_files_creation_and_updates() {
        let tmp = make_test_tempdir();
        let workspace = &tmp;

        let mut session = AgentSession::new(workspace, Some("test_session_1"));
        assert_eq!(session.title, "test_session_1");
        assert!(session.id.starts_with("sess_"));
        assert!(session.dir_path.exists());

        let plan_path = session.dir_path.join("implementation_plan.md");
        let mission_path = session.dir_path.join("mission.md");
        let walkthrough_path = session.dir_path.join("walkthrough.md");

        assert!(plan_path.exists());
        assert!(mission_path.exists());
        assert!(walkthrough_path.exists());

        // Update implementation plan
        session.update_implementation_plan(
            "Build authentication flow",
            &[ExploredItem {
                file_name: "auth.rs".to_string(),
                line_range: Some("#L1-50".to_string()),
                action_type: "Analyzed".to_string(),
            }],
            "Plan: Add JWT validation and login handler.",
        );

        let plan_content = std::fs::read_to_string(&plan_path).expect("read plan");
        assert!(plan_content.contains("Build authentication flow"));
        assert!(plan_content.contains("auth.rs"));

        // Update mission file
        let mission = AgentMission {
            id: "mission_1".to_string(),
            goal: "Build authentication flow".to_string(),
            tasks: vec![
                AgentTask {
                    id: "task_1".to_string(),
                    title: "Analyze auth.rs".to_string(),
                    status: TaskStatus::Completed,
                    target_file: Some("auth.rs".to_string()),
                    diff_summary: None,
                },
                AgentTask {
                    id: "task_2".to_string(),
                    title: "Implement JWT handler".to_string(),
                    status: TaskStatus::InProgress,
                    target_file: Some("auth.rs".to_string()),
                    diff_summary: None,
                },
            ],
            active_task_index: Some(1),
            is_active: true,
            is_paused: false,
            self_healing_attempts: 0,
            status_message: "Working on JWT handler".to_string(),
        };
        session.update_mission_file(&mission);

        let mission_content = std::fs::read_to_string(&mission_path).expect("read mission");
        assert!(mission_content.contains("- [x] **Analyze auth.rs**"));
        assert!(mission_content.contains("- [/] **Implement JWT handler**"));

        // Update walkthrough file
        let edits = vec![
            EditedFileSummary {
                file_name: "auth.rs".to_string(),
                lines_added: 45,
                lines_removed: 5,
            },
        ];
        session.update_walkthrough_file("Build authentication flow", &edits, true);

        let walkthrough_content = std::fs::read_to_string(&walkthrough_path).expect("read walkthrough");
        assert!(walkthrough_content.contains("verified and accepted"));
        assert!(walkthrough_content.contains("auth.rs"));
        assert!(walkthrough_content.contains("+45"));
        assert!(walkthrough_content.contains("-5"));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_saved_sessions() {
        let tmp = make_test_tempdir();
        let workspace = &tmp;

        let s1 = AgentSession::new(workspace, Some("sess_alpha"));
        let s2 = AgentSession::new(workspace, Some("sess_beta"));

        let list = list_saved_sessions(workspace);
        assert!(list.iter().any(|s| s.id == s1.id));
        assert!(list.iter().any(|s| s.id == s2.id));

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_list_artifacts_and_formatters() {
        let tmp = make_test_tempdir();
        let session = AgentSession::new(&tmp, Some("artifacts_session"));

        let arts = session.list_artifacts();
        assert_eq!(arts.len(), 3);
        assert!(arts.iter().any(|a| a.name == "implementation_plan.md"));
        assert!(arts.iter().any(|a| a.name == "mission.md"));
        assert!(arts.iter().any(|a| a.name == "walkthrough.md"));

        // Test custom artifact saved in artifacts_dir
        let custom_file = session.artifacts_dir().join("diagram.png");
        std::fs::write(&custom_file, b"binary image data").unwrap();

        let arts_after = session.list_artifacts();
        assert_eq!(arts_after.len(), 4);
        assert!(arts_after.iter().any(|a| a.name == "diagram.png"));

        // Formatters
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_relative_time(0), "recently");

        let _ = std::fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_session_messages_persistence_and_deletion() {
        let tmp = make_test_tempdir();
        let session = AgentSession::new(&tmp, Some("msg_session"));

        let msgs = vec![
            AiMessage {
                is_user: true,
                content: "How to parse JSON in Rust?".to_string(),
                ..Default::default()
            },
            AiMessage {
                is_user: false,
                content: "Use serde_json crate!".to_string(),
                ..Default::default()
            },
        ];

        session.save_messages(&msgs);
        assert!(session.messages_file_path().exists());

        let loaded = session.load_messages().expect("load saved messages");
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].content, "How to parse JSON in Rust?");
        assert_eq!(loaded[1].content, "Use serde_json crate!");

        let summaries = list_saved_sessions(&tmp);
        let s = summaries.iter().find(|s| s.id == session.id).expect("session in summary");
        assert_eq!(s.message_count, 2);

        // Delete session
        let deleted = delete_session(&tmp, &session.id);
        assert!(deleted);
        assert!(!session.dir_path.exists());

        let summaries_after = list_saved_sessions(&tmp);
        assert!(!summaries_after.iter().any(|s| s.id == session.id));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}

