use std::path::{Path, PathBuf};
use super::AgentToolResult;

pub fn execute_read_file(
    workspace: &Path,
    rel_path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> AgentToolResult {
    let cleaned = rel_path.trim().trim_matches('"').trim_matches('\'').trim();
    let direct = if Path::new(cleaned).is_absolute() {
        PathBuf::from(cleaned)
    } else {
        workspace.join(cleaned)
    };
    let full_path = if direct.exists() {
        direct
    } else {
        super::parser::ground_workspace_path_with_context(workspace, cleaned, None)
    };

    if let (Ok(can_ws), Ok(can_full)) = (workspace.canonicalize(), full_path.canonicalize()) {
        if !can_full.starts_with(&can_ws) {
            return AgentToolResult {
                tool_name: "read_file".to_string(),
                ok: false,
                output: format!("Access denied: Path `{}` is outside the workspace.", cleaned),
            };
        }
    }

    match std::fs::read_to_string(&full_path) {
        Ok(content) => {
            let lines: Vec<&str> = content.lines().collect();
            let total_lines = lines.len();

            let default_end = if start_line.is_none() && end_line.is_none() && total_lines > 250 {
                250
            } else {
                total_lines
            };

            let s_idx = start_line.unwrap_or(1).saturating_sub(1).min(total_lines);
            let e_idx = end_line.unwrap_or(default_end).min(total_lines);

            if s_idx >= e_idx && total_lines > 0 {
                return AgentToolResult {
                    tool_name: "read_file".to_string(),
                    ok: false,
                    output: format!(
                        "Invalid line range #{}-#{} (file has {} lines)",
                        s_idx + 1,
                        e_idx,
                        total_lines
                    ),
                };
            }

            let slice = &lines[s_idx..e_idx];
            let mut formatted = Vec::new();
            for (offset, line) in slice.iter().enumerate() {
                let line_no = s_idx + offset + 1;
                formatted.push(format!("{:4} | {}", line_no, line));
            }

            let (type_name, method) = crate::tools::environment::DetectorAgent::classify_file_quick(&full_path, &content);

            let mut output_str = format!(
                "File `{}` [Type: {} (via {})] (lines {}-{} of {}):\n{}",
                cleaned,
                type_name,
                method,
                s_idx + 1,
                e_idx,
                total_lines,
                formatted.join("\n")
            );

            if e_idx < total_lines && (start_line.is_none() || end_line.is_none()) {
                output_str.push_str(&format!(
                    "\n\n... [Showing lines {}-{} of {}. To inspect remaining code, call read_file with start_line={} and end_line={}]",
                    s_idx + 1,
                    e_idx,
                    total_lines,
                    e_idx + 1,
                    (e_idx + 250).min(total_lines)
                ));
            }

            AgentToolResult {
                tool_name: "read_file".to_string(),
                ok: true,
                output: output_str,
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "read_file".to_string(),
            ok: false,
            output: format!("Failed to read file `{}`: {}", cleaned, e),
        },
    }
}

pub fn extract_tool_call_read_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let lower = content.to_lowercase();
    let mut search_idx = 0;

    while let Some(pos) = lower[search_idx..].find("read_file") {
        let actual_pos = search_idx + pos;
        let window_end = (actual_pos + 400).min(content.len());
        let window = &content[actual_pos..window_end];

        if let Some(json_start) = window.find('{') {
            if let Some(json_end) = window[json_start..].find('}') {
                let json_str = &window[json_start..=json_start + json_end];
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    if let Some(p) = val.get("path").and_then(|v| v.as_str()) {
                        let trimmed = p.trim();
                        if !trimmed.is_empty() && !paths.contains(&trimmed.to_string()) {
                            paths.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        if let Some(param_pos) = window.find("file_paths") {
            let after_param = &window[param_pos..];
            if let Some(bracket_start) = after_param.find('[') {
                if let Some(bracket_end) = after_param[bracket_start..].find(']') {
                    let array_str = &after_param[bracket_start..=bracket_start + bracket_end];
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(array_str) {
                        if let Some(arr) = val.as_array() {
                            for item in arr {
                                if let Some(p) = item.as_str() {
                                    let trimmed = p.trim();
                                    if !trimmed.is_empty() && !paths.contains(&trimmed.to_string()) {
                                        paths.push(trimmed.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        search_idx = actual_pos + 9;
    }

    paths
}
