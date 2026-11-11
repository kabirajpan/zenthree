use std::fs;
use std::path::{Path, PathBuf};
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Individual replacement chunk in an atomic multi-replace operation.
#[derive(Debug, Clone)]
pub struct ReplacementChunk {
    pub start_line: usize,
    pub end_line: usize,
    pub target_content: String,
    pub replacement_content: String,
    pub allow_multiple: bool,
}

/// Tool for executing atomic, non-contiguous multi-block code edits in a single pass.
/// Implements reverse-engineered Antigravity / Cortex `multi_replace_file_content` specification.
#[derive(Default)]
pub struct MultiReplaceFileContentTool {
    workspace_root: Option<PathBuf>,
}

impl MultiReplaceFileContentTool {
    pub fn new() -> Self {
        Self { workspace_root: None }
    }

    pub fn with_workspace(mut self, root: PathBuf) -> Self {
        self.workspace_root = Some(root);
        self
    }
}

impl Tool for MultiReplaceFileContentTool {
    fn name(&self) -> &str {
        "multi_replace_file_content"
    }

    fn description(&self) -> &str {
        "Apply multiple non-contiguous edits to a single file atomically in one pass. Enforces exact character sequence matching and range boundaries without line offset drift."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "TargetFile": {
                    "type": "string",
                    "description": "Path to the target file to modify"
                },
                "Instruction": {
                    "type": "string",
                    "description": "Brief description of the changes being made"
                },
                "Description": {
                    "type": "string",
                    "description": "User-facing explanation of design decisions and rationale"
                },
                "ReplacementChunks": {
                    "type": "array",
                    "description": "List of non-overlapping chunks to replace in the file",
                    "items": {
                        "type": "object",
                        "properties": {
                            "StartLine": {
                                "type": "integer",
                                "description": "1-indexed starting line number of the search window"
                            },
                            "EndLine": {
                                "type": "integer",
                                "description": "1-indexed ending line number of the search window (inclusive)"
                            },
                            "TargetContent": {
                                "type": "string",
                                "description": "Exact character sequence to be replaced (must match verbatim)"
                            },
                            "ReplacementContent": {
                                "type": "string",
                                "description": "New content to replace the target content with"
                            },
                            "AllowMultiple": {
                                "type": "boolean",
                                "description": "If true, multiple occurrences within the range are replaced; otherwise multiple matches produce an error (default: false)"
                            }
                        },
                        "required": ["StartLine", "EndLine", "TargetContent", "ReplacementContent"]
                    }
                }
            },
            "required": ["TargetFile", "ReplacementChunks"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::Mutating
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let path_str = args.get("TargetFile")
            .or_else(|| args.get("path"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'TargetFile'".to_string()))?;

        let chunks_arr = args.get("ReplacementChunks")
            .or_else(|| args.get("replacement_chunks"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required array 'ReplacementChunks'".to_string()))?;

        if chunks_arr.is_empty() {
            return Err(ToolError::InvalidArgs("ReplacementChunks cannot be empty".to_string()));
        }

        let rel_norm = path_str.replace('\\', "/");
        if rel_norm.contains(".zenthree_sessions") || rel_norm.contains(".zenthree_state.json") {
            return Err(ToolError::ExecutionFailed(
                "Access denied: Cannot edit internal IDE runtime files.".to_string(),
            ));
        }

        let resolved = if let Some(ref ws) = self.workspace_root {
            if Path::new(path_str).is_absolute() {
                PathBuf::from(path_str)
            } else {
                ws.join(path_str)
            }
        } else {
            PathBuf::from(path_str)
        };

        if let Some(ref ws) = self.workspace_root {
            let canon_ws = ws.canonicalize().unwrap_or_else(|_| ws.clone());
            let canon_res = resolved.canonicalize().unwrap_or_else(|_| resolved.clone());
            if !canon_res.starts_with(&canon_ws) {
                return Err(ToolError::ExecutionFailed(format!(
                    "Access denied: Path '{}' is outside the project workspace boundary ('{}').",
                    path_str,
                    ws.display()
                )));
            }
        }

        let path = &resolved;
        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!("File not found: {}", path_str)));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read '{}': {}", path_str, e)))?;

        let original_lines: Vec<&str> = content.lines().collect();
        let total_lines = original_lines.len();

        let mut chunks: Vec<ReplacementChunk> = Vec::new();

        for (idx, item) in chunks_arr.iter().enumerate() {
            let start = item.get("StartLine")
                .or_else(|| item.get("start_line"))
                .and_then(|v| v.as_u64())
                .ok_or_else(|| ToolError::InvalidArgs(format!("Chunk {}: Missing StartLine", idx + 1)))? as usize;

            let end = item.get("EndLine")
                .or_else(|| item.get("end_line"))
                .and_then(|v| v.as_u64())
                .ok_or_else(|| ToolError::InvalidArgs(format!("Chunk {}: Missing EndLine", idx + 1)))? as usize;

            let target = item.get("TargetContent")
                .or_else(|| item.get("target_content"))
                .or_else(|| item.get("target"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArgs(format!("Chunk {}: Missing TargetContent", idx + 1)))?
                .to_string();

            let replacement = item.get("ReplacementContent")
                .or_else(|| item.get("replacement_content"))
                .or_else(|| item.get("replacement"))
                .and_then(|v| v.as_str())
                .ok_or_else(|| ToolError::InvalidArgs(format!("Chunk {}: Missing ReplacementContent", idx + 1)))?
                .to_string();

            let allow_multiple = item.get("AllowMultiple")
                .or_else(|| item.get("allow_multiple"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            if start == 0 {
                return Err(ToolError::InvalidArgs(format!(
                    "Chunk {}: StartLine must be 1-indexed (>= 1)",
                    idx + 1
                )));
            }

            if start > end {
                return Err(ToolError::InvalidArgs(format!(
                    "Chunk {}: StartLine ({}) cannot be greater than EndLine ({})",
                    idx + 1, start, end
                )));
            }

            if end > total_lines {
                return Err(ToolError::InvalidArgs(format!(
                    "Chunk {}: EndLine ({}) exceeds total file lines ({}) in '{}'",
                    idx + 1, end, total_lines, path_str
                )));
            }

            chunks.push(ReplacementChunk {
                start_line: start,
                end_line: end,
                target_content: target,
                replacement_content: replacement,
                allow_multiple,
            });
        }

        // Validate chunks are strictly non-overlapping
        let mut sorted_by_start = chunks.clone();
        sorted_by_start.sort_by_key(|c| c.start_line);
        for i in 0..sorted_by_start.len().saturating_sub(1) {
            if sorted_by_start[i].end_line >= sorted_by_start[i + 1].start_line {
                return Err(ToolError::InvalidArgs(format!(
                    "Overlapping replacement chunks detected: [{}, {}] overlaps with [{}, {}]. Chunks must be disjoint.",
                    sorted_by_start[i].start_line, sorted_by_start[i].end_line,
                    sorted_by_start[i + 1].start_line, sorted_by_start[i + 1].end_line
                )));
            }
        }

        // Apply edits in reverse order (bottom to top) to ensure line number invariance
        let mut working_lines: Vec<String> = original_lines.into_iter().map(|s| s.to_string()).collect();

        // Sort descending by start_line
        chunks.sort_by(|a, b| b.start_line.cmp(&a.start_line));

        for chunk in &chunks {
            let slice_start = chunk.start_line - 1;
            let slice_end = chunk.end_line; // exclusive range for slicing

            let target_slice = working_lines[slice_start..slice_end].join("\n");

            if !target_slice.contains(&chunk.target_content) {
                return Err(ToolError::ExecutionFailed(format!(
                    "TargetContent not found in lines {}-{} of '{}'. Check whitespace and exact characters.",
                    chunk.start_line, chunk.end_line, path_str
                )));
            }

            let occurrences = target_slice.matches(&chunk.target_content).count();
            if !chunk.allow_multiple && occurrences > 1 {
                return Err(ToolError::ExecutionFailed(format!(
                    "TargetContent matched {} times in lines {}-{} of '{}'. Narrow the line range or set AllowMultiple: true.",
                    occurrences, chunk.start_line, chunk.end_line, path_str
                )));
            }

            let patched_slice = if chunk.allow_multiple {
                target_slice.replace(&chunk.target_content, &chunk.replacement_content)
            } else {
                target_slice.replacen(&chunk.target_content, &chunk.replacement_content, 1)
            };

            let replacement_lines: Vec<String> = patched_slice.lines().map(|s| s.to_string()).collect();

            // Splice new lines into working_lines
            working_lines.splice(slice_start..slice_end, replacement_lines);
        }

        let mut final_content = working_lines.join("\n");
        // Preserve trailing newline if original content had one
        if content.ends_with('\n') {
            final_content.push('\n');
        }

        fs::write(path, final_content)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to write patched file '{}': {}", path_str, e)))?;

        let chunk_count = chunks.len();
        Ok(Value::String(format!(
            "Successfully applied {} replacement chunk(s) to '{}' atomically",
            chunk_count, path_str
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_multi_replace_file_content_atomic() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("code.rs");

        let initial_content = "fn first() {\n    let a = 1;\n}\n\nfn second() {\n    let b = 2;\n}\n";
        fs::write(&file_path, initial_content).unwrap();

        let tool = MultiReplaceFileContentTool::new().with_workspace(dir.path().to_path_buf());

        let res = tool.execute(json!({
            "TargetFile": file_path.to_str().unwrap(),
            "Instruction": "Update a and b constants",
            "ReplacementChunks": [
                {
                    "StartLine": 1,
                    "EndLine": 3,
                    "TargetContent": "let a = 1;",
                    "ReplacementContent": "let a = 100;"
                },
                {
                    "StartLine": 5,
                    "EndLine": 7,
                    "TargetContent": "let b = 2;",
                    "ReplacementContent": "let b = 200;"
                }
            ]
        }));

        assert!(res.is_ok(), "Multi-replace failed: {:?}", res.err());
        let updated = fs::read_to_string(&file_path).unwrap();
        assert!(updated.contains("let a = 100;"));
        assert!(updated.contains("let b = 200;"));
    }

    #[test]
    fn test_multi_replace_rejects_overlapping_chunks() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("overlap.rs");
        fs::write(&file_path, "line 1\nline 2\nline 3\nline 4\nline 5\n").unwrap();

        let tool = MultiReplaceFileContentTool::new().with_workspace(dir.path().to_path_buf());

        let res = tool.execute(json!({
            "TargetFile": file_path.to_str().unwrap(),
            "ReplacementChunks": [
                {
                    "StartLine": 1,
                    "EndLine": 3,
                    "TargetContent": "line 2",
                    "ReplacementContent": "line 2 modified"
                },
                {
                    "StartLine": 3,
                    "EndLine": 5,
                    "TargetContent": "line 4",
                    "ReplacementContent": "line 4 modified"
                }
            ]
        }));

        assert!(res.is_err());
        let err_msg = format!("{:?}", res.err());
        assert!(err_msg.contains("Overlapping"));
    }
}
