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
        .or_else(|| val.get("action"))
        .and_then(|v| v.as_str());

    let params = val
        .get("parameters")
        .or_else(|| val.get("input"))
        .or_else(|| val.get("arguments"))
        .unwrap_or(val);

    if let Some(tool) = tool_name {
        match tool {
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
            "run_command" | "exec" | "terminal" | "bash" | "sh" | "shell" | "run" | "execute" | "execute_command" => {
                let cmd = params
                    .get("cmd")
                    .or_else(|| params.get("command"))
                    .or_else(|| params.get("input"))
                    .or_else(|| params.get("code"))
                    .or_else(|| params.get("script"))
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
} else {
    // Fallback: Check if object directly specifies cmd/command
    if let Some(cmd) = val.get("cmd").or_else(|| val.get("command")).or_else(|| val.get("run")).and_then(|v| v.as_str()) {
        return Some(AgentToolCall::RunCommand {
            cmd: cmd.trim().to_string(),
        });
    }
    None
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
                || candidate.contains("\"action\"")
                || candidate.contains("\"cmd\"")
                || candidate.contains("\"command\"")
            {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(candidate) {
                    if let Some(call) = parse_single_tool_call_value(&val) {
                        if !results.iter().any(|r| r == &call) {
                            results.push(call);
                        }
                    }
                } else if candidate.contains("run_command") || candidate.contains("\"cmd\"") || candidate.contains("\"command\"") {
                    if let Some(call) = extract_lenient_run_command(candidate) {
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

    // 3. Fallback: Parse XML command tags like <run_command>cmd</run_command> or <command>cmd</command>
    for tag in ["run_command", "command", "bash", "terminal"] {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);
        let mut cur = content;
        while let Some(start) = cur.find(&open_tag) {
            let after = &cur[start + open_tag.len()..];
            if let Some(end) = after.find(&close_tag) {
                let cmd = after[..end].trim();
                if !cmd.is_empty() {
                    let call = AgentToolCall::RunCommand { cmd: cmd.to_string() };
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                }
                cur = &after[end + close_tag.len()..];
            } else {
                break;
            }
        }
    }

    // 4. Fallback: Parse markdown command directives like ### Run: <cmd> or ### Command: <cmd>
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("### Run:")
            .or_else(|| trimmed.strip_prefix("### Command:"))
            .or_else(|| trimmed.strip_prefix("### Exec:"))
        {
            let cmd = rest.trim().trim_matches('`').trim();
            if !cmd.is_empty() {
                let call = AgentToolCall::RunCommand { cmd: cmd.to_string() };
                if !results.iter().any(|r| r == &call) {
                    results.push(call);
                }
            }
        }
    }

    // 5. Fallback: Parse runnable commands from fenced ```bash ... ``` or ```sh ... ``` code blocks
    for block in content.split("```") {
        let trimmed = block.trim();
        let sh_code = if let Some(rest) = trimmed.strip_prefix("bash") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("sh") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("shell") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("terminal") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("zsh") {
            Some(rest)
        } else {
            None
        };

        if let Some(code) = sh_code {
            let lines: Vec<String> = code
                .lines()
                .map(|l| l.trim())
                .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with("//"))
                .map(|l| {
                    l.strip_prefix("$ ")
                        .or_else(|| l.strip_prefix("> "))
                        .unwrap_or(l)
                        .trim()
                        .to_string()
                })
                .filter(|l| !l.is_empty())
                .collect();

            if !lines.is_empty() {
                let full_cmd = lines.join(" && ");
                let call = AgentToolCall::RunCommand { cmd: full_cmd };
                if !results.iter().any(|r| r == &call) {
                    results.push(call);
                }
            }
        }
    }

    results
}

fn extract_lenient_run_command(text: &str) -> Option<AgentToolCall> {
    let cmd_pos = text.find("\"cmd\"").or_else(|| text.find("\"command\""))?;
    let after_cmd = &text[cmd_pos..];
    let colon_pos = after_cmd.find(':')?;
    let val_part = after_cmd[colon_pos + 1..].trim();
    let cmd_body = if let Some(stripped) = val_part.strip_prefix('"') {
        if let Some(end_q) = stripped.rfind('"') {
            &stripped[..end_q]
        } else {
            stripped.trim_end_matches('}').trim()
        }
    } else {
        val_part.trim_end_matches('}').trim()
    };

    let cleaned = cmd_body.trim();
    if !cleaned.is_empty() {
        Some(AgentToolCall::RunCommand { cmd: cleaned.to_string() })
    } else {
        None
    }
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
        } else if let Some(rest) = trimmed.strip_prefix("/* File:") {
            Some(rest.trim_end_matches("*/").trim())
        } else if let Some(rest) = trimmed.strip_prefix("/* file:") {
            Some(rest.trim_end_matches("*/").trim())
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            let candidate = rest.trim().trim_matches('`').trim_matches('*').trim();
            if is_likely_filepath(candidate) {
                Some(candidate)
            } else {
                None
            }
        } else if trimmed.starts_with("**") && trimmed.ends_with("**") && trimmed.len() > 4 {
            let candidate = trimmed[2..trimmed.len() - 2].trim().trim_matches('`').trim();
            if is_likely_filepath(candidate) {
                Some(candidate)
            } else {
                None
            }
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

pub fn is_likely_filepath(s: &str) -> bool {
    let lower = s.to_lowercase();
    let has_ext = lower.ends_with(".rs")
        || lower.ends_with(".jsx")
        || lower.ends_with(".js")
        || lower.ends_with(".tsx")
        || lower.ends_with(".ts")
        || lower.ends_with(".css")
        || lower.ends_with(".html")
        || lower.ends_with(".json")
        || lower.ends_with(".toml")
        || lower.ends_with(".py")
        || lower.ends_with(".go")
        || lower.ends_with(".md")
        || lower.ends_with(".sh")
        || lower.ends_with(".yaml")
        || lower.ends_with(".yml");
    has_ext && !s.contains(' ') && s.len() >= 3
}
