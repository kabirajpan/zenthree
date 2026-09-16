use std::path::{Path, PathBuf};
use crate::state::{EditedFileSummary, ExploredItem};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Pending,
    ReadingChecklist,
    InProgress,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTask {
    pub id: String,
    pub title: String,
    pub status: TaskStatus,
    pub target_file: Option<String>,
    pub diff_summary: Option<EditedFileSummary>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentMission {
    pub id: String,
    pub goal: String,
    pub tasks: Vec<AgentTask>,
    pub active_task_index: Option<usize>,
    pub is_active: bool,
    pub is_paused: bool,
    pub status_message: String,
    pub self_healing_attempts: usize,
}

impl AgentMission {
    /// Returns index of next task with Pending status
    pub fn next_runnable_task_index(&self) -> Option<usize> {
        self.tasks.iter().position(|t| t.status == TaskStatus::Pending)
    }

    /// Marks the specified task failed with a diagnostic message and advances to next runnable task
    pub fn mark_task_failed_and_continue(&mut self, idx: usize, error_msg: String) {
        if let Some(t) = self.tasks.get_mut(idx) {
            t.status = TaskStatus::Failed(error_msg);
        }
        if let Some(next_idx) = self.next_runnable_task_index() {
            self.active_task_index = Some(next_idx);
            if let Some(next_task) = self.tasks.get_mut(next_idx) {
                next_task.status = TaskStatus::InProgress;
                self.status_message = format!("Task {} failed, proceeding with: {}", idx + 1, next_task.title);
            }
            self.is_active = true;
        } else {
            self.active_task_index = None;
            self.is_active = false;
            self.status_message = "All tasks processed (completed with some failures)".to_string();
        }
    }

    /// Advances to the next pending task, marking current task completed
    pub fn complete_task_and_advance(&mut self, idx: usize) {
        if let Some(t) = self.tasks.get_mut(idx) {
            t.status = TaskStatus::Completed;
        }
        let total_tasks = self.tasks.len();
        if let Some(next_idx) = self.next_runnable_task_index() {
            self.active_task_index = Some(next_idx);
            if let Some(next_task) = self.tasks.get_mut(next_idx) {
                next_task.status = TaskStatus::InProgress;
                self.status_message = format!("Executing Task {} of {}: {}", next_idx + 1, total_tasks, next_task.title);
            }
            self.is_active = true;
        } else {
            self.active_task_index = None;
            self.is_active = false;
            self.status_message = "✓ All tasks completed successfully".to_string();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProviderError {
    Transient(String),
    Fatal(String),
}

impl std::fmt::Display for ProviderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProviderError::Transient(s) => write!(f, "Transient error: {}", s),
            ProviderError::Fatal(s) => write!(f, "Fatal error: {}", s),
        }
    }
}

pub fn classify_provider_error(err_str: &str) -> ProviderError {
    let lower = err_str.to_lowercase();
    if lower.contains("429")
        || lower.contains("rate limit")
        || lower.contains("too many requests")
        || lower.contains("503")
        || lower.contains("500")
        || lower.contains("timeout")
        || lower.contains("resource exhausted")
        || lower.contains("temporarily unavailable")
    {
        ProviderError::Transient(err_str.to_string())
    } else {
        ProviderError::Fatal(err_str.to_string())
    }
}

#[derive(Debug, Clone)]
pub struct ProviderChain {
    pub gemini_key: Option<String>,
    pub groq_key: Option<String>,
    pub preferred_model: Option<String>,
}

impl ProviderChain {
    pub fn new(
        gemini_key: Option<String>,
        groq_key: Option<String>,
        preferred_model: Option<String>,
    ) -> Self {
        Self {
            gemini_key,
            groq_key,
            preferred_model,
        }
    }

    /// Completes the prompt with retries on transient errors and automatic fallback between Gemini and Groq
    pub fn complete(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        history: &[crate::groq::ChatMessage],
    ) -> Result<String, String> {
        let is_gemini_target = self
            .preferred_model
            .as_deref()
            .map(|m| m.contains("gemini") || m.contains("gemma"))
            .unwrap_or(false);
        let is_groq_target = self
            .preferred_model
            .as_deref()
            .map(|m| m.contains("gpt") || m.contains("qwen") || m.contains("llama"))
            .unwrap_or(false);

        let mut try_gemini_first = (is_gemini_target || !is_groq_target) && self.gemini_key.is_some();
        if !try_gemini_first && self.groq_key.is_none() && self.gemini_key.is_some() {
            try_gemini_first = true;
        }

        let mut errors = Vec::new();

        if try_gemini_first {
            if let Some(ref gkey) = self.gemini_key {
                let pref = self.preferred_model.as_deref();
                for attempt in 0..2 {
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                    match crate::gemini::call_gemini_api_with_model(gkey, system_prompt, user_prompt, history, pref) {
                        Ok(res) => return Ok(res),
                        Err(e) => {
                            let classified = classify_provider_error(&e);
                            match classified {
                                ProviderError::Transient(_) if attempt == 0 => {
                                    log::warn!("Gemini transient error on attempt 1, retrying: {}", e);
                                    continue;
                                }
                                _ => {
                                    errors.push(format!("Gemini failed: {}", e));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Automatic fallback to Groq
            if let Some(ref qkey) = self.groq_key {
                log::info!("Falling back from Gemini to Groq provider...");
                let mut messages = Vec::new();
                messages.push(crate::groq::ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                });
                for h in history {
                    messages.push(h.clone());
                }
                messages.push(crate::groq::ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                });

                let pref = if is_groq_target { self.preferred_model.as_deref() } else { None };
                match crate::groq::call_groq_api_messages_with_model(qkey, &messages, pref) {
                    Ok(res) => return Ok(res),
                    Err(e) => errors.push(format!("Groq fallback failed: {}", e)),
                }
            }
        } else {
            // Groq first
            if let Some(ref qkey) = self.groq_key {
                let mut messages = Vec::new();
                messages.push(crate::groq::ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                });
                for h in history {
                    messages.push(h.clone());
                }
                messages.push(crate::groq::ChatMessage {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                });

                let pref = self.preferred_model.as_deref();
                for attempt in 0..2 {
                    if attempt > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(500));
                    }
                    match crate::groq::call_groq_api_messages_with_model(qkey, &messages, pref) {
                        Ok(res) => return Ok(res),
                        Err(e) => {
                            let classified = classify_provider_error(&e);
                            match classified {
                                ProviderError::Transient(_) if attempt == 0 => {
                                    log::warn!("Groq transient error on attempt 1, retrying: {}", e);
                                    continue;
                                }
                                _ => {
                                    errors.push(format!("Groq failed: {}", e));
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            // Automatic fallback to Gemini
            if let Some(ref gkey) = self.gemini_key {
                log::info!("Falling back from Groq to Gemini provider...");
                let pref = if is_gemini_target { self.preferred_model.as_deref() } else { None };
                match crate::gemini::call_gemini_api_with_model(gkey, system_prompt, user_prompt, history, pref) {
                    Ok(res) => return Ok(res),
                    Err(e) => errors.push(format!("Gemini fallback failed: {}", e)),
                }
            }
        }

        if errors.is_empty() {
            Err("No AI provider configured (GEMINI_API_KEY or GROQ_API_KEY required)".to_string())
        } else {
            Err(errors.join(" | "))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AgentToolCall {
    ReadFile {
        path: String,
        #[serde(default)]
        start_line: Option<usize>,
        #[serde(default)]
        end_line: Option<usize>,
    },
    SearchCode {
        query: String,
    },
    ListDirectory {
        #[serde(default)]
        path: Option<String>,
    },
    RunCommand {
        cmd: String,
    },
    EditFile {
        path: String,
        search: String,
        replace: String,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentToolResult {
    pub tool_name: String,
    pub ok: bool,
    pub output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SearchReplaceBlock {
    pub search: String,
    pub replace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum FileEditAction {
    CreateOrOverwrite(String),
    SurgicalPatch(Vec<SearchReplaceBlock>),
}

#[derive(Debug, Clone)]
pub struct TargetFileModification {
    pub file_path: PathBuf,
    pub action: FileEditAction,
}

impl TargetFileModification {
    pub fn new_overwrite(file_path: PathBuf, code: String) -> Self {
        Self {
            file_path,
            action: FileEditAction::CreateOrOverwrite(code),
        }
    }

    pub fn new_patch(file_path: PathBuf, patches: Vec<SearchReplaceBlock>) -> Self {
        Self {
            file_path,
            action: FileEditAction::SurgicalPatch(patches),
        }
    }

    pub fn apply_to(&self, original: &str) -> Result<String, String> {
        match &self.action {
            FileEditAction::CreateOrOverwrite(code) => Ok(code.clone()),
            FileEditAction::SurgicalPatch(blocks) => apply_search_replace(original, blocks),
        }
    }
}

/// Creates a structured multi-step Agent Mission based on user intent and workspace context
pub fn create_mission_for_prompt(prompt: &str, explored_items: &[ExploredItem]) -> AgentMission {
    let trimmed = prompt.trim();
    let lower = trimmed.to_lowercase();
    let is_refactor = lower.contains("refactor") || lower.contains("optimize");
    let is_explain = lower.contains("explain");

    let mut tasks = Vec::new();

    // Task 1: Context & Reading (Completed via tree-sitter & project indexing)
    let read_title = if !explored_items.is_empty() {
        let file_list: Vec<&str> = explored_items.iter().take(2).map(|i| i.file_name.as_str()).collect();
        format!("Analyze workspace context ({})", file_list.join(", "))
    } else {
        "Analyze workspace context & symbols".to_string()
    };
    tasks.push(AgentTask {
        id: "task_1_context".to_string(),
        title: read_title,
        status: TaskStatus::Completed,
        target_file: explored_items.first().map(|i| i.file_name.clone()),
        diff_summary: None,
    });

    // Task 2: Code Execution / Generation (In Progress)
    let exec_title = if lower.contains("create") || lower.contains("make") || lower.contains("add") {
        let desc = if trimmed.len() > 30 { &trimmed[..30] } else { trimmed };
        format!("Generate file & directory updates for `{}`", desc)
    } else if is_refactor {
        "Apply architectural refactoring".to_string()
    } else if is_explain {
        "Formulate comprehensive code analysis".to_string()
    } else {
        let desc = if trimmed.len() > 30 { &trimmed[..30] } else { trimmed };
        format!("Implement changes for `{}`", desc)
    };
    tasks.push(AgentTask {
        id: "task_2_exec".to_string(),
        title: exec_title,
        status: TaskStatus::InProgress,
        target_file: None,
        diff_summary: None,
    });

    // Task 3: Verification & self-check (Pending)
    tasks.push(AgentTask {
        id: "task_3_verify".to_string(),
        title: "Verify syntax and code consistency".to_string(),
        status: TaskStatus::Pending,
        target_file: None,
        diff_summary: None,
    });

    AgentMission {
        id: format!("mission_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)),
        goal: trimmed.to_string(),
        tasks,
        active_task_index: Some(1),
        is_active: true,
        is_paused: false,
        status_message: "Executing Task 2 of 3...".to_string(),
        self_healing_attempts: 0,
    }
}

/// System prompt instructing the autonomous agent to plan and deduce file tree architecture
pub fn build_autonomous_agent_prompt(_deliberation_confidence: f32) -> String {
    format!(
        "You are Zenthree AI, an autonomous software engineering agent integrated directly into the Zenthree code editor with full workspace access.\n\
        The Project Workspace File Tree is provided below in the context.\n\n\
        CRITICAL AUTONOMOUS RULES:\n\
        1. The user gives high-level feature requests without specifying file locations or paths (e.g. 'create contact page and add navigation on navbar').\n\
        2. Inspect the Project Workspace File Tree to deduce the framework (Qwik City, React, Next.js, Vite, Vue, Svelte, Rust, Python, etc.) and project conventions.\n\
        3. Formulate an Architectural Plan stating target files to create or modify.\n\
        4. SURGICAL CODE EDITING & DIRECT GENERATION REQUIREMENT:\n\
           - When you know what changes to make, output code modifications IMMEDIATELY:\n\
             * For NEW files: `### File: <relative_path>` followed by the full file in a fenced code block.\n\
             * For EXISTING files: Use surgical SEARCH/REPLACE diff blocks so modifications are precise:\n\
               `### File: <relative_path>`\n\
               ```\n\
               <<<<<<< SEARCH\n\
               // exact snippet from existing file to replace\n\
               =======\n\
               // new replacement snippet\n\
               >>>>>>> REPLACE\n\
               ```\n\
        5. WORKSPACE INSPECTION TOOLS (When you need to discover symbols, read files, or check codebase):\n\
           You may emit JSON tool calls to inspect the workspace before writing code:\n\
           - `{{\"tool\": \"search_code\", \"query\": \"<pattern>\"}}`: Grep code & symbols across workspace files.\n\
           - `{{\"tool\": \"read_file\", \"path\": \"<relative_path>\", \"start_line\": 1, \"end_line\": 50}}`: Read file with line numbers.\n\
           - `{{\"tool\": \"list_directory\", \"path\": \"<optional_path>\"}}`: List files and subdirectories.\n\
           - `{{\"tool\": \"run_command\", \"cmd\": \"<command>\"}}`: Run verification or test command.\n\
        6. Never output placeholders, ellipses (`// ...`), or unfinished functions inside REPLACE blocks. Provide full, working implementations.\n\
        The editor executes tools or parses `### File: <path>` diffs directly to apply changes and verify with the compiler."
    )
}

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

/// Applies a sequence of SEARCH/REPLACE surgical blocks using a 3-tier matcher:
/// Tier 1: Exact substring match
/// Tier 2: CRLF vs LF line-ending normalization
/// Tier 3: Line-by-line trimmed comparison with automatic indentation adjustment
pub fn apply_search_replace(
    original: &str,
    blocks: &[SearchReplaceBlock],
) -> Result<String, String> {
    if blocks.is_empty() {
        return Ok(original.to_string());
    }

    let mut current_content = original.to_string();

    for (idx, block) in blocks.iter().enumerate() {
        let search = &block.search;
        let replace = &block.replace;

        if search.is_empty() {
            if current_content.trim().is_empty() {
                current_content = replace.clone();
                continue;
            }
            return Err(format!("Block #{}: Search block cannot be empty", idx + 1));
        }

        // Tier 1: Exact Substring Match
        if let Some(pos) = current_content.find(search) {
            let before = &current_content[..pos];
            let after = &current_content[pos + search.len()..];
            current_content = format!("{}{}{}", before, replace, after);
            continue;
        }

        // Tier 2: CRLF / LF line-ending normalization
        let norm_current = current_content.replace("\r\n", "\n");
        let norm_search = search.replace("\r\n", "\n");
        let norm_replace = replace.replace("\r\n", "\n");

        if let Some(pos) = norm_current.find(&norm_search) {
            let before = &norm_current[..pos];
            let after = &norm_current[pos + norm_search.len()..];
            current_content = format!("{}{}{}", before, norm_replace, after);
            continue;
        }

        // Tier 3: Line-trimmed / Indentation-tolerant matching
        let cur_lines: Vec<&str> = norm_current.lines().collect();
        let search_lines: Vec<&str> = norm_search.lines().collect();
        let search_len = search_lines.len();

        if search_len > 0 && cur_lines.len() >= search_len {
            let mut matched_line_idx = None;

            for i in 0..=(cur_lines.len() - search_len) {
                let mut matches = true;
                for j in 0..search_len {
                    if cur_lines[i + j].trim() != search_lines[j].trim() {
                        matches = false;
                        break;
                    }
                }
                if matches {
                    matched_line_idx = Some(i);
                    break;
                }
            }

            if let Some(start_idx) = matched_line_idx {
                let orig_indent = cur_lines[start_idx].len() - cur_lines[start_idx].trim_start().len();
                let search_indent = search_lines
                    .iter()
                    .find(|l| !l.trim().is_empty())
                    .map(|l| l.len() - l.trim_start().len())
                    .unwrap_or(0);
                let delta = orig_indent as isize - search_indent as isize;

                let mut adjusted_replace_lines = Vec::new();
                for r_line in norm_replace.lines() {
                    if r_line.trim().is_empty() {
                        adjusted_replace_lines.push(String::new());
                    } else if delta > 0 {
                        adjusted_replace_lines.push(format!("{}{}", " ".repeat(delta as usize), r_line));
                    } else if delta < 0 {
                        let to_strip = (-delta) as usize;
                        let cur_r_indent = r_line.len() - r_line.trim_start().len();
                        let strip = to_strip.min(cur_r_indent);
                        adjusted_replace_lines.push(r_line[strip..].to_string());
                    } else {
                        adjusted_replace_lines.push(r_line.to_string());
                    }
                }

                let mut new_lines = Vec::new();
                new_lines.extend_from_slice(&cur_lines[..start_idx]);
                for r in &adjusted_replace_lines {
                    new_lines.push(r.as_str());
                }
                new_lines.extend_from_slice(&cur_lines[start_idx + search_len..]);

                let trailing_newline = norm_current.ends_with('\n');
                let mut joined = new_lines.join("\n");
                if trailing_newline && !joined.ends_with('\n') {
                    joined.push('\n');
                }

                current_content = joined;
                continue;
            }
        }

        return Err(format!(
            "Failed to find search pattern (block #{}):\n```\n{}\n```",
            idx + 1,
            search
        ));
    }

    Ok(current_content)
}

fn parse_single_json_tool_val(
    val: &serde_json::Value,
    workspace_root: &Path,
    results: &mut Vec<TargetFileModification>,
) {
    let is_edit = val.get("tool").and_then(|v| v.as_str()) == Some("edit_file")
        || val.get("name").and_then(|v| v.as_str()) == Some("edit_file")
        || val.get("action").and_then(|v| v.as_str()) == Some("edit_file");

    if !is_edit {
        return;
    }

    let params = val.get("parameters").unwrap_or(val);
    let path_str = params
        .get("path")
        .or_else(|| params.get("file_path"))
        .or_else(|| params.get("file"))
        .and_then(|v| v.as_str());

    if let Some(path_str) = path_str {
        let cleaned = path_str.trim().trim_matches('"');
        let full_path = if Path::new(cleaned).is_absolute() {
            PathBuf::from(cleaned)
        } else {
            workspace_root.join(cleaned)
        };

        let mut blocks = Vec::new();

        if let (Some(s), Some(r)) = (
            params.get("search").and_then(|v| v.as_str()),
            params.get("replace").and_then(|v| v.as_str()),
        ) {
            blocks.push(SearchReplaceBlock {
                search: s.to_string(),
                replace: r.to_string(),
            });
        } else if let Some(edits_arr) = params.get("edits").and_then(|v| v.as_array()) {
            for edit in edits_arr {
                if let (Some(s), Some(r)) = (
                    edit.get("search").and_then(|v| v.as_str()),
                    edit.get("replace").and_then(|v| v.as_str()),
                ) {
                    blocks.push(SearchReplaceBlock {
                        search: s.to_string(),
                        replace: r.to_string(),
                    });
                }
            }
        }

        if !blocks.is_empty() {
            if let Some(existing) = results.iter_mut().find(|m| m.file_path == full_path) {
                if let FileEditAction::SurgicalPatch(ref mut existing_blocks) = existing.action {
                    existing_blocks.extend(blocks);
                    return;
                }
            }
            results.push(TargetFileModification::new_patch(full_path, blocks));
        }
    }
}

/// Parses any JSON tool calls requesting edit_file
pub fn extract_json_tool_modifications(
    content: &str,
    workspace_root: &Path,
) -> Vec<TargetFileModification> {
    let mut results = Vec::new();
    if !content.contains("edit_file") {
        return results;
    }

    for block in content.split("```") {
        let trimmed = block.trim();
        let json_str = if let Some(rest) = trimmed.strip_prefix("json") {
            rest.trim()
        } else if trimmed.starts_with('{') {
            trimmed
        } else {
            continue;
        };

        if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
            parse_single_json_tool_val(&val, workspace_root, &mut results);
        }
    }

    results
}

/// Extracts any file paths requested inside pseudo <tool_call> or <function=read_file> blocks
pub fn extract_tool_call_read_paths(content: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let lower = content.to_lowercase();
    if !lower.contains("tool_call") && !lower.contains("function=read") && !lower.contains("read_file") {
        return paths;
    }

    let tokens: Vec<&str> = content
        .split(|c: char| c.is_whitespace() || c == '"' || c == '\'' || c == '[' || c == ']' || c == ',' || c == '<' || c == '>')
        .filter(|s| !s.is_empty())
        .collect();

    for token in tokens {
        let cleaned = token.trim_matches('"').trim_matches('\'').trim_matches('`').trim();
        let is_source_file = cleaned.ends_with(".tsx")
            || cleaned.ends_with(".ts")
            || cleaned.ends_with(".jsx")
            || cleaned.ends_with(".js")
            || cleaned.ends_with(".rs")
            || cleaned.ends_with(".vue")
            || cleaned.ends_with(".svelte")
            || cleaned.ends_with(".css")
            || cleaned.ends_with(".html")
            || cleaned.ends_with(".json")
            || cleaned.ends_with(".toml")
            || cleaned.ends_with(".py")
            || cleaned.ends_with(".go");

        if is_source_file && (cleaned.contains('/') || cleaned.contains('\\') || cleaned.starts_with("src")) {
            if !paths.contains(&cleaned.to_string()) {
                paths.push(cleaned.to_string());
            }
        }
    }

    paths
}

/// Helper to extract structured AgentToolCall from any JSON representation
pub fn parse_single_tool_call_value(val: &serde_json::Value) -> Option<AgentToolCall> {
    let tool_name = val.get("tool")
        .or_else(|| val.get("name"))
        .or_else(|| val.get("function"))
        .and_then(|v| v.as_str())?;

    let params = val.get("parameters")
        .or_else(|| val.get("input"))
        .or_else(|| val.get("arguments"))
        .unwrap_or(val);

    match tool_name {
        "read_file" => {
            let path = params.get("path")
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
            let query = params.get("query")
                .or_else(|| params.get("pattern"))
                .or_else(|| params.get("search"))
                .and_then(|v| v.as_str())?;
            Some(AgentToolCall::SearchCode {
                query: query.trim().to_string(),
            })
        }
        "list_directory" | "list_dir" | "ls" => {
            let path = params.get("path")
                .or_else(|| params.get("directory"))
                .or_else(|| params.get("dir"))
                .and_then(|v| v.as_str())
                .map(|s| s.trim().trim_matches('"').to_string());
            Some(AgentToolCall::ListDirectory { path })
        }
        "run_command" | "exec" | "terminal" => {
            let cmd = params.get("cmd")
                .or_else(|| params.get("command"))
                .and_then(|v| v.as_str())?;
            Some(AgentToolCall::RunCommand {
                cmd: cmd.trim().to_string(),
            })
        }
        "edit_file" => {
            let path = params.get("path")
                .or_else(|| params.get("file_path"))
                .and_then(|v| v.as_str())?;
            let search = params.get("search")
                .or_else(|| params.get("old_str"))
                .and_then(|v| v.as_str())?;
            let replace = params.get("replace")
                .or_else(|| params.get("new_str"))
                .and_then(|v| v.as_str())?;
            Some(AgentToolCall::EditFile {
                path: path.trim().trim_matches('"').to_string(),
                search: search.to_string(),
                replace: replace.to_string(),
            })
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

    // 2. Scan content for unquoted tool call JSON objects
    if results.is_empty() {
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('{') && trimmed.ends_with('}') && (trimmed.contains("\"tool\"") || trimmed.contains("\"name\"")) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                    if let Some(call) = parse_single_tool_call_value(&val) {
                        results.push(call);
                    }
                }
            }
        }
    }

    // 3. Backward compatibility: fallback to legacy pseudo <tool_call> <function=read_file>
    if results.is_empty() {
        let paths = extract_tool_call_read_paths(content);
        for p in paths {
            results.push(AgentToolCall::ReadFile {
                path: p,
                start_line: None,
                end_line: None,
            });
        }
    }

    results
}

/// Searches codebase for query string, skipping build/VCS artifacts and returning line-numbered snippets
pub fn execute_search_code(workspace: &Path, query: &str) -> AgentToolResult {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return AgentToolResult {
            tool_name: "search_code".to_string(),
            ok: false,
            output: "Search query cannot be empty".to_string(),
        };
    }

    let mut matches = Vec::new();
    let lower_query = trimmed.to_lowercase();

    fn walk_dir(
        dir: &Path,
        workspace: &Path,
        lower_query: &str,
        matches: &mut Vec<String>,
        max_matches: usize,
    ) {
        if matches.len() >= max_matches {
            return;
        }

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            if matches.len() >= max_matches {
                break;
            }

            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str.starts_with('.')
                || name_str == "node_modules"
                || name_str == "target"
                || name_str == "dist"
                || name_str == "build"
                || name_str == ".gemini"
                || name_str == ".git"
            {
                continue;
            }

            if path.is_dir() {
                walk_dir(&path, workspace, lower_query, matches, max_matches);
            } else if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                let is_code = matches!(
                    ext,
                    "rs" | "ts" | "tsx" | "js" | "jsx" | "vue" | "svelte" | "json" | "toml" | "html"
                        | "css" | "py" | "go" | "c" | "cpp" | "h" | "md" | "yaml" | "yml"
                );

                if !is_code {
                    continue;
                }

                if let Ok(meta) = path.metadata() {
                    if meta.len() > 500_000 {
                        continue;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(&path) {
                    let rel_path = path.strip_prefix(workspace).unwrap_or(&path).to_string_lossy();
                    for (line_idx, line) in content.lines().enumerate() {
                        if line.to_lowercase().contains(lower_query) {
                            let clean_line = line.trim();
                            let truncated_line = if clean_line.len() > 140 {
                                format!("{}...", &clean_line[..140])
                            } else {
                                clean_line.to_string()
                            };
                            matches.push(format!("{}:{}: {}", rel_path, line_idx + 1, truncated_line));
                            if matches.len() >= max_matches {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    walk_dir(workspace, workspace, &lower_query, &mut matches, 30);

    if matches.is_empty() {
        AgentToolResult {
            tool_name: "search_code".to_string(),
            ok: true,
            output: format!("No matches found for \"{}\" in workspace source files.", trimmed),
        }
    } else {
        AgentToolResult {
            tool_name: "search_code".to_string(),
            ok: true,
            output: format!(
                "Found {} match(es) for \"{}\":\n{}",
                matches.len(),
                trimmed,
                matches.join("\n")
            ),
        }
    }
}

/// Reads a file from workspace with optional start and end line ranges
pub fn execute_read_file(
    workspace: &Path,
    rel_path: &str,
    start_line: Option<usize>,
    end_line: Option<usize>,
) -> AgentToolResult {
    let cleaned = rel_path.trim().trim_matches('"').trim_matches('\'').trim();
    let full_path = if Path::new(cleaned).is_absolute() {
        PathBuf::from(cleaned)
    } else {
        workspace.join(cleaned)
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

            let s_idx = start_line.unwrap_or(1).saturating_sub(1).min(total_lines);
            let e_idx = end_line.unwrap_or(total_lines).min(total_lines);

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

            AgentToolResult {
                tool_name: "read_file".to_string(),
                ok: true,
                output: format!(
                    "File `{}` (lines {}-{} of {}):\n{}",
                    cleaned,
                    s_idx + 1,
                    e_idx,
                    total_lines,
                    formatted.join("\n")
                ),
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "read_file".to_string(),
            ok: false,
            output: format!("Failed to read file `{}`: {}", cleaned, e),
        },
    }
}

/// Lists files and directories within the workspace or a specified relative directory
pub fn execute_list_directory(workspace: &Path, rel_path: Option<&str>) -> AgentToolResult {
    let target_dir = match rel_path {
        Some(p) if !p.trim().is_empty() && p.trim() != "." => {
            let cleaned = p.trim().trim_matches('"').trim_matches('\'').trim();
            if Path::new(cleaned).is_absolute() {
                PathBuf::from(cleaned)
            } else {
                workspace.join(cleaned)
            }
        }
        _ => workspace.to_path_buf(),
    };

    let entries = match std::fs::read_dir(&target_dir) {
        Ok(e) => e,
        Err(err) => {
            return AgentToolResult {
                tool_name: "list_directory".to_string(),
                ok: false,
                output: format!("Failed to list directory `{}`: {}", target_dir.display(), err),
            };
        }
    };

    let mut dirs = Vec::new();
    let mut files = Vec::new();

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "node_modules" || name == "target" {
            continue;
        }

        if let Ok(file_type) = entry.file_type() {
            if file_type.is_dir() {
                dirs.push(format!("{}/", name));
            } else {
                let size_str = entry
                    .metadata()
                    .map(|m| {
                        let bytes = m.len();
                        if bytes < 1024 {
                            format!("{} B", bytes)
                        } else if bytes < 1024 * 1024 {
                            format!("{:.1} KB", bytes as f64 / 1024.0)
                        } else {
                            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
                        }
                    })
                    .unwrap_or_default();
                files.push(format!("{} ({})", name, size_str));
            }
        }
    }

    dirs.sort();
    files.sort();

    let display_path = target_dir.strip_prefix(workspace).unwrap_or(&target_dir).to_string_lossy();
    let label = if display_path.is_empty() || display_path == "." {
        "workspace root".to_string()
    } else {
        format!("`{}`", display_path)
    };

    let mut out = format!("Listing of {}:\n", label);
    for d in dirs {
        out.push_str(&format!("  📁 {}\n", d));
    }
    for f in files {
        out.push_str(&format!("  📄 {}\n", f));
    }

    AgentToolResult {
        tool_name: "list_directory".to_string(),
        ok: true,
        output: out.trim_end().to_string(),
    }
}

/// Executes a shell command non-destructively in workspace with security checks
pub fn execute_run_command(workspace: &Path, cmd: &str) -> AgentToolResult {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: "Command cannot be empty".to_string(),
        };
    }

    let lower = trimmed.to_lowercase();
    if lower.contains("rm -rf")
        || lower.contains("mkfs")
        || lower.contains("dd if=")
        || lower.contains(":(){ :|:& };:")
        || lower.contains("shutdown")
        || lower.contains("reboot")
        || lower.starts_with("sudo")
    {
        return AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: format!("Command blocked for safety reasons: `{}`", trimmed),
        };
    }

    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(trimmed)
        .current_dir(workspace)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{}\n{}", stdout, stderr).trim().to_string();
            let filtered = if combined.len() > 3000 {
                format!("{}...\n[Truncated {} bytes]", &combined[..3000], combined.len() - 3000)
            } else if combined.is_empty() {
                "(Command completed with empty output)".to_string()
            } else {
                combined
            };

            AgentToolResult {
                tool_name: "run_command".to_string(),
                ok: out.status.success(),
                output: filtered,
            }
        }
        Err(e) => AgentToolResult {
            tool_name: "run_command".to_string(),
            ok: false,
            output: format!("Failed to execute command `{}`: {}", trimmed, e),
        },
    }
}

/// Executes an AgentToolCall against the workspace
pub fn execute_agent_tool(workspace: &Path, call: &AgentToolCall) -> AgentToolResult {
    match call {
        AgentToolCall::ReadFile { path, start_line, end_line } => {
            execute_read_file(workspace, path, *start_line, *end_line)
        }
        AgentToolCall::SearchCode { query } => {
            execute_search_code(workspace, query)
        }
        AgentToolCall::ListDirectory { path } => {
            execute_list_directory(workspace, path.as_deref())
        }
        AgentToolCall::RunCommand { cmd } => {
            execute_run_command(workspace, cmd)
        }
        AgentToolCall::EditFile { path, search, replace } => {
            let full_path = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                workspace.join(path)
            };
            let block = SearchReplaceBlock {
                search: search.clone(),
                replace: replace.clone(),
            };
            let original = std::fs::read_to_string(&full_path).unwrap_or_default();
            match apply_search_replace(&original, &[block]) {
                Ok(patched) => match std::fs::write(&full_path, &patched) {
                    Ok(()) => AgentToolResult {
                        tool_name: "edit_file".to_string(),
                        ok: true,
                        output: format!("Successfully patched `{}`", path),
                    },
                    Err(e) => AgentToolResult {
                        tool_name: "edit_file".to_string(),
                        ok: false,
                        output: format!("Failed to write to `{}`: {}", path, e),
                    },
                },
                Err(err) => AgentToolResult {
                    tool_name: "edit_file".to_string(),
                    ok: false,
                    output: format!("Failed to apply patch to `{}`: {}", path, err),
                },
            }
        }
    }
}

/// Parses multiple proposed file blocks with relative paths from markdown,
/// supporting both full file creations and surgical SEARCH/REPLACE blocks.
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
                let path = if Path::new(cleaned).is_absolute() {
                    PathBuf::from(cleaned)
                } else {
                    workspace_root.join(cleaned)
                };
                current_target = Some(path);
            }
            continue;
        }

        if trimmed.starts_with("```") {
            flush_raw_patch(&current_target, &mut raw_patch_lines, &mut results);

            if in_block {
                if !code_lines.is_empty() {
                    let code = code_lines.join("\n");
                    let target = current_target.take().or_else(|| active_path.map(|p| p.to_path_buf()));
                    if let Some(target_file) = target {
                        let patch_blocks = parse_search_replace_blocks(&code);
                        let modif = if !patch_blocks.is_empty() {
                            TargetFileModification::new_patch(target_file.clone(), patch_blocks)
                        } else {
                            TargetFileModification::new_overwrite(target_file.clone(), code)
                        };

                        if let Some(existing) = results.iter_mut().find(|m| m.file_path == target_file) {
                            match (&mut existing.action, modif.action) {
                                (FileEditAction::SurgicalPatch(ref mut b1), FileEditAction::SurgicalPatch(b2)) => {
                                    b1.extend(b2);
                                }
                                (_, new_act) => {
                                    existing.action = new_act;
                                }
                            }
                        } else {
                            results.push(modif);
                        }
                    }
                }
                code_lines.clear();
                in_block = false;
            } else {
                in_block = true;
            }
        } else if in_block {
            code_lines.push(line.to_string());
        } else if current_target.is_some() {
            if is_search_marker(trimmed) || !raw_patch_lines.is_empty() {
                raw_patch_lines.push(line.to_string());
            }
        }
    }

    flush_raw_patch(&current_target, &mut raw_patch_lines, &mut results);

    results
}

/// Filters raw compiler stdout/stderr down to the most relevant error lines (diagnostics, error codes, file lines)
pub fn filter_compiler_errors(output: &str) -> String {
    let mut lines = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        // Skip compiler build progress / header lines
        if trimmed.starts_with("Checking ")
            || trimmed.starts_with("Compiling ")
            || trimmed.starts_with("Finished ")
            || trimmed.starts_with("Downloading ")
            || trimmed.starts_with("Downloaded ")
            || trimmed.is_empty()
        {
            continue;
        }

        if trimmed.contains("error[")
            || trimmed.contains("error:")
            || trimmed.contains("-->")
            || trimmed.contains("error TS")
            || trimmed.contains("::: ")
            || trimmed.contains("help:")
            || trimmed.contains("warning:")
        {
            lines.push(trimmed.to_string());
            if lines.len() >= 20 {
                break;
            }
        }
    }

    if lines.is_empty() {
        // Fallback: take first 10 non-empty lines
        for line in output.lines().filter(|l| !l.trim().is_empty()).take(10) {
            lines.push(line.trim().to_string());
        }
    }

    if lines.is_empty() {
        "Unknown compiler error".to_string()
    } else {
        lines.join("\n")
    }
}

/// Detects workspace project compiler (Rust Cargo or TypeScript tsc) and runs non-destructive check
pub fn run_project_compiler_check(workspace_root: &Path) -> Option<Result<(), String>> {
    // 1. Rust workspace / crate
    if workspace_root.join("Cargo.toml").exists() {
        let res = std::process::Command::new("cargo")
            .arg("check")
            .arg("--message-format=short")
            .current_dir(workspace_root)
            .output();

        if let Ok(out) = res {
            if out.status.success() {
                return Some(Ok(()));
            } else {
                let combined = format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                return Some(Err(filter_compiler_errors(&combined)));
            }
        } else {
            return None;
        }
    }

    // 2. TypeScript / JavaScript project
    if workspace_root.join("tsconfig.json").exists() {
        let res = std::process::Command::new("npx")
            .arg("tsc")
            .arg("--noEmit")
            .current_dir(workspace_root)
            .output();

        if let Ok(out) = res {
            if out.status.success() {
                return Some(Ok(()));
            } else {
                let combined = format!(
                    "{}\n{}",
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stderr)
                );
                return Some(Err(filter_compiler_errors(&combined)));
            }
        } else {
            return None;
        }
    }

    None
}

/// Generates a focused prompt asking the autonomous agent to repair the compiler errors
pub fn build_self_healing_prompt(
    original_goal: &str,
    compiler_errors: &str,
    target_file: Option<&str>,
) -> String {
    let file_hint = if let Some(f) = target_file {
        format!("in `{}`", f)
    } else {
        "in the target files".to_string()
    };

    format!(
        "COMPILER ERROR SELF-HEALING REQUEST:\n\
        The previous code generated for `{}` failed compiler check.\n\n\
        Compiler Diagnostics:\n\
        ```\n\
        {}\n\
        ```\n\n\
        Instructions:\n\
        1. Analyze and fix all compiler errors shown above {}.\n\
        2. Provide the corrected complete code for each affected file using `### File: <relative_path>` tags and fenced code blocks.\n\
        3. Do not omit any code or use placeholders. Return working, clean code.",
        original_goal, compiler_errors, file_hint
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_compiler_errors() {
        let raw = "\
   Compiling zenthree v0.1.0 (/path)
error[E0425]: cannot find value `x` in this scope
  --> src/main.rs:10:5
   |
10 |     x + 1
   |     ^ not found in this scope
help: a local variable with a similar name exists: `y`
   Finished dev profile [unoptimized + debuginfo]
";
        let filtered = filter_compiler_errors(raw);
        assert!(filtered.contains("error[E0425]: cannot find value `x` in this scope"));
        assert!(filtered.contains("--> src/main.rs:10:5"));
        assert!(filtered.contains("help: a local variable with a similar name exists: `y`"));
        assert!(!filtered.contains("Compiling zenthree"));
        assert!(!filtered.contains("Finished dev profile"));
    }

    #[test]
    fn test_build_self_healing_prompt() {
        let prompt = build_self_healing_prompt("add navbar", "error[E0425]: missing symbol", Some("src/Navbar.tsx"));
        assert!(prompt.contains("COMPILER ERROR SELF-HEALING REQUEST"));
        assert!(prompt.contains("add navbar"));
        assert!(prompt.contains("error[E0425]: missing symbol"));
        assert!(prompt.contains("in `src/Navbar.tsx`"));
    }

    #[test]
    fn test_extract_tool_call_read_paths() {
        let raw = r#"<tool_call> <function=read_file> <parameter=file_paths> [ "src/routes/services/index.tsx", "src/components/ui/Button.tsx" ] </tool_call>"#;
        let paths = extract_tool_call_read_paths(raw);
        assert_eq!(paths.len(), 2);
        assert!(paths.contains(&"src/routes/services/index.tsx".to_string()));
        assert!(paths.contains(&"src/components/ui/Button.tsx".to_string()));
    }

    #[test]
    fn test_apply_search_replace_exact() {
        let original = "fn calculate() -> i32 {\n    let a = 10;\n    let b = 20;\n    a + b\n}";
        let blocks = vec![SearchReplaceBlock {
            search: "    let b = 20;\n    a + b".to_string(),
            replace: "    let b = 50;\n    let c = 5;\n    a + b + c".to_string(),
        }];

        let result = apply_search_replace(original, &blocks).unwrap();
        assert!(result.contains("let b = 50;"));
        assert!(result.contains("a + b + c"));
        assert!(result.contains("let a = 10;"));
    }

    #[test]
    fn test_apply_search_replace_crlf_normalization() {
        let original = "function test() {\r\n    return 42;\r\n}";
        let blocks = vec![SearchReplaceBlock {
            search: "function test() {\n    return 42;\n}".to_string(),
            replace: "function test() {\n    return 100;\n}".to_string(),
        }];

        let result = apply_search_replace(original, &blocks).unwrap();
        assert!(result.contains("return 100;"));
    }

    #[test]
    fn test_apply_search_replace_indentation_adjustment() {
        // Original has 8 spaces indent
        let original = "impl Service {\n        pub fn execute() {\n        let val = 1;\n        println!(\"{}\", val);\n    }\n}";
        // LLM generates search with 4 spaces indent
        let blocks = vec![SearchReplaceBlock {
            search: "    let val = 1;\n    println!(\"{}\", val);".to_string(),
            replace: "    let val = 100;\n    log::info!(\"val={}\", val);".to_string(),
        }];

        let result = apply_search_replace(original, &blocks).unwrap();
        // Result should adapt to 8 spaces indentation
        assert!(result.contains("        let val = 100;"));
        assert!(result.contains("        log::info!(\"val={}\", val);"));
    }

    #[test]
    fn test_apply_search_replace_multiple_blocks() {
        let original = "const A = 1;\nconst B = 2;\nconst C = 3;";
        let blocks = vec![
            SearchReplaceBlock {
                search: "const A = 1;".to_string(),
                replace: "const A = 100;".to_string(),
            },
            SearchReplaceBlock {
                search: "const C = 3;".to_string(),
                replace: "const C = 300;".to_string(),
            },
        ];

        let result = apply_search_replace(original, &blocks).unwrap();
        assert_eq!(result, "const A = 100;\nconst B = 2;\nconst C = 300;");
    }

    #[test]
    fn test_parse_search_replace_blocks() {
        let raw = "\
<<<<<<< SEARCH
let x = 1;
=======
let x = 2;
>>>>>>> REPLACE
Some text in between
<<<<<<< SEARCH
let y = 10;
=======
let y = 20;
>>>>>>> REPLACE";

        let blocks = parse_search_replace_blocks(raw);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].search, "let x = 1;");
        assert_eq!(blocks[0].replace, "let x = 2;");
        assert_eq!(blocks[1].search, "let y = 10;");
        assert_eq!(blocks[1].replace, "let y = 20;");
    }

    #[test]
    fn test_extract_surgical_patch_markdown() {
        let markdown = "\
### File: src/components/Header.tsx
```tsx
<<<<<<< SEARCH
<h1>Old Title</h1>
=======
<h1>New Dynamic Title</h1>
>>>>>>> REPLACE
```";
        let root = PathBuf::from("/test/project");
        let mods = extract_all_file_modifications(markdown, &root, None);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].file_path, root.join("src/components/Header.tsx"));

        let patched = mods[0].apply_to("<div>\n<h1>Old Title</h1>\n</div>").unwrap();
        assert!(patched.contains("<h1>New Dynamic Title</h1>"));
    }

    #[test]
    fn test_extract_json_tool_edit_file() {
        let json_text = r#"
```json
{
  "tool": "edit_file",
  "path": "src/utils.ts",
  "search": "export const foo = 1;",
  "replace": "export const foo = 42;"
}
```
"#;
        let root = PathBuf::from("/test/project");
        let mods = extract_all_file_modifications(json_text, &root, None);
        assert_eq!(mods.len(), 1);
        assert_eq!(mods[0].file_path, root.join("src/utils.ts"));

        let patched = mods[0].apply_to("export const foo = 1;").unwrap();
        assert_eq!(patched, "export const foo = 42;");
    }

    #[test]
    fn test_parse_agent_tool_calls_json() {
        let content = r#"
```json
[
  { "tool": "search_code", "query": "auth_token" },
  { "tool": "read_file", "path": "src/state.rs", "start_line": 10, "end_line": 25 }
]
```
"#;
        let calls = parse_agent_tool_calls(content);
        assert_eq!(calls.len(), 2);
        match &calls[0] {
            AgentToolCall::SearchCode { query } => assert_eq!(query, "auth_token"),
            _ => panic!("Expected SearchCode"),
        }
        match &calls[1] {
            AgentToolCall::ReadFile { path, start_line, end_line } => {
                assert_eq!(path, "src/state.rs");
                assert_eq!(*start_line, Some(10));
                assert_eq!(*end_line, Some(25));
            }
            _ => panic!("Expected ReadFile"),
        }
    }

    #[test]
    fn test_execute_read_file_line_range() {
        let tmp = std::env::temp_dir().join("zenthree_read_test");
        let _ = std::fs::create_dir_all(&tmp);
        let sample_file = tmp.join("sample.txt");
        let _ = std::fs::write(&sample_file, "line1\nline2\nline3\nline4\nline5\n");

        let res = execute_read_file(&tmp, "sample.txt", Some(2), Some(4));
        assert!(res.ok);
        assert!(res.output.contains("line2"));
        assert!(res.output.contains("line3"));
        assert!(res.output.contains("line4"));
        assert!(!res.output.contains("line1"));
        assert!(!res.output.contains("line5"));

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_execute_search_code() {
        let tmp = std::env::temp_dir().join("zenthree_search_test");
        let src_dir = tmp.join("src");
        let _ = std::fs::create_dir_all(&src_dir);
        let file = src_dir.join("main.rs");
        let _ = std::fs::write(&file, "fn start_engine() {\n    let token = \"secret_xyz\";\n}\n");

        let res = execute_search_code(&tmp, "secret_xyz");
        assert!(res.ok);
        assert!(res.output.contains("src/main.rs:2:"));
        assert!(res.output.contains("secret_xyz"));

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_execute_list_directory() {
        let tmp = std::env::temp_dir().join("zenthree_list_test");
        let sub = tmp.join("components");
        let _ = std::fs::create_dir_all(&sub);
        let _ = std::fs::write(tmp.join("index.ts"), "export {}");

        let res = execute_list_directory(&tmp, None);
        assert!(res.ok);
        assert!(res.output.contains("components/"));
        assert!(res.output.contains("index.ts"));

        let _ = std::fs::remove_dir_all(tmp);
    }

    #[test]
    fn test_agent_mission_task_progression_and_failure_resilience() {
        let mut mission = AgentMission {
            id: "m1".to_string(),
            goal: "test goal".to_string(),
            tasks: vec![
                AgentTask {
                    id: "t1".to_string(),
                    title: "Task 1".to_string(),
                    status: TaskStatus::InProgress,
                    target_file: None,
                    diff_summary: None,
                },
                AgentTask {
                    id: "t2".to_string(),
                    title: "Task 2".to_string(),
                    status: TaskStatus::Pending,
                    target_file: None,
                    diff_summary: None,
                },
                AgentTask {
                    id: "t3".to_string(),
                    title: "Task 3".to_string(),
                    status: TaskStatus::Pending,
                    target_file: None,
                    diff_summary: None,
                },
            ],
            active_task_index: Some(0),
            is_active: true,
            is_paused: false,
            status_message: "Starting".to_string(),
            self_healing_attempts: 0,
        };

        // Task 1 fails -> Mission continues to Task 2!
        mission.mark_task_failed_and_continue(0, "Mock failure".to_string());
        assert_eq!(mission.tasks[0].status, TaskStatus::Failed("Mock failure".to_string()));
        assert_eq!(mission.active_task_index, Some(1));
        assert_eq!(mission.tasks[1].status, TaskStatus::InProgress);
        assert!(mission.is_active);

        // Task 2 completes -> Mission advances to Task 3
        mission.complete_task_and_advance(1);
        assert_eq!(mission.tasks[1].status, TaskStatus::Completed);
        assert_eq!(mission.active_task_index, Some(2));
        assert_eq!(mission.tasks[2].status, TaskStatus::InProgress);
        assert!(mission.is_active);

        // Task 3 completes -> Mission finished
        mission.complete_task_and_advance(2);
        assert_eq!(mission.tasks[2].status, TaskStatus::Completed);
        assert_eq!(mission.active_task_index, None);
        assert!(!mission.is_active);
    }

    #[test]
    fn test_classify_provider_error() {
        assert_eq!(
            classify_provider_error("429 Too Many Requests"),
            ProviderError::Transient("429 Too Many Requests".to_string())
        );
        assert_eq!(
            classify_provider_error("503 Service Temporarily Unavailable"),
            ProviderError::Transient("503 Service Temporarily Unavailable".to_string())
        );
        assert_eq!(
            classify_provider_error("404 Model Not Found"),
            ProviderError::Fatal("404 Model Not Found".to_string())
        );
        assert_eq!(
            classify_provider_error("401 Invalid API Key"),
            ProviderError::Fatal("401 Invalid API Key".to_string())
        );
    }
}

