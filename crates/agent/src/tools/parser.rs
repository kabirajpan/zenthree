use std::collections::HashMap;
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
            "edit_file" | "patch_file" => {
                let path = params
                    .get("path")
                    .or_else(|| params.get("file_path"))
                    .or_else(|| params.get("file"))
                    .and_then(|v| v.as_str())?;
                let search = params
                    .get("search")
                    .or_else(|| params.get("old_str"))
                    .and_then(|v| v.as_str());
                let replace = params
                    .get("replace")
                    .or_else(|| params.get("new_str"))
                    .and_then(|v| v.as_str());

                if let (Some(s), Some(r)) = (search, replace) {
                    Some(AgentToolCall::EditFile {
                        path: path.trim().trim_matches('"').to_string(),
                        search: s.to_string(),
                        replace: r.to_string(),
                    })
                } else if let Some(edits_arr) = params.get("edits").and_then(|v| v.as_array()) {
                    if let Some(first) = edits_arr.first() {
                        let s = first.get("search").or_else(|| first.get("old_str")).and_then(|v| v.as_str())?;
                        let r = first.get("replace").or_else(|| first.get("new_str")).and_then(|v| v.as_str()).unwrap_or("");
                        Some(AgentToolCall::EditFile {
                            path: path.trim().trim_matches('"').to_string(),
                            search: s.to_string(),
                            replace: r.to_string(),
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            "write_file" | "create_file" | "new_file" | "overwrite_file" => {
                let path = params
                    .get("path")
                    .or_else(|| params.get("file_path"))
                    .or_else(|| params.get("file"))
                    .and_then(|v| v.as_str())?;
                let content = params
                    .get("content")
                    .or_else(|| params.get("code"))
                    .or_else(|| params.get("text"))
                    .or_else(|| params.get("contents"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                Some(AgentToolCall::WriteFile {
                    path: path.trim().trim_matches('"').to_string(),
                    content: content.to_string(),
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

/// Helper to parse one or more AgentToolCalls from a JSON value (supports "edits": [...] array)
pub fn parse_tool_call_values(val: &serde_json::Value) -> Vec<AgentToolCall> {
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
        if tool == "edit_file" || tool == "patch_file" {
            if let Some(path) = params.get("path").or_else(|| params.get("file_path")).or_else(|| params.get("file")).and_then(|v| v.as_str()) {
                if let Some(edits_arr) = params.get("edits").and_then(|v| v.as_array()) {
                    let mut list = Vec::new();
                    for edit in edits_arr {
                        if let Some(search) = edit.get("search").or_else(|| edit.get("old_str")).and_then(|v| v.as_str()) {
                            let replace = edit.get("replace").or_else(|| edit.get("new_str")).and_then(|v| v.as_str()).unwrap_or("");
                            list.push(AgentToolCall::EditFile {
                                path: path.trim().trim_matches('"').to_string(),
                                search: search.to_string(),
                                replace: replace.to_string(),
                            });
                        }
                    }
                    if !list.is_empty() {
                        return list;
                    }
                }
            }
        }
    }

    if let Some(single) = parse_single_tool_call_value(val) {
        vec![single]
    } else {
        Vec::new()
    }
}

/// Extracts function name from an XML tool call block (e.g. `<function=list_files>` or `<function name="list_files">`)
fn extract_xml_function_name(block: &str) -> Option<String> {
    // 1. Attribute-style: <function=list_files>, <invoke name="list_files">, <tool:write_file>
    let re_attr = regex::Regex::new(
        r#"(?i)<\s*(?:function|invoke|call|action|tool)(?:[\s=:_]+(?:name\s*=\s*)?["']?([a-zA-Z0-9_\-\.\/]+)["']?|\s+name\s*=\s*["']?([a-zA-Z0-9_\-\.\/]+)["']?)\s*>"#
    ).ok()?;
    if let Some(caps) = re_attr.captures(block) {
        if let Some(m) = caps.get(1).or_else(|| caps.get(2)) {
            let name = m.as_str().trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    // 2. Tag-style: <function>list_files</function>, <invoke>list_files</invoke>
    let re_tag = regex::Regex::new(
        r#"(?i)<\s*(?:function|invoke|call|action|tool)\s*>([a-zA-Z0-9_\-\.\/]+)<\s*/\s*(?:function|invoke|call|action|tool)\s*>"#
    ).ok()?;
    if let Some(caps) = re_tag.captures(block) {
        if let Some(m) = caps.get(1) {
            let name = m.as_str().trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    // 3. Name-style: <name=list_files>, <name>list_files</name>
    let re_name = regex::Regex::new(
        r#"(?i)<\s*name(?:[\s=:_]+["']?([a-zA-Z0-9_\-\.\/]+)["']?|\s*>([a-zA-Z0-9_\-\.\/]+)<\s*/\s*name\s*)>"#
    ).ok()?;
    if let Some(caps) = re_name.captures(block) {
        if let Some(m) = caps.get(1).or_else(|| caps.get(2)) {
            let name = m.as_str().trim();
            if !name.is_empty() {
                return Some(name.to_string());
            }
        }
    }

    None
}

/// Extracts parameter key-value pairs from an XML tool call block.
/// Supports both self-delimited `<parameter=key>val</parameter>` and consecutive `<parameter=key>val <parameter=key2>val2` or unclosed tags.
fn extract_xml_parameters_map(block: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    let param_start_re = match regex::Regex::new(
        r#"(?i)<\s*(?:parameter|param|arg|argument)(?:[\s=:_]+(?:name\s*=\s*)?["']?([a-zA-Z0-9_\-]+)["']?|\s+name\s*=\s*["']?([a-zA-Z0-9_\-]+)["']?)\s*>"#
    ) {
        Ok(r) => r,
        Err(_) => return map,
    };

    let param_close_re = match regex::Regex::new(
        r#"(?i)<\s*/\s*(?:parameter|param|arg|argument)\s*>"#
    ) {
        Ok(r) => r,
        Err(_) => return map,
    };

    let matches: Vec<_> = param_start_re.find_iter(block).collect();
    let caps_vec: Vec<_> = param_start_re.captures_iter(block).collect();

    for (i, (m, caps)) in matches.iter().zip(caps_vec.iter()).enumerate() {
        let param_name = match caps.get(1).or_else(|| caps.get(2)) {
            Some(name_match) => name_match.as_str().to_lowercase(),
            None => continue,
        };

        let val_start = m.end();
        let next_start = if i + 1 < matches.len() {
            Some(matches[i + 1].start())
        } else {
            None
        };

        let mut val_end = next_start.unwrap_or(block.len());

        // Check if there is an explicit closing parameter tag between val_start and val_end
        if let Some(close_m) = param_close_re.find_at(block, val_start) {
            if close_m.start() < val_end {
                val_end = close_m.start();
            }
        }

        if val_start <= val_end {
            let mut raw_val = block[val_start..val_end].trim();
            // Strip outer matching quotes/backticks if present
            if (raw_val.starts_with('"') && raw_val.ends_with('"') && raw_val.len() >= 2)
                || (raw_val.starts_with('\'') && raw_val.ends_with('\'') && raw_val.len() >= 2)
                || (raw_val.starts_with('`') && raw_val.ends_with('`') && raw_val.len() >= 2)
            {
                raw_val = &raw_val[1..raw_val.len() - 1];
            }
            map.insert(param_name, raw_val.to_string());
        }
    }

    // Also support direct child tags like <path>src/App.tsx</path> or <content>...</content>
    for tag in ["path", "file_path", "file", "content", "code", "text", "contents", "cmd", "command", "search", "replace", "query", "directory", "dir"] {
        if !map.contains_key(tag) {
            let re_child = regex::Regex::new(&format!(r#"(?is)<\s*{}\s*>([\s\S]*?)<\s*/\s*{}\s*>"#, tag, tag)).ok();
            if let Some(r) = re_child {
                if let Some(caps) = r.captures(block) {
                    if let Some(m) = caps.get(1) {
                        map.insert(tag.to_string(), m.as_str().trim().to_string());
                    }
                }
            }
        }
    }

    map
}

/// Maps a function name and extracted XML parameters to an AgentToolCall.
fn map_function_and_params_to_tool_call(
    func_name: &str,
    params: &HashMap<String, String>,
) -> Option<AgentToolCall> {
    let name_lower = func_name.trim().to_lowercase();
    match name_lower.as_str() {
        "list_files" | "list_directory" | "list_dir" | "ls" | "directory_list" | "dir_list" => {
            let path = params
                .get("path")
                .or_else(|| params.get("directory"))
                .or_else(|| params.get("dir"))
                .or_else(|| params.get("folder"))
                .map(|p| p.trim().trim_matches('"').to_string());
            let path = match path {
                Some(p) if p.is_empty() || p == "." || p == "./" => Some(".".to_string()),
                Some(p) => Some(p),
                None => Some(".".to_string()),
            };
            Some(AgentToolCall::ListDirectory { path })
        }
        "run_command" | "exec" | "terminal" | "bash" | "sh" | "shell" | "run" | "execute" | "execute_command" | "cmd" => {
            let cmd = params
                .get("cmd")
                .or_else(|| params.get("command"))
                .or_else(|| params.get("input"))
                .or_else(|| params.get("code"))
                .or_else(|| params.get("script"))?;
            Some(AgentToolCall::RunCommand {
                cmd: cmd.trim().to_string(),
            })
        }
        "read_file" | "view_file" | "cat" | "read" | "open_file" => {
            let path = params
                .get("path")
                .or_else(|| params.get("file_path"))
                .or_else(|| params.get("file"))?;
            let start_line = params.get("start_line").and_then(|s| s.parse::<usize>().ok());
            let end_line = params.get("end_line").and_then(|s| s.parse::<usize>().ok());
            Some(AgentToolCall::ReadFile {
                path: path.trim().trim_matches('"').to_string(),
                start_line,
                end_line,
            })
        }
        "write_file" | "create_file" | "new_file" | "overwrite_file" | "save_file" => {
            let path = params
                .get("path")
                .or_else(|| params.get("file_path"))
                .or_else(|| params.get("file"))?;
            let content = params
                .get("content")
                .or_else(|| params.get("code"))
                .or_else(|| params.get("text"))
                .or_else(|| params.get("contents"))
                .cloned()
                .unwrap_or_default();
            Some(AgentToolCall::WriteFile {
                path: path.trim().trim_matches('"').to_string(),
                content,
            })
        }
        "edit_file" | "patch_file" | "modify_file" | "replace_in_file" => {
            let path = params
                .get("path")
                .or_else(|| params.get("file_path"))
                .or_else(|| params.get("file"))?;
            let search = params
                .get("search")
                .or_else(|| params.get("old_str"))
                .or_else(|| params.get("old_code"))
                .or_else(|| params.get("find"))?;
            let replace = params
                .get("replace")
                .or_else(|| params.get("new_str"))
                .or_else(|| params.get("new_code"))
                .cloned()
                .unwrap_or_default();
            Some(AgentToolCall::EditFile {
                path: path.trim().trim_matches('"').to_string(),
                search: search.clone(),
                replace,
            })
        }
        "search_code" | "grep" | "search" | "find_in_files" => {
            let query = params
                .get("query")
                .or_else(|| params.get("pattern"))
                .or_else(|| params.get("search"))
                .or_else(|| params.get("text"))?;
            Some(AgentToolCall::SearchCode {
                query: query.trim().to_string(),
            })
        }
        "detect_environment" | "detect_toolchain" | "detect_project" => {
            let subpath = params
                .get("subpath")
                .or_else(|| params.get("path"))
                .map(|s| s.trim().trim_matches('"').to_string());
            Some(AgentToolCall::DetectEnvironment { subpath })
        }
        "git_status" => Some(AgentToolCall::GitStatus),
        "git_diff" => {
            let staged_only = params
                .get("staged_only")
                .map(|s| s == "true" || s == "1")
                .unwrap_or(false);
            Some(AgentToolCall::GitDiff { staged_only })
        }
        other => {
            if let Some(cmd) = params.get("cmd").or_else(|| params.get("command")) {
                return Some(AgentToolCall::RunCommand { cmd: cmd.trim().to_string() });
            }
            let json_map: serde_json::Map<String, serde_json::Value> = params
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            Some(AgentToolCall::Mcp {
                tool_name: other.to_string(),
                arguments: serde_json::Value::Object(json_map),
            })
        }
    }
}

/// Parses XML/pseudo-XML tool calls produced by models such as Qwen, DeepSeek, Claude, etc.
/// Handles forms like:
/// `< tool_call> < function=list_files> < parameter=path>. </ tool_call>`
/// `<tool_call>\n<function=run_command>\n<parameter=cmd>bun run dev</parameter>\n</tool_call>`
pub fn parse_xml_tool_calls(content: &str) -> Vec<AgentToolCall> {
    let mut results = Vec::new();

    let open_re = match regex::Regex::new(r"(?i)<\s*(?:tool_call|function_call)\b[^>]*>") {
        Ok(r) => r,
        Err(_) => return results,
    };
    let close_re = match regex::Regex::new(r"(?i)<\s*/\s*(?:tool_call|function_call)\s*>") {
        Ok(r) => r,
        Err(_) => return results,
    };

    let mut blocks = Vec::new();
    let mut start_search = 0;

    while let Some(m) = open_re.find_at(content, start_search) {
        let block_start = m.end();
        if let Some(close_m) = close_re.find_at(content, block_start) {
            let next_open = open_re.find_at(content, block_start);
            if let Some(next_o) = next_open {
                if next_o.start() < close_m.start() {
                    blocks.push(&content[block_start..next_o.start()]);
                    start_search = next_o.start();
                    continue;
                }
            }
            blocks.push(&content[block_start..close_m.start()]);
            start_search = close_m.end();
        } else {
            blocks.push(&content[block_start..]);
            break;
        }
    }

    // Fallback: If no <tool_call> tags were present, check if standalone <function=...> tags are present
    if blocks.is_empty() {
        let func_detect_re = regex::Regex::new(r#"(?i)<\s*function(?:[\s=:_]+(?:name\s*=\s*)?["']?[a-zA-Z0-9_\-\.\/]+|\s+name\s*=)"#).ok();
        if let Some(f_re) = func_detect_re {
            if f_re.is_match(content) {
                blocks.push(content);
            }
        }
    }

    let func_split_re = regex::Regex::new(
        r#"(?i)(?=<\s*function(?:[\s=:_]+(?:name\s*=\s*)?["']?[a-zA-Z0-9_\-\.\/]+["']?|\s+name\s*=\s*["']?[a-zA-Z0-9_\-\.\/]+["']?|\s*>))\s*"#
    ).ok();

    for block in blocks {
        let trimmed_block = block.trim();

        // 1. Check if the block contains JSON
        if trimmed_block.starts_with('{') || trimmed_block.starts_with('[') {
            let repaired = repair_json_newlines(trimmed_block);
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&repaired) {
                for call in parse_tool_call_values(&val) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                }
                continue;
            }
        }

        // 2. Sub-divide if multiple <function=...> tags exist in one block
        let sub_blocks: Vec<&str> = if let Some(ref splitter) = func_split_re {
            let parts: Vec<&str> = splitter.split(block).map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                vec![block]
            } else {
                parts
            }
        } else {
            vec![block]
        };

        for sub in sub_blocks {
            if let Some(func_name) = extract_xml_function_name(sub) {
                let params = extract_xml_parameters_map(sub);
                if let Some(call) = map_function_and_params_to_tool_call(&func_name, &params) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                }
            }
        }
    }

    results
}

/// Parses standalone XML tool tags like `<write_file path="...">content</write_file>`
/// or `<create_file path="...">content</create_file>`
pub fn parse_direct_xml_tags(content: &str) -> Vec<AgentToolCall> {
    let mut results = Vec::new();

    // 1. Direct write_file / create_file / new_file
    let write_re = regex::Regex::new(
        r#"(?is)<\s*(?:write_file|create_file|new_file)\b([^>]*)>([\s\S]*?)<\s*/\s*(?:write_file|create_file|new_file)\s*>"#
    ).ok();

    if let Some(re) = write_re {
        for caps in re.captures_iter(content) {
            let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            let path_attr_re = regex::Regex::new(r#"(?i)\b(?:path|file|file_path)\s*=\s*["']?([^"'\s>]+)["']?"#).ok();
            let mut path = path_attr_re.and_then(|r| r.captures(attrs)).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string());

            if path.is_none() {
                let path_tag_re = regex::Regex::new(r#"(?is)<\s*(?:path|file|file_path)\s*>([\s\S]*?)<\s*/\s*(?:path|file|file_path)\s*>"#).ok();
                path = path_tag_re.and_then(|r| r.captures(body)).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string());
            }

            if let Some(p) = path {
                let content_tag_re = regex::Regex::new(r#"(?is)<\s*(?:content|code|text)\s*>([\s\S]*?)<\s*/\s*(?:content|code|text)\s*>"#).ok();
                let file_content = if let Some(c_caps) = content_tag_re.and_then(|r| r.captures(body)) {
                    c_caps.get(1).map(|m| m.as_str().trim().to_string()).unwrap_or_default()
                } else {
                    body.trim().to_string()
                };

                results.push(AgentToolCall::WriteFile {
                    path: p.trim_matches('"').trim_matches('\'').to_string(),
                    content: file_content,
                });
            }
        }
    }

    // 2. Direct edit_file / patch_file
    let edit_re = regex::Regex::new(
        r#"(?is)<\s*(?:edit_file|patch_file)\b([^>]*)>([\s\S]*?)<\s*/\s*(?:edit_file|patch_file)\s*>"#
    ).ok();

    if let Some(re) = edit_re {
        for caps in re.captures_iter(content) {
            let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let body = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            let path_attr_re = regex::Regex::new(r#"(?i)\b(?:path|file|file_path)\s*=\s*["']?([^"'\s>]+)["']?"#).ok();
            let mut path = path_attr_re.and_then(|r| r.captures(attrs)).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string());

            if path.is_none() {
                let path_tag_re = regex::Regex::new(r#"(?is)<\s*(?:path|file|file_path)\s*>([\s\S]*?)<\s*/\s*(?:path|file|file_path)\s*>"#).ok();
                path = path_tag_re.and_then(|r| r.captures(body)).and_then(|c| c.get(1)).map(|m| m.as_str().trim().to_string());
            }

            if let Some(p) = path {
                let search_re = regex::Regex::new(r#"(?is)<\s*(?:search|old_str|old_code)\s*>([\s\S]*?)<\s*/\s*(?:search|old_str|old_code)\s*>"#).ok();
                let replace_re = regex::Regex::new(r#"(?is)<\s*(?:replace|new_str|new_code)\s*>([\s\S]*?)<\s*/\s*(?:replace|new_str|new_code)\s*>"#).ok();

                let search = search_re.and_then(|r| r.captures(body)).and_then(|c| c.get(1)).map(|m| m.as_str().to_string());
                let replace = replace_re.and_then(|r| r.captures(body)).and_then(|c| c.get(1)).map(|m| m.as_str().to_string()).unwrap_or_default();

                if let Some(s) = search {
                    results.push(AgentToolCall::EditFile {
                        path: p.trim_matches('"').trim_matches('\'').to_string(),
                        search: s,
                        replace,
                    });
                }
            }
        }
    }

    results
}

/// Parses structured tool calls from model output (JSON blocks, XML tags, inline JSON, or legacy tags)
pub fn parse_agent_tool_calls(content: &str) -> Vec<AgentToolCall> {
    let mut results = Vec::new();

    // 0. Priority: Parse XML tool calls (e.g. Qwen / DeepSeek / Claude pseudo-XML tags)
    for call in parse_xml_tool_calls(content) {
        if !results.iter().any(|r| r == &call) {
            results.push(call);
        }
    }

    // 0.1 Direct XML tags like <write_file path="..."> or <create_file path="...">
    for call in parse_direct_xml_tags(content) {
        if !results.iter().any(|r| r == &call) {
            results.push(call);
        }
    }

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

        let repaired = repair_json_newlines(json_str);
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&repaired) {
            if let Some(arr) = val.as_array() {
                for item in arr {
                    for call in parse_tool_call_values(item) {
                        results.push(call);
                    }
                }
            } else {
                for call in parse_tool_call_values(&val) {
                    results.push(call);
                }
            }
        } else if let Some(call) = extract_lenient_edit_file(json_str) {
            if !results.iter().any(|r| r == &call) {
                results.push(call);
            }
        } else if let Some(call) = extract_lenient_write_file(json_str) {
            if !results.iter().any(|r| r == &call) {
                results.push(call);
            }
        }
    }

    // 2. Fallback: Parse standalone inline JSON objects like {"tool": "..."}
    let mut search_idx = 0;
    while let Some(open_idx) = content[search_idx..].find('{') {
        let actual_open = search_idx + open_idx;
        let slice = &content[actual_open..];
        let close_rel = find_json_object_end(slice);

        if let Some(rel) = close_rel {
            let actual_close = actual_open + rel;
            let candidate = &content[actual_open..=actual_close];
            if candidate.contains("\"tool\"")
                || candidate.contains("\"function\"")
                || candidate.contains("\"name\"")
                || candidate.contains("\"action\"")
                || candidate.contains("\"cmd\"")
                || candidate.contains("\"command\"")
            {
                let repaired = repair_json_newlines(candidate);
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&repaired) {
                    for call in parse_tool_call_values(&val) {
                        if !results.iter().any(|r| r == &call) {
                            results.push(call);
                        }
                    }
                } else if let Some(call) = extract_lenient_edit_file(candidate) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                } else if let Some(call) = extract_lenient_write_file(candidate) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
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
            // Unclosed or trailing tool JSON - attempt lenient extraction from remaining text
            let remainder = &content[actual_open..];
            if remainder.contains("edit_file") {
                if let Some(call) = extract_lenient_edit_file(remainder) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                }
            } else if remainder.contains("write_file") || remainder.contains("create_file") {
                if let Some(call) = extract_lenient_write_file(remainder) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                }
            } else if remainder.contains("run_command") || remainder.contains("\"cmd\"") {
                if let Some(call) = extract_lenient_run_command(remainder) {
                    if !results.iter().any(|r| r == &call) {
                        results.push(call);
                    }
                }
            }
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

    // 5. Fallback: Parse runnable commands from fenced ```bash ... ``` or ```sh ... ``` code blocks (only if no explicit tool calls found)
    if results.is_empty() {
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
}

    results
}

fn find_json_object_end(text: &str) -> Option<usize> {
    let mut depth = 0;
    let mut in_string = false;
    let mut escaped = false;

    for (i, c) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && in_string {
            escaped = true;
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            continue;
        }
        if !in_string {
            if c == '{' {
                depth += 1;
            } else if c == '}' {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
        }
    }
    None
}

fn repair_json_newlines(json: &str) -> String {
    let mut out = String::with_capacity(json.len() + 64);
    let mut in_string = false;
    let mut escaped = false;

    for c in json.chars() {
        if escaped {
            out.push(c);
            escaped = false;
            continue;
        }
        if c == '\\' && in_string {
            escaped = true;
            out.push(c);
            continue;
        }
        if c == '"' {
            in_string = !in_string;
            out.push(c);
            continue;
        }
        if in_string {
            match c {
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                other => out.push(other),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn unescape_json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            if let Some(next) = chars.next() {
                match next {
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    '\\' => out.push('\\'),
                    '"' => out.push('"'),
                    other => {
                        out.push('\\');
                        out.push(other);
                    }
                }
            } else {
                out.push('\\');
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn extract_lenient_field_raw(text: &str, field: &str) -> Option<String> {
    let pattern = format!("\"{}\"", field);
    let key_pos = text.find(&pattern)?;
    let after = &text[key_pos + pattern.len()..];
    let colon_pos = after.find(':')?;
    let val_part = after[colon_pos + 1..].trim_start();
    if let Some(stripped) = val_part.strip_prefix('"') {
        let end_q = stripped.find('"')?;
        Some(stripped[..end_q].to_string())
    } else {
        let end_delim = val_part.find(',').or_else(|| val_part.find('}'))?;
        Some(val_part[..end_delim].trim().trim_matches('"').to_string())
    }
}

fn extract_block_between(text: &str, key: &str, next_keys: &[&str]) -> Option<String> {
    let key_pattern = format!("\"{}\"", key);
    let start_idx = text.find(&key_pattern)?;
    let after_key = &text[start_idx + key_pattern.len()..];
    let colon_idx = after_key.find(':')?;
    let val_start = after_key[colon_idx + 1..].trim_start();
    let without_opening_quote = if let Some(s) = val_start.strip_prefix('"') {
        s
    } else {
        val_start
    };

    let mut end_idx = None;
    for next_k in next_keys {
        let next_pattern = format!("\"{}\"", next_k);
        if let Some(pos) = without_opening_quote.find(&next_pattern) {
            end_idx = Some(match end_idx {
                Some(curr) => std::cmp::min(curr, pos),
                None => pos,
            });
        }
    }

    let raw_val = if let Some(end) = end_idx {
        &without_opening_quote[..end]
    } else {
        without_opening_quote
    };

    let trimmed = raw_val.trim();
    let without_trailing_comma = trimmed.trim_end_matches(',').trim();
    let without_trailing_quote = without_trailing_comma
        .strip_suffix('"')
        .unwrap_or(without_trailing_comma)
        .trim_end_matches('}')
        .trim();

    Some(unescape_json_string(without_trailing_quote))
}

fn extract_lenient_edit_file(text: &str) -> Option<AgentToolCall> {
    if !text.contains("edit_file") {
        return None;
    }
    let path = extract_lenient_field_raw(text, "path")
        .or_else(|| extract_lenient_field_raw(text, "file_path"))?;
    let search = extract_block_between(text, "search", &["replace", "new_str"])
        .or_else(|| extract_block_between(text, "old_str", &["replace", "new_str"]))?;
    let replace = extract_block_between(text, "replace", &["}", "tool"])
        .or_else(|| extract_block_between(text, "new_str", &["}", "tool"]))?;

    Some(AgentToolCall::EditFile {
        path: path.trim().trim_matches('"').to_string(),
        search,
        replace,
    })
}

fn extract_lenient_write_file(text: &str) -> Option<AgentToolCall> {
    if !text.contains("write_file") && !text.contains("create_file") && !text.contains("new_file") && !text.contains("overwrite_file") {
        return None;
    }
    let path = extract_lenient_field_raw(text, "path")
        .or_else(|| extract_lenient_field_raw(text, "file_path"))
        .or_else(|| extract_lenient_field_raw(text, "file"))?;
    let content = extract_block_between(text, "content", &["}", "tool"])
        .or_else(|| extract_block_between(text, "code", &["}", "tool"]))
        .or_else(|| extract_block_between(text, "text", &["}", "tool"]))
        .unwrap_or_default();

    Some(AgentToolCall::WriteFile {
        path: path.trim().trim_matches('"').to_string(),
        content,
    })
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
        match call {
            AgentToolCall::EditFile { path, search, replace } => {
                let p = ground_workspace_path(workspace_root, &path);
                let block = SearchReplaceBlock { search, replace };
                if let Some(existing) = results.iter_mut().find(|m: &&mut TargetFileModification| m.file_path == p) {
                    if let FileEditAction::SurgicalPatch(ref mut blocks) = existing.action {
                        blocks.push(block);
                    }
                } else {
                    results.push(TargetFileModification::new_patch(p, vec![block]));
                }
            }
            AgentToolCall::WriteFile { path, content } => {
                let p = ground_workspace_path(workspace_root, &path);
                if let Some(existing) = results.iter_mut().find(|m: &&mut TargetFileModification| m.file_path == p) {
                    existing.action = FileEditAction::CreateOrOverwrite(content);
                } else {
                    results.push(TargetFileModification::new_overwrite(p, content));
                }
            }
            _ => {}
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
        } else if let Some(rest) = trimmed.strip_prefix("**File:**") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("Create file:").or_else(|| trimmed.strip_prefix("create file:")) {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("Create ").or_else(|| trimmed.strip_prefix("create ")) {
            let candidate = rest.trim().trim_end_matches(':').trim().trim_matches('`').trim_matches('*').trim();
            if is_likely_filepath(candidate) {
                Some(candidate)
            } else {
                None
            }
        } else if let Some(candidate) = trimmed.strip_suffix(':').map(|s| s.trim().trim_matches('`').trim_matches('*').trim()) {
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
                if current_target.is_none() {
                    let fence_info = trimmed.trim_start_matches('`').trim();
                    if let Some(path_str) = extract_filepath_from_fence_info(fence_info) {
                        let path = ground_workspace_path(workspace_root, &path_str);
                        current_target = Some(path);
                    }
                }
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

pub fn extract_filepath_from_fence_info(info: &str) -> Option<String> {
    if info.is_empty() {
        return None;
    }
    for token in info.split_whitespace() {
        let t = token.trim();
        if let Some(val) = t.strip_prefix("file=").or_else(|| t.strip_prefix("path=")).or_else(|| t.strip_prefix("filename=")).or_else(|| t.strip_prefix("title=")) {
            let clean = val.trim_matches('"').trim_matches('\'').trim();
            if is_likely_filepath(clean) {
                return Some(clean.to_string());
            }
        }
        if let Some(colon_pos) = t.find(':') {
            let after_colon = t[colon_pos + 1..].trim().trim_matches('"').trim_matches('\'');
            if is_likely_filepath(after_colon) {
                return Some(after_colon.to_string());
            }
        }
        let clean_t = t.trim_matches('"').trim_matches('\'');
        if is_likely_filepath(clean_t) {
            return Some(clean_t.to_string());
        }
    }
    None
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_edit_file_with_jsx_and_curly_braces() {
        let input = r#"
I will update the calculator App.tsx now:
{"tool": "edit_file", "path": "calculator/src/App.tsx", "search": "function App()\n{ const [ count, setCount ] = useState(0)\n\nreturn ( < > < section id=\"center \"> < div className=\"hero \"> < img src={heroImg} className=\"base \" width=\"170 \" height=\"179 \" alt=\" \" /> < img src={reactLogo} className=\"framework \" alt=\"React logo \" /> < img src={viteLogo} className=\"vite \" alt=\"Vite logo \" /> Get started Edit src/App.tsx and save to test HMR < button type= \"button \" className= \"counter \" onClick={() => setCount((count) => count + 1)}> Count is {count}", "replace": "function App() {\n  const [ input, setInput ] = useState('0');\n  const handleClick = (val: string) => { setInput(prev => prev === '0'? val: prev + val); };\n  return <div>Calculator: {input}</div>;\n}"}
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        if let AgentToolCall::EditFile { path, search, replace } = &calls[0] {
            assert_eq!(path, "calculator/src/App.tsx");
            assert!(search.contains("heroImg"));
            assert!(replace.contains("setInput"));
        } else {
            panic!("Expected EditFile tool call");
        }
    }

    #[test]
    fn test_parse_edit_file_with_raw_newlines() {
        let input = r#"
{"tool": "edit_file", "path": "src/App.tsx", "search": "function App() {
  return <div>Old</div>;
}", "replace": "function App() {
  return <div>New</div>;
}"}
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        if let AgentToolCall::EditFile { path, search, replace } = &calls[0] {
            assert_eq!(path, "src/App.tsx");
            assert!(search.contains("Old"));
            assert!(replace.contains("New"));
        } else {
            panic!("Expected EditFile tool call");
        }
    }

    #[test]
    fn test_parse_write_file_tool_call() {
        let input = r#"
I will update src/App.css to implement the calculator styling.
```json
{
  "tool": "write_file",
  "path": "calculator-app/src/App.css",
  "content": ".calculator { width: 250px; margin: 0 auto; }"
}
```
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        if let AgentToolCall::WriteFile { path, content } = &calls[0] {
            assert_eq!(path, "calculator-app/src/App.css");
            assert!(content.contains(".calculator"));
        } else {
            panic!("Expected WriteFile tool call");
        }
    }

    #[test]
    fn test_parse_edit_file_with_edits_array() {
        let input = r#"
```json
{
  "tool": "edit_file",
  "path": "calculator-app/src/App.tsx",
  "edits": [
    {
      "search": "import { useState } from 'react'",
      "replace": "import { useState, useReducer } from 'react'"
    }
  ]
}
```
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        if let AgentToolCall::EditFile { path, search, replace } = &calls[0] {
            assert_eq!(path, "calculator-app/src/App.tsx");
            assert_eq!(search, "import { useState } from 'react'");
            assert_eq!(replace, "import { useState, useReducer } from 'react'");
        } else {
            panic!("Expected EditFile tool call from edits array");
        }
    }

    #[test]
    fn test_extract_json_modifications_with_write_and_patch() {
        let tmp = tempfile::tempdir().unwrap();
        let input = r#"
```json
{
  "tool": "write_file",
  "path": "styles.css",
  "content": "body { margin: 0; }"
}
```
```json
{
  "tool": "edit_file",
  "path": "app.js",
  "search": "const a = 1;",
  "replace": "const a = 2;"
}
```
        "#;
        let mods = extract_json_tool_modifications(input, tmp.path());
        assert_eq!(mods.len(), 2);
        let write_mod = mods.iter().find(|m| m.file_path == tmp.path().join("styles.css")).unwrap();
        match &write_mod.action {
            FileEditAction::CreateOrOverwrite(c) => assert_eq!(c, "body { margin: 0; }"),
            _ => panic!("Expected CreateOrOverwrite action"),
        }
        let edit_mod = mods.iter().find(|m| m.file_path == tmp.path().join("app.js")).unwrap();
        match &edit_mod.action {
            FileEditAction::SurgicalPatch(blocks) => {
                assert_eq!(blocks.len(), 1);
                assert_eq!(blocks[0].search, "const a = 1;");
                assert_eq!(blocks[0].replace, "const a = 2;");
            }
            _ => panic!("Expected SurgicalPatch action"),
        }
    }

    #[test]
    fn test_parse_qwen_xml_tool_call_list_files() {
        let input = "< tool_call> < function=list_files> < parameter=path>. </ tool_call>";
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::ListDirectory { path } => {
                assert_eq!(path.as_deref(), Some("."));
            }
            _ => panic!("Expected ListDirectory call, got {:?}", calls[0]),
        }
    }

    #[test]
    fn test_parse_qwen_xml_tool_call_multiline_run_command() {
        let input = r#"
<tool_call>
<function=run_command>
<parameter=cmd>
bun create vite calculator --template react-ts
</parameter>
</tool_call>
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::RunCommand { cmd } => {
                assert_eq!(cmd, "bun create vite calculator --template react-ts");
            }
            _ => panic!("Expected RunCommand call, got {:?}", calls[0]),
        }
    }

    #[test]
    fn test_parse_qwen_xml_tool_call_write_file_with_jsx() {
        let input = r#"
<tool_call>
<function=write_file>
<parameter=path>src/App.tsx</parameter>
<parameter=content>
import React from 'react';
export default function App() {
  return <div className="calc"><button>1</button></div>;
}
</parameter>
</tool_call>
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::WriteFile { path, content } => {
                assert_eq!(path, "src/App.tsx");
                assert!(content.contains(r#"<div className="calc"><button>1</button></div>"#));
            }
            _ => panic!("Expected WriteFile call, got {:?}", calls[0]),
        }
    }

    #[test]
    fn test_parse_qwen_xml_tool_call_edit_file() {
        let input = r#"
<tool_call>
<function=edit_file>
<parameter=path>src/index.css</parameter>
<parameter=search>body { margin: 0; }</parameter>
<parameter=replace>body { margin: 0; background: #111; }</parameter>
</tool_call>
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::EditFile { path, search, replace } => {
                assert_eq!(path, "src/index.css");
                assert_eq!(search, "body { margin: 0; }");
                assert_eq!(replace, "body { margin: 0; background: #111; }");
            }
            _ => panic!("Expected EditFile call, got {:?}", calls[0]),
        }
    }

    #[test]
    fn test_parse_direct_xml_write_file() {
        let input = r#"
<write_file path="src/main.tsx">
import ReactDOM from 'react-dom/client';
ReactDOM.createRoot(document.getElementById('root')!).render(<App />);
</write_file>
        "#;
        let calls = parse_agent_tool_calls(input);
        assert_eq!(calls.len(), 1);
        match &calls[0] {
            AgentToolCall::WriteFile { path, content } => {
                assert_eq!(path, "src/main.tsx");
                assert!(content.contains("ReactDOM"));
            }
            _ => panic!("Expected WriteFile call from direct XML, got {:?}", calls[0]),
        }
    }

    #[test]
    fn test_extract_all_file_modifications_from_fence_header() {
        let tmp = tempfile::tempdir().unwrap();
        let input = r#"
Here is the calculator component:
```tsx src/Calculator.tsx
export default function Calculator() {
  return <div>0</div>;
}
```
        "#;
        let mods = extract_all_file_modifications(input, tmp.path(), None);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].file_path, tmp.path().join("src/Calculator.tsx"));
        match &mods[0].action {
            FileEditAction::CreateOrOverwrite(code) => {
                assert!(code.contains("export default function Calculator"));
            }
            _ => panic!("Expected CreateOrOverwrite action"),
        }
    }
}

