use super::fs::{edit_file_schema, read_file_schema, write_file_schema};
use super::search::{find_path_schema, grep_schema};
use super::terminal::terminal_schema;
use serde_json::Value;

/// Returns all native tool JSON Schemas ready for model function declarations.
pub fn all_tool_schemas() -> Vec<Value> {
    vec![
        read_file_schema(),
        write_file_schema(),
        edit_file_schema(),
        grep_schema(),
        find_path_schema(),
        terminal_schema(),
    ]
}

/// Returns all tool JSON Schemas including dynamically registered external MCP tools.
pub fn all_tool_schemas_with_mcp(bridge: Option<&super::mcp_bridge::McpToolBridge>) -> Vec<Value> {
    let mut schemas = all_tool_schemas();
    if let Some(mcp) = bridge {
        schemas.extend(mcp.to_llm_tool_schemas());
    }
    schemas
}

/// Formats complete documentation and schemas of all available engineering tools for the system prompt.
pub fn format_tool_definitions_prompt() -> String {
    let mut out = String::new();
    out.push_str("## Available Engineering Tools & Calling Formats\n");
    out.push_str("You have access to the following tools to inspect, modify, and execute commands in the workspace:\n\n");

    out.push_str("### 1. `write_file`\n");
    out.push_str("Creates a new file or completely overwrites an existing file with the provided content.\n");
    out.push_str("- Parameter `path`: (string, required) Path relative to the workspace root\n");
    out.push_str("- Parameter `content`: (string, required) The complete file content\n");
    out.push_str("JSON Format:\n`{\"tool\": \"write_file\", \"path\": \"path/to/file.ext\", \"content\": \"...full code...\"}`\n");
    out.push_str("XML Format:\n`<tool_call><function=write_file><parameter=path>path/to/file.ext</parameter><parameter=content>...full code...</parameter></tool_call>`\n\n");

    out.push_str("### 2. `edit_file`\n");
    out.push_str("Surgically replaces a specific block of code in an existing file using exact matching.\n");
    out.push_str("Format:\n");
    out.push_str("### File: path/to/file.ext\n");
    out.push_str("<<<<<<< SEARCH\n");
    out.push_str("[exact lines from existing file to replace]\n");
    out.push_str("=======\n");
    out.push_str("[new replacement lines]\n");
    out.push_str(">>>>>>> REPLACE\n\n");

    out.push_str("### 3. `terminal`\n");
    out.push_str("Executes a shell/CLI command inside the project workspace directory (e.g. bun, npm, cargo, git).\n");
    out.push_str("- Parameter `command`: (string, required) Command line to execute\n");
    out.push_str("JSON Format:\n`{\"tool\": \"terminal\", \"command\": \"bun run dev\"}`\n");
    out.push_str("XML Format:\n`<tool_call><function=terminal><parameter=command>bun run dev</parameter></tool_call>`\n\n");

    out.push_str("### 4. `read_file`\n");
    out.push_str("Reads lines of a file from the workspace.\n");
    out.push_str("- Parameter `path`: (string, required) File path relative to workspace root\n");
    out.push_str("- Parameter `start_line`: (integer, optional) 1-based start line\n");
    out.push_str("- Parameter `end_line`: (integer, optional) 1-based end line\n");
    out.push_str("Format:\n`{\"tool\": \"read_file\", \"path\": \"src/main.rs\", \"start_line\": 1, \"end_line\": 60}`\n\n");

    out.push_str("### 5. `list_directory`\n");
    out.push_str("Lists all files and directories in a directory to explore the project structure.\n");
    out.push_str("- Parameter `path`: (string, optional, defaults to \".\")\n");
    out.push_str("Format:\n`{\"tool\": \"list_directory\", \"path\": \".\"}`\n\n");

    out.push_str("### 6. `grep`\n");
    out.push_str("Searches for regex patterns or text symbols across files in the workspace.\n");
    out.push_str("- Parameter `regex`: (string, required) Search pattern\n");
    out.push_str("- Parameter `include_pattern`: (string, optional) Glob filter like `**/*.rs` or `**/*.tsx`\n");
    out.push_str("Format:\n`{\"tool\": \"grep\", \"regex\": \"useState\", \"include_pattern\": \"**/*.tsx\"}`\n\n");

    out.push_str("CRITICAL: When the user asks you to create files or build applications, immediately output the `write_file` or `terminal` tool invocation. Never just describe what to do—execute it!");
    out
}

/// Generates the few-shot `<tool_call_examples>` block proven to boost tool calling accuracy.
pub fn format_few_shot_tool_examples() -> String {
    let mut out = String::new();
    out.push_str("<tool_call_examples>\n");
    out.push_str("Example 1: Reading a specific file range\n");
    out.push_str("User: \"How does the agent handle secret scrubbing?\"\n");
    out.push_str("Assistant:\n");
    out.push_str("<thought>I will inspect `crates/agent/src/environment/secret_scrubber.rs` to review the regex patterns.</thought>\n");
    out.push_str("{\"tool\": \"read_file\", \"path\": \"crates/agent/src/environment/secret_scrubber.rs\", \"start_line\": 1, \"end_line\": 60}\n\n");

    out.push_str("Example 2: Finding code symbols with grep\n");
    out.push_str("User: \"Where is the PermissionEngine defined?\"\n");
    out.push_str("Assistant:\n");
    out.push_str("<thought>Searching for struct PermissionEngine across Rust files.</thought>\n");
    out.push_str("{\"tool\": \"grep\", \"regex\": \"struct PermissionEngine\", \"include_pattern\": \"**/*.rs\"}\n\n");

    out.push_str("Example 3: Surgical code modification\n");
    out.push_str("User: \"Update the timeout default to 45 seconds\"\n");
    out.push_str("Assistant:\n");
    out.push_str("<thought>Updating DEFAULT_TIMEOUT_SECS in process_runner.rs.</thought>\n");
    out.push_str("<<<<<<< SEARCH\n");
    out.push_str("pub const DEFAULT_TIMEOUT_SECS: u64 = 30;\n");
    out.push_str("=======\n");
    out.push_str("pub const DEFAULT_TIMEOUT_SECS: u64 = 45;\n");
    out.push_str(">>>>>>> REPLACE\n\n");

    out.push_str("Example 4: Running a verification command\n");
    out.push_str("User: \"Check if the agent crate compiles\"\n");
    out.push_str("Assistant:\n");
    out.push_str("<thought>Running cargo check on the agent crate.</thought>\n");
    out.push_str("{\"tool\": \"terminal\", \"command\": \"cargo check -p agent\"}\n");
    out.push_str("</tool_call_examples>");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tool_schemas_validity() {
        let schemas = all_tool_schemas();
        assert_eq!(schemas.len(), 6);

        for schema in schemas {
            assert!(schema.get("name").is_some());
            assert!(schema.get("description").is_some());
            assert!(schema.get("parameters").is_some());
        }
    }

    #[test]
    fn test_format_few_shot_tool_examples() {
        let block = format_few_shot_tool_examples();
        assert!(block.starts_with("<tool_call_examples>"));
        assert!(block.contains("read_file"));
        assert!(block.contains("grep"));
        assert!(block.contains("SEARCH"));
        assert!(block.contains("terminal"));
        assert!(block.ends_with("</tool_call_examples>"));
    }
}
