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
