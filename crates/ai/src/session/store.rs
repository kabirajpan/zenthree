use std::path::{Path, PathBuf};
use super::summary::SessionSummary;

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
    if let Ok(entries) = std::fs::read_dir(&root) {
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
                        } else if let Some(stripped) = t.strip_prefix("Implementation Plan -") {
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

                if message_count == 0 {
                    // Empty ghost folder with 0 messages: clean it up and do not list it
                    let _ = std::fs::remove_dir_all(entry.path());
                    continue;
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
    if list.is_empty() && root.exists() {
        let _ = std::fs::remove_dir_all(&root);
        if let Some(parent) = root.parent() {
            let _ = std::fs::remove_dir(parent);
        }
    }
    list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    list
}
