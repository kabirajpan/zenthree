use std::fs;
use std::path::Path;
use serde_json::{json, Value};
use crate::traits::tool::{Tool, ToolRisk};
use crate::types::error::ToolError;

/// Tool for reading file contents safely, with optional 1-indexed line range slicing.
pub struct ReadFileTool;

impl Tool for ReadFileTool {
    fn name(&self) -> &str {
        "read_file"
    }

    fn description(&self) -> &str {
        "Read the contents of a file. Specify start_line and end_line (1-indexed) to read a specific slice. Compact by default for large files (>300 lines); use show_all: true to read entire file."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Relative or absolute path to the file"
                },
                "start_line": {
                    "type": "integer",
                    "description": "Optional 1-indexed start line number"
                },
                "end_line": {
                    "type": "integer",
                    "description": "Optional 1-indexed end line number (inclusive)"
                },
                "show_all": {
                    "type": "boolean",
                    "description": "If true, reads the entire file even if large without line capping (defaults to false)"
                }
            },
            "required": ["path"]
        })
    }

    fn risk(&self) -> ToolRisk {
        ToolRisk::ReadOnly
    }

    fn execute(&self, args: Value) -> Result<Value, ToolError> {
        let path_str = args.get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ToolError::InvalidArgs("Missing required parameter 'path'".to_string()))?;

        let path = Path::new(path_str);
        if !path.exists() {
            return Err(ToolError::ExecutionFailed(format!("File not found: {}", path_str)));
        }
        if path.is_dir() {
            return Err(ToolError::ExecutionFailed(format!("Path is a directory, not a file: {}", path_str)));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| ToolError::ExecutionFailed(format!("Failed to read '{}': {}", path_str, e)))?;

        let start_line = args.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
        let end_line = args.get("end_line").and_then(|v| v.as_u64()).map(|v| v as usize);
        let show_all = args.get("show_all").and_then(|v| v.as_bool()).unwrap_or(false);

        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        if start_line.is_some() || end_line.is_some() {
            let start = start_line.unwrap_or(1).saturating_sub(1);
            let end = end_line.unwrap_or(total_lines).min(total_lines);

            if start >= total_lines {
                return Ok(Value::String(String::new()));
            }

            let slice = &lines[start..end];
            let mut formatted = String::new();
            for (idx, line) in slice.iter().enumerate() {
                formatted.push_str(&format!("{:4} | {}\n", start + idx + 1, line));
            }
            formatted.push_str("\n// Note: Line numbers (e.g. '  10 | ') are shown for reference. When calling `edit_file`, provide only the original raw code in `target` without line number prefixes.");
            Ok(Value::String(formatted))

        } else if !show_all && total_lines > 300 {
            // Default compact protection: cap at first 300 lines with guidance
            let slice = &lines[0..300];
            let mut formatted = format!(
                "// [File has {} lines. Displaying lines 1..300. Use start_line/end_line or show_all: true to view remaining lines.]\n",
                total_lines
            );
            for (idx, line) in slice.iter().enumerate() {
                formatted.push_str(&format!("{:4} | {}\n", idx + 1, line));
            }
            formatted.push_str(&format!(
                "\n// ... ({} more lines omitted. Use start_line: 301 or show_all: true to read further)\n",
                total_lines - 300
            ));
            Ok(Value::String(formatted))
        } else {
            Ok(Value::String(content))
        }
    }
}
