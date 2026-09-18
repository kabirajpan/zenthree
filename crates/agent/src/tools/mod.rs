pub mod matcher;
pub mod patch;
pub mod read;
pub mod search;
pub mod list;
pub mod command;
pub mod git;
pub mod environment;
pub mod parser;
pub mod fs;
pub mod terminal;
pub mod schema;

use std::path::Path;
use serde::{Deserialize, Serialize};

pub use matcher::match_and_replace;
pub use patch::{SearchReplaceBlock, FileEditAction, TargetFileModification, apply_search_replace};
pub use read::{execute_read_file, extract_tool_call_read_paths};
pub use search::{
    execute_search_code, execute_grep, execute_find_path, grep_schema, find_path_schema,
    GrepInput, GrepMatch, GrepOutput, FindPathInput, FindPathOutput,
};
pub use list::execute_list_directory;
pub use command::execute_run_command;
pub use git::{execute_git_status, execute_git_diff};
pub use environment::{DetectorAgent, execute_detect_environment};
pub use parser::{
    parse_search_replace_blocks, parse_agent_tool_calls, parse_single_tool_call_value,
    parse_tool_call_values, extract_all_file_modifications, extract_json_tool_modifications,
    ground_workspace_path,
};
pub mod mcp_bridge;

pub use fs::{
    execute_write_file, execute_edit_file, write_file_schema, edit_file_schema,
    read_file_schema, ReadFileInput, ReadFileOutput, WriteFileInput, WriteFileOutput,
    EditFileInput, EditFileOutput,
};
pub use terminal::{
    execute_terminal, terminal_schema, TerminalInput, TerminalOutput,
};
pub use schema::{all_tool_schemas, all_tool_schemas_with_mcp, format_few_shot_tool_examples, format_tool_definitions_prompt};
pub use mcp_bridge::{McpToolBridge, RegisteredMcpTool};

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
    },
    EditFile {
        path: String,
        search: String,
        replace: String,
    },
    WriteFile {
        path: String,
        content: String,
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
    Mcp {
        tool_name: String,
        #[serde(default)]
        arguments: serde_json::Value,
    },
}

impl AgentToolCall {
    pub fn tool_name(&self) -> &str {
        match self {
            AgentToolCall::ReadFile { .. } => "read_file",
            AgentToolCall::SearchCode { .. } => "grep",
            AgentToolCall::ListDirectory { .. } => "list_directory",
            AgentToolCall::RunCommand { .. } => "terminal",
            AgentToolCall::EditFile { .. } => "edit_file",
            AgentToolCall::WriteFile { .. } => "write_file",
            AgentToolCall::DetectEnvironment { .. } => "detect_environment",
            AgentToolCall::GitStatus => "git_status",
            AgentToolCall::GitDiff { .. } => "git_diff",
            AgentToolCall::Mcp { tool_name, .. } => tool_name.as_str(),
        }
    }

    pub fn to_json_args(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or_else(|_| serde_json::json!({}))
    }
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
        AgentToolCall::RunCommand { cmd } => {
            execute_run_command(workspace, cmd)
        }
        AgentToolCall::EditFile { path, search, replace } => {
            let full_path = ground_workspace_path(workspace, path);
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
        AgentToolCall::WriteFile { path, content } => {
            let full_path = ground_workspace_path(workspace, path);
            if let Some(parent) = full_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            match std::fs::write(&full_path, content) {
                Ok(_) => AgentToolResult {
                    tool_name: "write_file".to_string(),
                    ok: true,
                    output: format!("Successfully wrote `{}`", path),
                },
                Err(e) => AgentToolResult {
                    tool_name: "write_file".to_string(),
                    ok: false,
                    output: format!("Failed to write file `{}`: {}", path, e),
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
        AgentToolCall::Mcp { tool_name, .. } => {
            AgentToolResult {
                tool_name: tool_name.clone(),
                ok: false,
                output: format!("External MCP tool '{}' requires an active McpToolBridge to execute", tool_name),
            }
        }
    }
}

pub fn execute_agent_tool_with_bridge(
    workspace: &Path,
    call: &AgentToolCall,
    bridge: Option<&McpToolBridge>,
) -> AgentToolResult {
    match call {
        AgentToolCall::Mcp { tool_name, arguments } => {
            if let Some(mcp) = bridge {
                mcp.execute_mcp_tool(tool_name, Some(arguments.clone()), std::time::Duration::from_secs(30))
            } else {
                AgentToolResult {
                    tool_name: tool_name.clone(),
                    ok: false,
                    output: format!("No MCP bridge available to execute external tool '{}'", tool_name),
                }
            }
        }
        other => execute_agent_tool(workspace, other),
    }
}

pub fn execute_agent_tool_with_provider(
    workspace: &Path,
    call: &AgentToolCall,
    _provider: Option<&crate::providers::ProviderChain>,
) -> AgentToolResult {
    execute_agent_tool(workspace, call)
}

