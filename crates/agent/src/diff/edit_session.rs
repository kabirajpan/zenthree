use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use crate::diff::matrix_diff::{DiffOp, MatrixDiff, StreamingDiff};
use crate::environment::workspace_jail::WorkspaceJail;
use crate::tools::fs::apply_replacement;

/// Represents an in-memory editing session on an open file buffer.
/// Allows streaming incremental patches from an LLM and rendering live diffs
/// before committing to the filesystem.
#[derive(Debug, Clone)]
pub struct BufferEditSession {
    file_path: PathBuf,
    original_content: String,
    working_content: String,
    diff: StreamingDiff,
    is_dirty: bool,
}

/// Status of applying a streaming edit to a buffer session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditSessionStatus {
    pub is_dirty: bool,
    pub operations: Vec<DiffOp>,
    pub unified_diff: String,
}

impl BufferEditSession {
    /// Creates a new buffer edit session for the specified path with its original content.
    pub fn new(file_path: impl Into<PathBuf>, original_content: &str) -> Self {
        let path = file_path.into();
        Self {
            file_path: path,
            original_content: original_content.to_string(),
            working_content: original_content.to_string(),
            diff: StreamingDiff::new(original_content),
            is_dirty: false,
        }
    }

    pub fn file_path(&self) -> &Path {
        &self.file_path
    }

    pub fn original_content(&self) -> &str {
        &self.original_content
    }

    pub fn working_content(&self) -> &str {
        &self.working_content
    }

    pub fn is_dirty(&self) -> bool {
        self.is_dirty
    }

    /// Replaces the working content with `new_content` and recomputes the active diff.
    pub fn update_content(&mut self, new_content: &str) -> EditSessionStatus {
        self.working_content = new_content.to_string();
        self.is_dirty = self.working_content != self.original_content;
        let ops = MatrixDiff::compute_diff(&self.original_content, &self.working_content);
        let unified_diff = MatrixDiff::format_unified(&ops);

        EditSessionStatus {
            is_dirty: self.is_dirty,
            operations: ops,
            unified_diff,
        }
    }

    /// Streams raw tokens into the buffer as full replacement content.
    pub fn push_streaming_chunk(&mut self, chunk: &str) -> EditSessionStatus {
        let ops = self.diff.push_chunk(chunk);
        self.working_content = self.diff.current_text().to_string();
        self.is_dirty = self.working_content != self.original_content;
        let unified_diff = MatrixDiff::format_unified(&ops);

        EditSessionStatus {
            is_dirty: self.is_dirty,
            operations: ops,
            unified_diff,
        }
    }

    /// Applies a search/replace block (`<<<<<<< SEARCH ... ======= ... >>>>>>> REPLACE`)
    /// to the current working content in-memory.
    pub fn apply_patch_block(
        &mut self,
        search: &str,
        replace: &str,
    ) -> Result<EditSessionStatus, String> {
        let (updated, _replacements, _strategy) = apply_replacement(&self.working_content, search, replace)?;

        Ok(self.update_content(&updated))
    }

    /// Commits the working content atomically to the filesystem within the workspace jail.
    pub fn commit(
        &mut self,
        workspace_root: &Path,
        jail: Option<&WorkspaceJail>,
    ) -> Result<usize, String> {
        if !self.is_dirty {
            return Ok(0);
        }

        let full_path = if self.file_path.is_absolute() {
            self.file_path.clone()
        } else {
            workspace_root.join(&self.file_path)
        };

        if let Some(j) = jail {
            j.validate_write(&full_path)
                .map_err(|e| format!("Security jail write denial: {}", e))?;
        }

        // Create parent directories if needed
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directories: {}", e))?;
        }

        // Atomic write via sibling tempfile
        let file_name = full_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("target");
        let temp_name = format!(".{}.tmp_{}", file_name, std::process::id());
        let temp_path = full_path.with_file_name(temp_name);

        std::fs::write(&temp_path, &self.working_content)
            .map_err(|e| format!("Failed to write temporary buffer file: {}", e))?;

        std::fs::rename(&temp_path, &full_path)
            .map_err(|e| format!("Failed to commit buffer rename: {}", e))?;

        let bytes = self.working_content.len();
        self.original_content = self.working_content.clone();
        self.diff = StreamingDiff::new(&self.original_content);
        self.is_dirty = false;

        Ok(bytes)
    }

    /// Discards all uncommitted edits, restoring original content.
    pub fn discard(&mut self) {
        self.working_content = self.original_content.clone();
        self.diff = StreamingDiff::new(&self.original_content);
        self.is_dirty = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_buffer_edit_session_lifecycle() {
        let initial = "const PORT: u16 = 3000;\nfn start() {}\n";
        let mut session = BufferEditSession::new("server.rs", initial);

        assert!(!session.is_dirty());
        assert_eq!(session.working_content(), initial);

        // Apply patch
        let status = session
            .apply_patch_block("const PORT: u16 = 3000;", "const PORT: u16 = 8080;")
            .expect("Patch should apply");

        assert!(status.is_dirty);
        assert!(session.is_dirty());
        assert!(status.unified_diff.contains("-const PORT: u16 = 3000;"));
        assert!(status.unified_diff.contains("+const PORT: u16 = 8080;"));

        // Test Discard
        session.discard();
        assert!(!session.is_dirty());
        assert_eq!(session.working_content(), initial);
    }

    #[test]
    fn test_buffer_commit_to_disk() {
        let temp = TempDir::new().unwrap();
        let ws = temp.path();
        let rel_file = "src/config.rs";

        let mut session = BufferEditSession::new(rel_file, "// original");
        session.update_content("// updated line 1\n// updated line 2");

        assert!(session.is_dirty());
        let written = session.commit(ws, None).expect("Commit must succeed");
        assert!(written > 0);
        assert!(!session.is_dirty());

        let disk_content = std::fs::read_to_string(ws.join(rel_file)).unwrap();
        assert_eq!(disk_content, "// updated line 1\n// updated line 2");
    }
}
