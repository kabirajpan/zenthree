pub mod matcher;
pub mod patch;
pub mod read;
pub mod search;
pub mod list;
pub mod command;
pub mod git;
pub mod environment;
pub mod parser;
pub mod registry;

use std::path::Path;
use serde::{Deserialize, Serialize};

pub use registry::{ToolRegistry, ToolDefinition, ToolParam};
pub use matcher::match_and_replace;
pub use patch::{SearchReplaceBlock, FileEditAction, TargetFileModification, apply_search_replace};
pub use read::{execute_read_file, extract_tool_call_read_paths};
pub use search::execute_search_code;
pub use list::execute_list_directory;
pub use command::{
    execute_run_command, execute_run_command_with_cwd, find_nearest_manifest_dir,
    resolve_command_cwd, is_scaffolding_command, detect_scaffolded_project_files,
    parse_cd_statement, PROJECT_MANIFEST_NAMES,
};
pub use git::{execute_git_status, execute_git_diff};
pub use environment::{DetectorAgent, execute_detect_environment};
pub use parser::{
    parse_search_replace_blocks, parse_agent_tool_calls, parse_single_tool_call_value,
    extract_all_file_modifications, extract_json_tool_modifications, ground_workspace_path,
    ground_workspace_path_with_context, verify_or_relocate_target_with_search,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
        #[serde(default)]
        cwd: Option<String>,
    },
    EditFile {
        path: String,
        search: String,
        replace: String,
    },
    DetectEnvironment {
        #[serde(default)]
        subpath: Option<String>,
    },
    GitStatus,
    GitDiff {
        #[serde(default)]
        staged_only: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentToolResult {
    pub tool_name: String,
    pub ok: bool,
    pub output: String,
}

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
        AgentToolCall::RunCommand { cmd, cwd } => {
            execute_run_command_with_cwd(workspace, cmd, cwd.as_deref())
        }
        AgentToolCall::EditFile { path, search, replace } => {
            let full_path = if Path::new(path).is_absolute() {
                std::path::PathBuf::from(path)
            } else {
                workspace.join(path)
            };
            match std::fs::read_to_string(&full_path) {
                Ok(content) => {
                    let blocks = vec![SearchReplaceBlock {
                        search: search.clone(),
                        replace: replace.clone(),
                    }];
                    match apply_search_replace(&content, &blocks) {
                        Ok(new_code) => match std::fs::write(&full_path, new_code) {
                            Ok(_) => AgentToolResult {
                                tool_name: "edit_file".to_string(),
                                ok: true,
                                output: format!("Successfully patched `{}`", path),
                            },
                            Err(e) => AgentToolResult {
                                tool_name: "edit_file".to_string(),
                                ok: false,
                                output: format!("Failed to write patched file `{}`: {}", path, e),
                            },
                        },
                        Err(e) => AgentToolResult {
                            tool_name: "edit_file".to_string(),
                            ok: false,
                            output: format!("Failed to apply patch to `{}`: {}", path, e),
                        },
                    }
                }
                Err(e) => AgentToolResult {
                    tool_name: "edit_file".to_string(),
                    ok: false,
                    output: format!("Failed to read target file `{}`: {}", path, e),
                },
            }
        }
        AgentToolCall::DetectEnvironment { subpath } => {
            execute_detect_environment(workspace, subpath.as_deref())
        }
        AgentToolCall::GitStatus => {
            execute_git_status(workspace)
        }
        AgentToolCall::GitDiff { staged_only } => {
            execute_git_diff(workspace, *staged_only)
        }
    }
}

pub fn execute_agent_tool_with_provider(
    workspace: &Path,
    call: &AgentToolCall,
    _provider: Option<&crate::providers::ProviderChain>,
) -> AgentToolResult {
    execute_agent_tool(workspace, call)
}

