use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParam {
    pub name: &'static str,
    pub param_type: &'static str,
    pub description: &'static str,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub params: Vec<ToolParam>,
    pub example: &'static str,
}

/// Central Tool Registry exposing all available tools, their schemas, and execution examples.
pub struct ToolRegistry;

impl ToolRegistry {
    pub fn all() -> Vec<ToolDefinition> {
        vec![
            ToolDefinition {
                name: "run_command",
                description: "Execute a shell command in the workspace. Returns stdout and stderr.",
                params: vec![
                    ToolParam {
                        name: "cmd",
                        param_type: "string",
                        description: "The terminal command to execute (e.g. `cargo check`, `npm test`, `pytest`).",
                        required: true,
                    },
                    ToolParam {
                        name: "cwd",
                        param_type: "string",
                        description: "Working directory relative to workspace root.",
                        required: false,
                    },
                ],
                example: r#"{"tool": "run_command", "cmd": "cargo check"}"#,
            },
            ToolDefinition {
                name: "read_file",
                description: "Read file contents with line numbers from the workspace.",
                params: vec![
                    ToolParam {
                        name: "path",
                        param_type: "string",
                        description: "Workspace-relative file path.",
                        required: true,
                    },
                    ToolParam {
                        name: "start_line",
                        param_type: "integer",
                        description: "1-indexed starting line number.",
                        required: false,
                    },
                    ToolParam {
                        name: "end_line",
                        param_type: "integer",
                        description: "1-indexed ending line number.",
                        required: false,
                    },
                ],
                example: r#"{"tool": "read_file", "path": "src/main.rs", "start_line": 1, "end_line": 50}"#,
            },
            ToolDefinition {
                name: "search_code",
                description: "Search code or symbols across workspace files.",
                params: vec![
                    ToolParam {
                        name: "query",
                        param_type: "string",
                        description: "Regex pattern or text symbol to search for.",
                        required: true,
                    },
                ],
                example: r#"{"tool": "search_code", "query": "fn handle_event"}"#,
            },
            ToolDefinition {
                name: "list_directory",
                description: "List files and subdirectories in a workspace path.",
                params: vec![
                    ToolParam {
                        name: "path",
                        param_type: "string",
                        description: "Directory path relative to workspace (defaults to workspace root).",
                        required: false,
                    },
                ],
                example: r#"{"tool": "list_directory", "path": "src"}"#,
            },
            ToolDefinition {
                name: "edit_file",
                description: "Surgically edit an existing file using search and replace.",
                params: vec![
                    ToolParam {
                        name: "path",
                        param_type: "string",
                        description: "Workspace-relative file path.",
                        required: true,
                    },
                    ToolParam {
                        name: "search",
                        param_type: "string",
                        description: "Exact existing code snippet to locate.",
                        required: true,
                    },
                    ToolParam {
                        name: "replace",
                        param_type: "string",
                        description: "New replacement code snippet.",
                        required: true,
                    },
                ],
                example: r#"{"tool": "edit_file", "path": "src/config.json", "search": "\"port\": 3000", "replace": "\"port\": 8080"}"#,
            },
            ToolDefinition {
                name: "detect_environment",
                description: "Detect language, build system, and package manager for the workspace.",
                params: vec![
                    ToolParam {
                        name: "subpath",
                        param_type: "string",
                        description: "Subdirectory to inspect.",
                        required: false,
                    },
                ],
                example: r#"{"tool": "detect_environment"}"#,
            },
            ToolDefinition {
                name: "git_status",
                description: "Check git working tree status, staged, and unstaged modifications.",
                params: vec![],
                example: r#"{"tool": "git_status"}"#,
            },
            ToolDefinition {
                name: "git_diff",
                description: "Inspect git diff of modified files against HEAD.",
                params: vec![
                    ToolParam {
                        name: "staged_only",
                        param_type: "boolean",
                        description: "Whether to view only staged git changes.",
                        required: false,
                    },
                ],
                example: r#"{"tool": "git_diff", "staged_only": false}"#,
            },
        ]
    }

    /// Renders clean, structured tool documentation with schemas and examples for the LLM.
    pub fn render_for_prompt() -> String {
        let mut out = String::from("### Tool Registry:\nTo call a tool, output a JSON object: `{\"tool\": \"<tool_name>\", ...}`.\n\n");
        for (i, tool) in Self::all().iter().enumerate() {
            out.push_str(&format!("{}. `{}`\n- Description: {}\n", i + 1, tool.name, tool.description));
            if !tool.params.is_empty() {
                out.push_str("- Parameters:\n");
                for p in &tool.params {
                    let req = if p.required { "required" } else { "optional" };
                    out.push_str(&format!("  * `{}` ({}, {}): {}\n", p.name, p.param_type, req, p.description));
                }
            } else {
                out.push_str("- Parameters: None\n");
            }
            out.push_str(&format!("- Example: `{}`\n\n", tool.example));
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry_render() {
        let rendered = ToolRegistry::render_for_prompt();
        assert!(rendered.contains("### Tool Registry:"));
        assert!(rendered.contains("`run_command`"));
        assert!(rendered.contains("`read_file`"));
        assert!(rendered.contains("`search_code`"));
        assert!(rendered.contains("`edit_file`"));
        assert!(rendered.contains("`git_status`"));
        assert!(rendered.contains(r#"{"tool": "run_command", "cmd": "cargo check"}"#));
    }
}
