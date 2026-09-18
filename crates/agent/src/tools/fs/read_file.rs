use crate::environment::{ProtectedPathGuard, WorkspaceJail};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::path::Path;

/// Input parameters for the `read_file` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadFileInput {
    pub path: String,
    #[serde(default)]
    pub start_line: Option<usize>,
    #[serde(default)]
    pub end_line: Option<usize>,
}

/// Output returned by the `read_file` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadFileOutput {
    pub path: String,
    pub content: String,
    pub total_lines: usize,
    pub start_line: usize,
    pub end_line: usize,
}

/// Clamps optional line numbers to a valid 1-indexed inclusive range.
///
/// Models occasionally pass 0 despite instructions, or invert start/end bounds.
/// This matches Zed's `resolve_line_range` coordinate normalizer.
pub fn resolve_line_range(start_line: Option<usize>, end_line: Option<usize>) -> (usize, usize) {
    let start = start_line.unwrap_or(1).max(1);
    let end = end_line.unwrap_or(usize::MAX).max(start);
    (start, end)
}

/// Formats text lines in `cat -n` style: right-aligned in 6 chars + tab.
pub fn format_with_line_numbers(lines: &[&str], start_line: usize) -> String {
    let mut output = String::new();
    for (idx, line) in lines.iter().enumerate() {
        let line_no = start_line + idx;
        let _ = writeln!(output, "{line_no:>6}\t{line}");
    }
    output
}

/// Reads file contents safely, enforcing workspace jail and line range slicing.
pub fn execute_read_file(
    workspace_root: &Path,
    jail: Option<&WorkspaceJail>,
    relative_path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> Result<ReadFileOutput, String> {
    let jail_storage;
    let effective_jail = match jail {
        Some(j) => j,
        None => {
            jail_storage = WorkspaceJail::new([workspace_root]);
            &jail_storage
        }
    };

    let resolved_path = match effective_jail.validate_read(Path::new(relative_path)) {
        Ok(p) => p,
        Err(violation) => {
            return Err(format!("Security denial: {}", violation));
        }
    };

    // 2. Sensitive path inspection
    if ProtectedPathGuard::is_sensitive_read(&resolved_path) {
        log::warn!("Reading sensitive credential file: {}", resolved_path.display());
    }

    // 3. Read file content
    let raw_content = std::fs::read_to_string(&resolved_path)
        .map_err(|e| format!("Failed to read file `{}`: {}", relative_path, e))?;

    let all_lines: Vec<&str> = raw_content.lines().collect();
    let total_lines = all_lines.len();

    let (start, end) = resolve_line_range(start_line, end_line);

    if total_lines == 0 {
        return Ok(ReadFileOutput {
            path: relative_path.to_string(),
            content: String::new(),
            total_lines: 0,
            start_line: 1,
            end_line: 1,
        });
    }

    let zero_start = start.saturating_sub(1).min(total_lines);
    let zero_end = end.min(total_lines);

    let sliced_lines = if zero_start >= total_lines {
        &[]
    } else {
        &all_lines[zero_start..zero_end]
    };

    let formatted = format_with_line_numbers(sliced_lines, start);

    Ok(ReadFileOutput {
        path: relative_path.to_string(),
        content: formatted,
        total_lines,
        start_line: start,
        end_line: zero_end,
    })
}

/// Returns the JSON Schema specification for `read_file`.
pub fn read_file_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "read_file",
        "description": "Reads file contents with 1-indexed line numbers in `cat -n` format (right-aligned line numbers + tab).\n\nUse this tool before editing files to understand existing code and exact line numbers.\n\n<example>\n{\"tool\": \"read_file\", \"path\": \"crates/agent/src/lib.rs\", \"start_line\": 1, \"end_line\": 60}\n</example>",
        "parameters": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project-relative path to the file to read."
                },
                "start_line": {
                    "type": "integer",
                    "description": "1-based starting line number (optional, clamped >= 1)."
                },
                "end_line": {
                    "type": "integer",
                    "description": "1-based ending line number (optional, clamped >= start_line)."
                }
            },
            "required": ["path"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_resolve_line_range_clamping() {
        // start 0 clamped to 1
        assert_eq!(resolve_line_range(Some(0), Some(10)), (1, 10));
        // end < start clamped to start
        assert_eq!(resolve_line_range(Some(15), Some(5)), (15, 15));
        // none defaults
        assert_eq!(resolve_line_range(None, None), (1, usize::MAX));
    }

    #[test]
    fn test_execute_read_file_line_numbers() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        fs::write(&file_path, "line one\nline two\nline three\nline four\n").unwrap();

        let res = execute_read_file(dir.path(), None, "test.txt", Some(2), Some(3)).unwrap();
        assert_eq!(res.total_lines, 4);
        assert_eq!(res.start_line, 2);
        assert_eq!(res.end_line, 3);
        assert!(res.content.contains("     2\tline two"));
        assert!(res.content.contains("     3\tline three"));
        assert!(!res.content.contains("line one"));
    }

    #[test]
    fn test_execute_read_file_jail_violation() {
        let dir = tempdir().unwrap();
        let res = execute_read_file(dir.path(), None, "../outside.txt", None, None);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Security denial"));
    }
}
