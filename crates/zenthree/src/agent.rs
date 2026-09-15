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

#[derive(Debug, Clone)]
pub struct TargetFileModification {
    pub file_path: PathBuf,
    pub proposed_code: String,
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
        4. ABSOLUTE DIRECT CODE GENERATION REQUIREMENT:\n\
           - DO NOT emit fake `<tool_call>` tags, `<function=...>`, or say 'Let me read the files first'.\n\
           - You must write and output the complete, working code for ALL required files IMMEDIATELY in this response.\n\
           - Use the explicit file tag for each file:\n\
             `### File: <relative_path>`\n\
             followed by a fenced code block with the language tag (e.g. ```tsx, ```rust, ```html, ```css).\n\
        5. Never output placeholders, ellipses (`// ...`), or unfinished functions. Provide full, working implementations.\n\
        The editor parses `### File: <path>` directly to apply diffs, write files to disk, and verify with the compiler."
    )
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

/// Parses multiple proposed file blocks with relative paths from markdown
pub fn extract_all_file_modifications(
    content: &str,
    workspace_root: &Path,
    active_path: Option<&Path>,
) -> Vec<TargetFileModification> {
    let mut results = Vec::new();
    let mut current_target: Option<PathBuf> = None;
    let mut in_block = false;
    let mut code_lines = Vec::new();

    for line in content.lines() {
        let trimmed = line.trim();

        let stripped = if let Some(rest) = trimmed.strip_prefix("### File:") {
            Some(rest)
        } else if let Some(rest) = trimmed.strip_prefix("### file:") {
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
            let cleaned = candidate.trim().trim_matches('`').trim_matches('*').trim();
            if !cleaned.is_empty() {
                let path = if Path::new(cleaned).is_absolute() {
                    PathBuf::from(cleaned)
                } else {
                    workspace_root.join(cleaned)
                };
                current_target = Some(path);
            }
        }

        if trimmed.starts_with("```") {
            if in_block {
                if !code_lines.is_empty() {
                    let code = code_lines.join("\n");
                    let target = current_target.take().or_else(|| active_path.map(|p| p.to_path_buf()));
                    if let Some(target_file) = target {
                        results.push(TargetFileModification {
                            file_path: target_file,
                            proposed_code: code,
                        });
                    }
                }
                code_lines.clear();
                in_block = false;
            } else {
                in_block = true;
            }
        } else if in_block {
            code_lines.push(line);
        }
    }

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
}
