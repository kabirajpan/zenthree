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
                let cwd = params
                    .get("cwd")
                    .or_else(|| params.get("working_dir"))
                    .or_else(|| params.get("dir"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.trim().to_string());
                Some(AgentToolCall::RunCommand {
                    cmd: cmd.trim().to_string(),
                    cwd,
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
        let cwd = val
            .get("cwd")
            .or_else(|| val.get("working_dir"))
            .or_else(|| val.get("dir"))
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string());
        return Some(AgentToolCall::RunCommand {
            cmd: cmd.trim().to_string(),
            cwd,
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
                    let call = AgentToolCall::RunCommand { cmd: cmd.to_string(), cwd: None };
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
                let call = AgentToolCall::RunCommand { cmd: cmd.to_string(), cwd: None };
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
                let call = AgentToolCall::RunCommand { cmd: full_cmd, cwd: None };
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
        Some(AgentToolCall::RunCommand { cmd: cleaned.to_string(), cwd: None })
    } else {
        None
    }
}

pub fn is_ignored_path_segment(name: &str) -> bool {
    name == ".git"
        || name == ".zenthree"
        || name == ".svn"
        || name == ".hg"
        || name == ".cache"
        || name == ".idea"
        || name == ".DS_Store"
        || name == "Thumbs.db"
        || name == "target"
        || name == "node_modules"
        || name == "dist"
        || name == "build"
        || name == "vendor"
        || name == ".next"
        || name == ".nuxt"
        || name == ".turbo"
}

/// Cross-verifies that the target file actually contains the `search_snippet`.
/// If it does not, scans the workspace for candidate files with the same filename (or other source files)
/// that actually contain the snippet, automatically relocating to the correct file and
/// preventing edits to the wrong file!
pub fn verify_or_relocate_target_with_search(
    workspace_root: &Path,
    target: PathBuf,
    search_snippet: &str,
) -> PathBuf {
    let clean_search = search_snippet.trim();
    if clean_search.is_empty() {
        return target;
    }

    // 1. Direct check: Does target file already contain the search snippet?
    if let Ok(content) = std::fs::read_to_string(&target) {
        if content.contains(clean_search)
            || content.replace("\r\n", "\n").contains(&clean_search.replace("\r\n", "\n"))
        {
            return target;
        }
    }

    // 2. Target does not contain search snippet.
    // Search the workspace for any candidate file with the same filename that DOES contain it.
    let fname = target.file_name().and_then(|f| f.to_str()).unwrap_or("");
    if !fname.is_empty() {
        let mut stack = vec![(workspace_root.to_path_buf(), 0usize)];
        while let Some((dir, depth)) = stack.pop() {
            if depth > 7 {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    if is_ignored_path_segment(&name) {
                        continue;
                    }
                    if path.is_dir() {
                        stack.push((path, depth + 1));
                    } else if path.is_file() && name == fname && path != target {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains(clean_search)
                                || content.replace("\r\n", "\n").contains(&clean_search.replace("\r\n", "\n"))
                            {
                                return path;
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Fallback scan: check other likely source files if search snippet is distinctive (>= 15 chars)
    if clean_search.len() >= 15 {
        let mut stack = vec![(workspace_root.to_path_buf(), 0usize)];
        while let Some((dir, depth)) = stack.pop() {
            if depth > 7 {
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    let name = entry.file_name().to_string_lossy().to_string();
                    if is_ignored_path_segment(&name) {
                        continue;
                    }
                    if path.is_dir() {
                        stack.push((path, depth + 1));
                    } else if path.is_file() && path != target && is_likely_filepath(&name) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            if content.contains(clean_search)
                                || content.replace("\r\n", "\n").contains(&clean_search.replace("\r\n", "\n"))
                            {
                                return path;
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to original target (matcher will safely error rather than corrupting)
    target
}

pub fn extract_json_tool_modifications(
    content: &str,
    workspace_root: &Path,
) -> Vec<TargetFileModification> {
    extract_json_tool_modifications_with_context(content, workspace_root, None)
}

pub fn extract_json_tool_modifications_with_context(
    content: &str,
    workspace_root: &Path,
    active_path: Option<&Path>,
) -> Vec<TargetFileModification> {
    let mut results = Vec::new();
    let calls = parse_agent_tool_calls(content);

    for call in calls {
        if let AgentToolCall::EditFile { path, search, replace } = call {
            let p = ground_workspace_path_with_context(workspace_root, &path, active_path);
            let verified_p = verify_or_relocate_target_with_search(workspace_root, p, &search);
            let block = SearchReplaceBlock { search, replace };
            if let Some(existing) = results.iter_mut().find(|m: &&mut TargetFileModification| m.file_path == verified_p) {
                if let FileEditAction::SurgicalPatch(ref mut blocks) = existing.action {
                    blocks.push(block);
                }
            } else {
                results.push(TargetFileModification::new_patch(verified_p, vec![block]));
            }
        }
    }

    results
}

pub fn ground_workspace_path(workspace_root: &Path, cleaned: &str) -> PathBuf {
    ground_workspace_path_with_context(workspace_root, cleaned, None)
}

/// Intelligently grounds a relative or bare filepath against the workspace:
/// 1. Suffix matching for partial/nested relative paths (e.g. `src/tools/read.rs` -> `crates/ai/src/tools/read.rs`).
/// 2. Workspace-wide exact filename resolution for existing files (prevents modifying/creating ghost files on root).
/// 3. Anti-root-dumping placement for new files (prefers active file sibling directory or standard `src/`/`app/`/`lib/` over root).
pub fn ground_workspace_path_with_context(
    workspace_root: &Path,
    cleaned: &str,
    active_path: Option<&Path>,
) -> PathBuf {
    let clean_str = cleaned.trim().trim_matches('`').trim_matches('*').trim();
    if clean_str.is_empty() {
        return workspace_root.to_path_buf();
    }
    if Path::new(clean_str).is_absolute() {
        return PathBuf::from(clean_str);
    }

    let direct = workspace_root.join(clean_str);
    if direct.exists() {
        return direct;
    }

    let p = Path::new(clean_str);
    let has_directory_components = p.parent().map_or(false, |par| !par.as_os_str().is_empty());

    // 1. If explicit multi-segment directory components were provided and its parent exists in workspace
    if has_directory_components {
        if let Some(parent) = direct.parent() {
            if parent.exists() && parent.is_dir() {
                return direct;
            }
        }
    }

    // Helper: list candidates recursively in workspace (excluding build artifacts & noise)
    let mut matching_files = Vec::new();
    let mut suffix_matches = Vec::new();
    let fname = p.file_name().and_then(|f| f.to_str()).unwrap_or(clean_str);

    let mut stack = vec![(workspace_root.to_path_buf(), 0usize)];
    while let Some((dir, depth)) = stack.pop() {
        if depth > 7 || matching_files.len() > 30 {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = entry.file_name().to_string_lossy().to_string();
                if is_ignored_path_segment(&name) {
                    continue;
                }
                if path.is_dir() {
                    stack.push((path, depth + 1));
                } else if path.is_file() {
                    // Check exact filename match
                    if name == fname {
                        matching_files.push(path.clone());
                    }
                    // Check suffix match for relative paths like "src/tools/read.rs" matching "crates/ai/src/tools/read.rs"
                    if has_directory_components {
                        if let Ok(rel) = path.strip_prefix(workspace_root) {
                            let rel_str = rel.to_string_lossy();
                            if rel_str.ends_with(clean_str) {
                                suffix_matches.push(path);
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Suffix match takes high priority for qualified subpaths
    if let Some(suf) = suffix_matches.first() {
        return suf.clone();
    }

    // 3. Exact filename match for existing files
    if !matching_files.is_empty() {
        if let Some(active) = active_path {
            if let Some(active_dir) = active.parent() {
                // Check if any matching file shares the active file's directory
                if let Some(same_dir) = matching_files.iter().find(|m| m.parent() == Some(active_dir)) {
                    return same_dir.clone();
                }
            }
        }
        // Return first existing matching file in workspace
        return matching_files[0].clone();
    }

    // 4. If explicit directory components were specified and no existing file matched:
    // Keep the requested structure relative to workspace_root (e.g. ".github/workflows/ci.yml" or "src/components/NewButton.tsx")
    if has_directory_components {
        return direct;
    }

    // 5. Bare filename for a NEW file (doesn't exist anywhere in the project):
    let fname_lower = fname.to_lowercase();
    let is_root_file = fname_lower == "readme.md"
        || fname_lower == "license"
        || fname_lower == "licence"
        || fname_lower == "makefile"
        || fname_lower == "justfile"
        || fname_lower == "dockerfile"
        || fname_lower.starts_with("dockerfile.")
        || fname_lower == "docker-compose.yml"
        || fname_lower == "docker-compose.yaml"
        || fname_lower == "procfile"
        || fname_lower == ".dockerignore"
        || fname_lower == ".gitignore"
        || fname_lower == ".gitattributes"
        || fname_lower == ".editorconfig"
        || fname_lower.starts_with(".env")
        || fname_lower.starts_with(".eslint")
        || fname_lower.starts_with(".prettier")
        || fname_lower == "cargo.toml"
        || fname_lower == "package.json"
        || fname_lower == "tsconfig.json"
        || fname_lower == "pyproject.toml"
        || fname_lower == "go.mod"
        || fname_lower == "cmakelists.txt"
        || fname_lower == "composer.json"
        || fname_lower == "gemfile";

    if is_root_file {
        return direct;
    }

    // Sibling of active file: if user has an active file open in workspace, place new sibling file in same folder
    if let Some(active) = active_path {
        if let Some(active_dir) = active.parent() {
            if active_dir.starts_with(workspace_root) && active_dir.exists() {
                return active_dir.join(fname);
            }
        }
    }

    // Place in standard source directory if present (e.g. src/, app/, lib/, pkg/)
    for standard_src in ["src", "app", "lib", "pkg"] {
        let candidate_dir = workspace_root.join(standard_src);
        if candidate_dir.is_dir() {
            return candidate_dir.join(fname);
        }
    }

    direct
}

pub fn extract_all_file_modifications(
    content: &str,
    workspace_root: &Path,
    active_path: Option<&Path>,
) -> Vec<TargetFileModification> {
    let mut results = extract_json_tool_modifications_with_context(content, workspace_root, active_path);
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
                let first_search = blocks.first().map(|b| b.search.as_str()).unwrap_or("");
                let verified_tf = verify_or_relocate_target_with_search(workspace_root, tf, first_search);
                if let Some(existing) = res.iter_mut().find(|m| m.file_path == verified_tf) {
                    if let FileEditAction::SurgicalPatch(ref mut b) = existing.action {
                        b.extend(blocks);
                    }
                } else {
                    res.push(TargetFileModification::new_patch(verified_tf, blocks));
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
                let path = ground_workspace_path_with_context(workspace_root, cleaned, active_path);
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
                            let first_search = surgical_blocks.first().map(|b| b.search.as_str()).unwrap_or("");
                            let verified_tf = verify_or_relocate_target_with_search(workspace_root, target_file, first_search);
                            if let Some(existing) = results.iter_mut().find(|m| m.file_path == verified_tf) {
                                if let FileEditAction::SurgicalPatch(ref mut b) = existing.action {
                                    b.extend(surgical_blocks);
                                }
                            } else {
                                results.push(TargetFileModification::new_patch(verified_tf, surgical_blocks));
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
    let is_known_config_file = lower == "dockerfile"
        || lower.starts_with("dockerfile.")
        || lower == "makefile"
        || lower == "justfile"
        || lower == "procfile"
        || lower == "gemfile"
        || lower == ".dockerignore"
        || lower == ".gitignore"
        || lower == ".gitattributes"
        || lower == ".editorconfig"
        || lower.starts_with(".env")
        || lower.starts_with(".eslint")
        || lower.starts_with(".prettier");

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
        || lower.ends_with(".yml")
        || lower.ends_with(".xml")
        || lower.ends_with(".sql")
        || lower.ends_with(".conf")
        || lower.ends_with(".cfg")
        || lower.ends_with(".ini");

    (has_ext || is_known_config_file) && !s.contains(' ') && s.len() >= 2
}
