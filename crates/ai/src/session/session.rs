use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::types::{AiMessage, EditedFileSummary, ExploredItem};
use crate::orchestrator::mission::AgentMission;
use super::summary::SessionArtifact;

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        static SESSION_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let seq = SESSION_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let id = format!("sess_{}_{}", now, seq);
        let session_title = title.unwrap_or("New Session").to_string();

        let dir_path = super::store::get_session_dir(base_workspace, &id);

        Self {
            id,
            title: session_title,
            created_at: now,
            updated_at: now,
            dir_path,
            has_plan: false,
            has_mission: false,
            has_walkthrough: false,
        }
    }

    pub fn init_starter_files(&mut self) {
        let _ = std::fs::create_dir_all(&self.dir_path);
        let _ = std::fs::create_dir_all(self.artifacts_dir());
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
        let _ = std::fs::create_dir_all(&self.dir_path);
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

        let _ = std::fs::create_dir_all(&self.dir_path);
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
        let completed = mission.tasks.iter().filter(|t| t.status == crate::orchestrator::mission::TaskStatus::Completed).count();
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
                crate::orchestrator::mission::TaskStatus::Completed => "[x]",
                crate::orchestrator::mission::TaskStatus::InProgress => "[/]",
                crate::orchestrator::mission::TaskStatus::ReadingChecklist => "[*]",
                crate::orchestrator::mission::TaskStatus::Pending => "[ ]",
                crate::orchestrator::mission::TaskStatus::Failed(_) => "[!]",
            };
            let badge = task.sub_agent.map(|sa| format!(" `{}`", sa.badge())).unwrap_or_default();
            content.push_str(&format!("- {} **{}**{}", mark, task.title, badge));
            if let Some(ref target) = task.target_file {
                content.push_str(&format!(" (`{}`)", target));
            }
            if let crate::orchestrator::mission::TaskStatus::Failed(ref err) = task.status {
                content.push_str(&format!(" — *Error: {}*", err));
            }
            content.push_str("\n");
        }

        content.push_str("\n---\n*Generated autonomously by ZENE Agent*\n");

        let _ = std::fs::create_dir_all(&self.dir_path);
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

        content.push_str("## Verification & Status\n\n");
        if verified {
            content.push_str("- [x] Workspace verification passed\n- [x] Syntax & AST structure intact\n");
        } else {
            content.push_str("- [ ] Pending final verification / compiler check\n");
        }

        let _ = std::fs::create_dir_all(&self.dir_path);
        let _ = std::fs::write(&path, content);
        self.has_walkthrough = true;
    }

    pub fn load_from_dir(dir_path: PathBuf) -> Option<Self> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return None;
        }

        let session_id = dir_path.file_name()?.to_str()?;
        let plan_path = dir_path.join("implementation_plan.md");
        let mut title = session_id.to_string();

        if let Ok(content) = std::fs::read_to_string(&plan_path) {
            if let Some(first_line) = content.lines().next() {
                let t = first_line.trim_start_matches('#').trim();
                if let Some(stripped) = t.strip_prefix("Implementation Plan:") {
                    title = stripped.trim().to_string();
                } else if let Some(stripped) = t.strip_prefix("Implementation Plan -") {
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
