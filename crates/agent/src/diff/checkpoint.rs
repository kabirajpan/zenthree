use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use serde::{Deserialize, Serialize};

/// Snapshot of a single file before modifications in a turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileSnapshot {
    /// Target path (relative or absolute).
    pub path: PathBuf,
    /// Pre-turn content. If `None`, file did not exist before this turn (was created).
    pub pre_turn_content: Option<String>,
}

/// Point-in-time snapshot of all files affected during a specific agent turn.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TurnCheckpoint {
    /// Turn identifier.
    pub turn_id: u64,
    /// Human-readable label or intent of the turn.
    pub description: String,
    /// Snapshots of all files touched in this turn.
    pub snapshots: Vec<FileSnapshot>,
    /// Creation timestamp.
    pub created_at: SystemTime,
}

/// Report summarizing files restored or removed when reverting a checkpoint.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RevertReport {
    pub turn_id: u64,
    /// Files restored to their pre-turn contents.
    pub restored_files: Vec<PathBuf>,
    /// Newly created files removed upon revert.
    pub deleted_files: Vec<PathBuf>,
}

/// Multi-buffer checkpoint timeline managing atomic rollback across modified files.
#[derive(Debug, Clone, Default)]
pub struct CheckpointTimeline {
    checkpoints: Vec<TurnCheckpoint>,
}

impl CheckpointTimeline {
    pub fn new() -> Self {
        Self {
            checkpoints: Vec::new(),
        }
    }

    /// Returns the total number of checkpoints in the timeline.
    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    /// Returns true if the timeline has no checkpoints.
    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    /// Returns a reference to all recorded checkpoints.
    pub fn checkpoints(&self) -> &[TurnCheckpoint] {
        &self.checkpoints
    }

    /// Captures pre-modification snapshots of the given files before executing a turn.
    pub fn capture_turn(
        &mut self,
        turn_id: u64,
        description: &str,
        file_paths: &[impl AsRef<Path>],
        workspace_root: &Path,
    ) -> TurnCheckpoint {
        let mut snapshots = Vec::with_capacity(file_paths.len());

        for p in file_paths {
            let rel_or_abs = p.as_ref();
            let full_path = if rel_or_abs.is_absolute() {
                rel_or_abs.to_path_buf()
            } else {
                workspace_root.join(rel_or_abs)
            };

            let pre_turn_content = if full_path.exists() {
                fs::read_to_string(&full_path).ok()
            } else {
                None // File does not exist yet (will be created in this turn)
            };

            snapshots.push(FileSnapshot {
                path: rel_or_abs.to_path_buf(),
                pre_turn_content,
            });
        }

        let checkpoint = TurnCheckpoint {
            turn_id,
            description: description.to_string(),
            snapshots,
            created_at: SystemTime::now(),
        };

        self.checkpoints.push(checkpoint.clone());
        checkpoint
    }

    /// Reverts the specified turn, atomically restoring all files to their pre-turn states.
    pub fn revert_turn(
        &mut self,
        turn_id: u64,
        workspace_root: &Path,
    ) -> Result<RevertReport, std::io::Error> {
        let idx = self
            .checkpoints
            .iter()
            .position(|c| c.turn_id == turn_id)
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Checkpoint for turn {} not found", turn_id),
                )
            })?;

        let checkpoint = self.checkpoints.remove(idx);
        let mut report = RevertReport {
            turn_id,
            restored_files: Vec::new(),
            deleted_files: Vec::new(),
        };

        for snapshot in checkpoint.snapshots {
            let full_path = if snapshot.path.is_absolute() {
                snapshot.path.clone()
            } else {
                workspace_root.join(&snapshot.path)
            };

            match snapshot.pre_turn_content {
                Some(original_content) => {
                    // Restore original content
                    if let Some(parent) = full_path.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::write(&full_path, original_content)?;
                    report.restored_files.push(snapshot.path);
                }
                None => {
                    // File didn't exist before this turn -> delete newly created file
                    if full_path.exists() {
                        fs::remove_file(&full_path)?;
                        report.deleted_files.push(snapshot.path);
                    }
                }
            }
        }

        Ok(report)
    }

    /// Undoes the most recent turn checkpoint in the timeline.
    pub fn undo_latest(
        &mut self,
        workspace_root: &Path,
    ) -> Result<Option<RevertReport>, std::io::Error> {
        if let Some(latest) = self.checkpoints.last() {
            let turn_id = latest.turn_id;
            let report = self.revert_turn(turn_id, workspace_root)?;
            Ok(Some(report))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_checkpoint_multi_file_rollback() {
        let temp = TempDir::new().unwrap();
        let ws = temp.path();

        let file_a = "crates/core/src/lib.rs";
        let file_b = "crates/core/src/new_mod.rs";

        // Setup existing file A
        let full_a = ws.join(file_a);
        fs::create_dir_all(full_a.parent().unwrap()).unwrap();
        fs::write(&full_a, "pub fn original_a() {}\n").unwrap();

        // File B does not exist initially
        let full_b = ws.join(file_b);
        assert!(!full_b.exists());

        let mut timeline = CheckpointTimeline::new();

        // 1. Capture turn 1 checkpoint before modifying
        timeline.capture_turn(1, "Add new_mod and update lib", &[file_a, file_b], ws);
        assert_eq!(timeline.len(), 1);

        // 2. Perform modifications (Turn 1 actions)
        fs::write(&full_a, "pub fn modified_a() {}\n").unwrap();
        fs::write(&full_b, "pub fn created_b() {}\n").unwrap();

        assert_eq!(fs::read_to_string(&full_a).unwrap(), "pub fn modified_a() {}\n");
        assert!(full_b.exists());

        // 3. Revert Turn 1
        let report = timeline.undo_latest(ws).unwrap().expect("Should have reverted");
        assert_eq!(report.turn_id, 1);
        assert_eq!(report.restored_files.len(), 1);
        assert_eq!(report.deleted_files.len(), 1);

        // 4. Verify post-revert state: File A restored, File B deleted
        assert_eq!(fs::read_to_string(&full_a).unwrap(), "pub fn original_a() {}\n");
        assert!(!full_b.exists(), "Newly created file should be removed on rollback");
    }
}
