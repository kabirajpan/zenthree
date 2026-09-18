use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};

/// Default character threshold beyond which tool output is spilled to disk.
/// 150,000 chars is approximately 37,500 tokens.
pub const DEFAULT_LARGE_OUTPUT_THRESHOLD: usize = 150_000;

static SPILL_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Result of evaluating whether a tool output needed spilling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpilledOutputResult {
    /// Whether the tool payload was spilled to disk.
    pub is_spilled: bool,
    /// The content to present in the LLM conversation context.
    /// If spilled, contains the pointer and navigation instructions.
    pub rendered_content: String,
    /// Path to the spilled file on disk if spilled.
    pub spill_path: Option<PathBuf>,
    /// Original character length of the raw output.
    pub original_char_count: usize,
}

/// Spills oversized tool payloads to disk to avoid context window overflow.
#[derive(Debug, Clone)]
pub struct LargeOutputSpiller {
    threshold: usize,
    workspace_root: Option<PathBuf>,
}

impl LargeOutputSpiller {
    /// Creates a new spiller with the default threshold (150,000 chars).
    pub fn new(workspace_root: Option<PathBuf>) -> Self {
        Self {
            threshold: DEFAULT_LARGE_OUTPUT_THRESHOLD,
            workspace_root,
        }
    }

    /// Creates a new spiller with a customized character threshold.
    pub fn with_threshold(threshold: usize, workspace_root: Option<PathBuf>) -> Self {
        Self {
            threshold,
            workspace_root,
        }
    }

    /// Returns the active character threshold.
    pub fn threshold(&self) -> usize {
        self.threshold
    }

    /// Determines target directory for spilled files: `<workspace>/.zenthree/tmp/`
    /// or system temp directory if workspace is unset.
    fn resolve_spill_dir(&self) -> PathBuf {
        if let Some(ref root) = self.workspace_root {
            root.join(".zenthree").join("tmp")
        } else {
            std::env::temp_dir().join("zenthree_tmp")
        }
    }

    /// Generates a collision-resistant unique filename for spilled output.
    fn generate_spill_filename() -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let seq = SPILL_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        format!("tool_spill_{}_{}_{}.txt", timestamp, pid, seq)
    }

    /// Checks if the tool output exceeds the threshold and spills it to disk if necessary.
    pub fn process_output(&self, tool_name: &str, output: &str) -> SpilledOutputResult {
        let char_count = output.chars().count();
        if char_count <= self.threshold {
            return SpilledOutputResult {
                is_spilled: false,
                rendered_content: output.to_string(),
                spill_path: None,
                original_char_count: char_count,
            };
        }

        // Output exceeds threshold -> write to disk
        let spill_dir = self.resolve_spill_dir();
        if let Err(err) = fs::create_dir_all(&spill_dir) {
            log::error!("Failed to create spill directory {:?}: {}", spill_dir, err);
            // Fallback: return message indicating spill failure with a head/tail excerpt
            return self.build_fallback_result(tool_name, output, char_count, err.to_string());
        }

        let spill_filename = Self::generate_spill_filename();
        let spill_path = spill_dir.join(spill_filename);

        match fs::write(&spill_path, output) {
            Ok(_) => {
                let rendered_content = format!(
                    "[TOOL OUTPUT EXCEEDED INLINE BUDGET]\n\
                    The output for tool '{tool_name}' was {char_count} characters (exceeding the {threshold} inline limit).\n\
                    The complete raw output has been saved to:\n\
                    {path}\n\n\
                    To examine this output efficiently:\n\
                    1. Use `grep` with a regex pattern or `include_pattern` to find specific keywords or error strings.\n\
                    2. Use `read_file` with `start_line` and `end_line` parameters to inspect specific sections.",
                    tool_name = tool_name,
                    char_count = char_count,
                    threshold = self.threshold,
                    path = spill_path.display()
                );

                SpilledOutputResult {
                    is_spilled: true,
                    rendered_content,
                    spill_path: Some(spill_path),
                    original_char_count: char_count,
                }
            }
            Err(err) => {
                log::error!("Failed to write spilled output to {:?}: {}", spill_path, err);
                self.build_fallback_result(tool_name, output, char_count, err.to_string())
            }
        }
    }

    fn build_fallback_result(
        &self,
        tool_name: &str,
        output: &str,
        char_count: usize,
        err_msg: String,
    ) -> SpilledOutputResult {
        // In case of disk error, keep first 5000 and last 2000 chars with an explicit warning
        let head: String = output.chars().take(5000).collect();
        let tail: String = output.chars().skip(char_count.saturating_sub(2000)).collect();
        let rendered_content = format!(
            "[WARNING: TOOL OUTPUT EXCEEDED BUDGET AND DISK WRITE FAILED: {err_msg}]\n\
            Tool '{tool_name}' returned {char_count} chars. Truncated inline excerpt:\n\
            {head}\n\n[... truncated {skipped} chars ...]\n\n{tail}",
            skipped = char_count.saturating_sub(7000)
        );

        SpilledOutputResult {
            is_spilled: false,
            rendered_content,
            spill_path: None,
            original_char_count: char_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_small_output_passes_through_unmodified() {
        let temp_dir = TempDir::new().unwrap();
        let spiller = LargeOutputSpiller::new(Some(temp_dir.path().to_path_buf()));

        let text = "Small tool execution result from cargo check";
        let res = spiller.process_output("terminal", text);

        assert!(!res.is_spilled);
        assert_eq!(res.rendered_content, text);
        assert_eq!(res.spill_path, None);
        assert_eq!(res.original_char_count, text.chars().count());
    }

    #[test]
    fn test_oversized_output_spills_to_disk() {
        let temp_dir = TempDir::new().unwrap();
        let spiller = LargeOutputSpiller::with_threshold(1000, Some(temp_dir.path().to_path_buf()));

        let large_payload = "A".repeat(2500);
        let res = spiller.process_output("read_file", &large_payload);

        assert!(res.is_spilled);
        assert_eq!(res.original_char_count, 2500);
        assert!(res.spill_path.is_some());

        let spill_path = res.spill_path.unwrap();
        assert!(spill_path.exists());
        assert!(spill_path.to_string_lossy().contains(".zenthree/tmp/tool_spill_"));

        // Content of file matches original payload
        let disk_content = fs::read_to_string(&spill_path).unwrap();
        assert_eq!(disk_content, large_payload);

        // Rendered content contains instructions and path
        assert!(res.rendered_content.contains("[TOOL OUTPUT EXCEEDED INLINE BUDGET]"));
        assert!(res.rendered_content.contains("read_file"));
        assert!(res.rendered_content.contains(&spill_path.display().to_string()));
    }
}
