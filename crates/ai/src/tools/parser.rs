use std::path::{Path, PathBuf};
use super::patch::{FileEditAction, SearchReplaceBlock, TargetFileModification};
use super::AgentToolCall;

fn is_search_marker(trimmed: &str) -> bool {
    let s = trimmed.trim_start_matches("//").trim_start_matches("/*").trim();
    s.starts_with("<<<<<<<") && s.to_uppercase().contains("SEARCH")
}

fn is_divider_marker(trimmed: &str) -> bool {
    let s = trimmed.trim_start_matches("//").trim_start_matches("/*").trim();
    s.starts_with("=======")
}

fn is_replace_marker(trimmed: &str) -> bool {
    let s = trimmed.trim_start_matches("//").trim_start_matches("/*").trim();
    s.starts_with(">>>>>>>")
}

#[derive(PartialEq)]
enum ParseState {
    Outside,
    InSearch,
    InReplace,
}

/// Parses surgical <<<<<<< SEARCH ... ======= ... >>>>>>> REPLACE blocks from text
pub fn parse_search_replace_blocks(text: &str) -> Vec<SearchReplaceBlock> {
    let mut blocks = Vec::new();
    let mut state = ParseState::Outside;
    let mut search_lines = Vec::new();
    let mut replace_lines = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        match state {
            ParseState::Outside => {
                if is_search_marker(trimmed) {
                    state = ParseState::InSearch;
                    search_lines.clear();
                    replace_lines.clear();
                }
            }
            ParseState::InSearch => {
                if is_divider_marker(trimmed) {
                    state = ParseState::InReplace;
                } else {
                    search_lines.push(line);
                }
            }
            ParseState::InReplace => {
                if is_replace_marker(trimmed) {
                    blocks.push(SearchReplaceBlock {
                        search: search_lines.join("\n"),
                        replace: replace_lines.join("\n"),
                    });
                    state = ParseState::Outside;
                    search_lines.clear();
                    replace_lines.clear();
                } else {
                    replace_lines.push(line);
                }
            }
        }
    }

    if state == ParseState::InReplace && (!search_lines.is_empty() || !replace_lines.is_empty()) {
        blocks.push(SearchReplaceBlock {
            search: search_lines.join("\n"),
            replace: replace_lines.join("\n"),
        });
    }

    blocks
}

/// Helper to extract structured AgentToolCall from any JSON representation
pub fn parse_single_tool_call_value(val: &serde_json::Value) -> Option<AgentToolCall> {
    let tool_name = val
        .get("tool")
        .or_else(|| val.get("name"))
        .or_else(|| val.get("function"))
        .and_then(|v| v.as_str())?;

    let params = val
        .get("parameters")
        .or_else(|| val.get("input"))
        .or_else(|| val.get("arguments"))
        .unwrap_or(val);

    match tool_name {
        "read_file" => {
            let path = params
                .get("path")
                .or_else(|| params.get("file_path"))
                .or_else(|| params.get("file"))
                .and_then(|v| v.as_str())?;
            let start_line = params.get("start_line").and_then(|v| v.as_u64()).map(|v| v as usize);
            let end_line = params.get("end_line").and_then(|v| v.as_u64()).map(|v| v as usize);
            Some(AgentToolCall::ReadFile {
                path: path.trim().trim_matches('"').to_string(),
                start_line,
                end_line,
            })
        }
        "search_code" | "grep" | "search" => {
            let query = params
                .get("query")
                .or_else(|| params.get("pattern"))
                .or_else(|| params.get("search"))
                .and_then(|v| v.as_str())?;
            Some(AgentToolCall::SearchCode {
                query: query.trim().to_string(),
            })
        }
        "list_directory" | "list_dir" | "ls" => {
            let path = params
                .get("path")
                .or_else(|| params.get("directory"))
                .or_else(|| params.get("dir"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim().trim_matches('"').to_string());
            Some(AgentToolCall::ListDirectory { path })
        }
        "run_command" | "exec" | "terminal" => {
            let cmd = params
                .get("cmd")
                .or_else(|| params.get("command"))
                .and_then(|v| v.as_str())?;
            Some(AgentToolCall::RunCommand {
                cmd: cmd.trim().to_string(),
            })
        }
        "edit_file" => {
            let path = params
                .get("path")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())?;
            let search = params
                .get("search")
                .or_else(|| params.get("old_str"))
                .and_then(|v| v.as_str())?;
            let replace = params
                .get("replace")
                .or_else(|| params.get("new_str"))
                .and_then(|v| v.as_str())?;
            Some(AgentToolCall::EditFile {
                path: path.trim().trim_matches('"').to_string(),
                search: search.to_string(),
                replace: replace.to_string(),
            })
        }
        "detect_environment" | "detect_toolchain" | "detect_project" => {
            let subpath = params
                .get("subpath")
                .or_else(|| params.get("path"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim().trim_matches('"').to_string());
            Some(AgentToolCall::DetectEnvironment { subpath })
        }
        "git_status" => Some(AgentToolCall::GitStatus),
        "git_diff" => {
            let staged_only = params
                .get("staged_only")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            Some(AgentToolCall::GitDiff { staged_only })
        }
        _ => None,
    }
}

/// Parses structured tool calls from model output (JSON blocks, inline JSON, or legacy tags)
pub fn parse_agent_tool_calls(content: &str) -> Vec<AgentToolCall> {
    let mut results = Vec::new();

    // 1. Check fenced code blocks (```json ... ``` or ``` ...)
    for block in content.split("```") {
        let trimmed = block.trim();
        let json_str = if let Some(rest) = trimmed.strip_prefix("json") {
            rest.trim()
        } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
            trimmed
        } else {
            continue;
        };

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            if let Some(arr) = val.as_array() {
                for item in arr {
                    if let Some(call) = parse_single_tool_call_value(item) {
                        results.push(call);
                    }
                }
            } else if let Some(call) = parse_single_tool_call_value(&val) {
                results.push(call);
            }
        }
    }

    // 2. Fallback: Parse standalone inline JSON objects like {"tool": "..."}
    let mut search_idx = 0;
    while let Some(open_idx) = content[search_idx..].find('{') {
        let actual_open = search_idx + open_idx;
        let mut depth = 0;
        let mut close_idx = None;

        for (i, c) in content[actual_open..].char_indices() {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(actual_open + i);
                    break;
                }
            }
        }

        if let Some(actual_close) = close_idx {
            let candidate = &content[actual_open..=actual_close];
            if candidate.contains("\"tool\"")
                || candidate.contains("\"function\"")
                || candidate.contains("\"name\"")
            {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                    if let Some(call) = parse_single_tool_call_value(&val) {
                        if !results.iter().any(|r| r == &call) {
                            results.push(call);
                        }
                    }
                }
            }
            search_idx = actual_close + 1;
        } else {
            break;
        }
    }

    results
}

pub fn extract_json_tool_modifications(
    content: &str,
    workspace_root: &Path,
) -> Vec<TargetFileModification> {
    let mut results = Vec::new();
    let calls = parse_agent_tool_calls(content);

    for call in calls {
        if let AgentToolCall::EditFile { path, search, replace } = call {
            let p = if Path::new(&path).is_absolute() {
                PathBuf::from(&path)
            } else {
                workspace_root.join(&path)
            };
            let block = SearchReplaceBlock { search, replace };
            if let Some(existing) = results.iter_mut().find(|m: &&mut TargetFileModification| m.file_path == p) {
                if let FileEditAction::SurgicalPatch(ref mut blocks) = existing.action {
                    blocks.push(block);
                }
            } else {
                results.push(TargetFileModification::new_patch(p, vec![block]));
            }
        }
    }

    results
}

pub fn ground_workspace_path(workspace_root: &Path, cleaned: &str) -> PathBuf {
    if Path::new(cleaned).is_absolute() {
        return PathBuf::from(cleaned);
    }
    let direct = workspace_root.join(cleaned);
    if direct.exists() {
        return direct;
    }
    if let Some(parent) = direct.parent() {
        if parent.exists() {
            return direct;
        }
    }
    // Search subfolders in workspace_root for matching subproject or file
    if let Ok(entries) = std::fs::read_dir(workspace_root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let candidate = p.join(cleaned);
                if candidate.exists() || candidate.parent().map(|par| par.exists()).unwrap_or(false) {
                    return candidate;
                }
            }
        }
    }
    direct
}

pub fn extract_all_file_modifications(
    content: &str,
    workspace_root: &Path,
    active_path: Option<&Path>,
) -> Vec<TargetFileModification> {
    let mut results = extract_json_tool_modifications(content, workspace_root);
    let mut current_target: Option<PathBuf> = None;
    let mut in_block = false;
    let mut code_lines = Vec::new();
    let mut raw_patch_lines = Vec::new();

    let flush_raw_patch = |target: &Option<PathBuf>, lines: &mut Vec<String>, res: &mut Vec<TargetFileModification>| {
        if lines.is_empty() {
            return;
        }
        let text = lines.join("\n");
        let blocks = parse_search_replace_blocks(&text);
        if !blocks.is_empty() {
            let target_file = target.clone().or_else(|| active_path.map(|p| p.to_path_buf()));
            if let Some(tf) = target_file {
                if let Some(existing) = res.iter_mut().find(|m| m.file_path == tf) {
                    if let FileEditAction::SurgicalPatch(ref mut b) = existing.action {
                        b.extend(blocks);
                    }
                } else {
                    res.push(TargetFileModification::new_patch(tf, blocks));
                }
            }
        }
        lines.clear();
    };

    for line in content.lines() {
        let trimmed = line.trim();

        let stripped = if let Some(rest) = trimmed.strip_prefix("### File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### file:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### Edit File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### Patch File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### Create File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### New File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("**File**:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("// File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("// file:") {
            Some(rest)
        } else {
            None
        };

        if let Some(candidate) = stripped {
            flush_raw_patch(&current_target, &mut raw_patch_lines, &mut results);

            let cleaned = candidate.trim().trim_matches('`').trim_matches('*').trim();
            if !cleaned.is_empty() {
                let path = ground_workspace_path(workspace_root, cleaned);
                current_target = Some(path);
            }
            continue;
        }

        if trimmed.starts_with("```") {
            flush_raw_patch(&current_target, &mut raw_patch_lines, &mut results);

            if in_block {
                if !code_lines.is_empty() {
                    let block_text = code_lines.join("\n");
                    let surgical_blocks = parse_search_replace_blocks(&block_text);

                    // A code block ONLY targets a file if:
                    // 1. current_target was explicitly declared (e.g. ### File: <path>), OR
                    // 2. It contains explicit surgical SEARCH/REPLACE blocks AND active_path exists.
                    // Plain code blocks without an explicit file header must NEVER overwrite active_path!
                    let target_file = if let Some(t) = current_target.take() {
                        Some(t)
                    } else if !surgical_blocks.is_empty() {
                        active_path.map(|p| p.to_path_buf())
                    } else {
                        None
                    };

                    if let Some(target_file) = target_file {
                        if !surgical_blocks.is_empty() {
                            if let Some(existing) = results.iter_mut().find(|m| m.file_path == target_file) {
                                if let FileEditAction::SurgicalPatch(ref mut b) = existing.action {
                                    b.extend(surgical_blocks);
                                }
                            } else {
                                results.push(TargetFileModification::new_patch(target_file, surgical_blocks));
                            }
                        } else {
                            results.push(TargetFileModification::new_overwrite(target_file, block_text));
                        }
                    }
                }
                in_block = false;
                code_lines.clear();
            } else {
                in_block = true;
                code_lines.clear();
            }
            continue;
        }

        if in_block {
            code_lines.push(line);
        } else {
            raw_patch_lines.push(line.to_string());
        }
    }

    flush_raw_patch(&current_target, &mut raw_patch_lines, &mut results);

    results
}
