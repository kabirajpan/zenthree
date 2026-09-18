//! Phase 4: Unified Tool Suite & Schema Registry Example
//!
//! Demonstrates how Zenthree implements, sandboxes, and exposes its native tool suite:
//! - JSON Schema generation compatible with OpenAI, Gemini, and Anthropic
//! - Few-shot prompt injection (<tool_call_examples>) boosting model pass rates
//! - Filesystem Tools: bounded read_file, atomic write_file, fuzzy edit_file
//! - Search Tools: ripgrep-backed grep and glob-based find_path
//! - Terminal Tool: sandboxed process execution wired with HardcodedSecurityRules
//!
//! Run with: `cargo run --example phase4_tools_demo -p agent`

use agent::environment::AgentEnvironment;
use agent::tools::fs::{execute_edit_file, execute_read_file, execute_write_file};
use agent::tools::search::{execute_find_path, execute_grep};
use agent::tools::terminal::execute_terminal;
use agent::tools::{all_tool_schemas, format_few_shot_tool_examples};
use tempfile::tempdir;

fn main() {
    println!("============================================================");
    println!("       ZENTHREE AGENT: PHASE 4 UNIFIED TOOL SUITE DEMO      ");
    println!("============================================================\n");

    let temp_workspace = tempdir().expect("Failed to create temporary workspace");
    let ws = temp_workspace.path();
    let env = AgentEnvironment::new([ws]);

    // ---------------------------------------------------------------------
    // 1. Tool JSON Schemas & Few-Shot Prompt Block
    // ---------------------------------------------------------------------
    println!("--- 1. Unified Tool Schemas & Few-Shot Prompts ---");
    let schemas = all_tool_schemas();
    println!("Total registered native tool schemas: {}", schemas.len());
    for s in &schemas {
        let name = s.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
        println!("  - Tool declaration: `{}`", name);
    }

    println!("\nInjected Few-Shot `<tool_call_examples>` Prompt Block:");
    println!("------------------------------------------------------------");
    println!("{}", format_few_shot_tool_examples());
    println!("------------------------------------------------------------");

    // ---------------------------------------------------------------------
    // 2. Filesystem Tools (write_file, read_file, edit_file)
    // ---------------------------------------------------------------------
    println!("\n--- 2. Filesystem Tools Execution ---");
    let sample_rel_path = "crates/demo/src/main.rs";

    // A. write_file (Atomic creation)
    let initial_code = r#"fn main() {
    println!("Starting server on port 3000...");
    let retries = 3;
}
"#;
    let write_res = execute_write_file(ws, Some(&env.jail), sample_rel_path, initial_code)
        .expect("write_file should succeed");
    println!("✓ [write_file] Created `{}` ({} bytes, created={})", write_res.path, write_res.bytes_written, write_res.created);

    // B. read_file (cat -n line numbered)
    let read_res = execute_read_file(ws, Some(&env.jail), sample_rel_path, Some(1), Some(4))
        .expect("read_file should succeed");
    println!("✓ [read_file] Output in cat -n format:\n{}", read_res.content);

    // C. edit_file (Surgical search/replace)
    let search_target = "println!(\"Starting server on port 3000...\");";
    let replacement = "println!(\"Starting server on port 8080...\");";
    let edit_res = execute_edit_file(ws, Some(&env.jail), sample_rel_path, search_target, replacement)
        .expect("edit_file should succeed");
    println!("✓ [edit_file] Applied patch (replacements={}, strategy={})", edit_res.replacements_made, edit_res.strategy_used);

    // Verify patched content
    let post_read = execute_read_file(ws, Some(&env.jail), sample_rel_path, Some(2), Some(2)).unwrap();
    println!("✓ [verification] Line after patch:\n{}", post_read.content);

    // ---------------------------------------------------------------------
    // 3. Search Tools (find_path & grep)
    // ---------------------------------------------------------------------
    println!("--- 3. Search Tools Execution ---");
    // Create an auxiliary file for search testing
    execute_write_file(ws, Some(&env.jail), "docs/architecture.md", "# Architecture\nServer port configured in main.rs\n").unwrap();

    // A. find_path
    let find_res = execute_find_path(ws, "*main.rs", None, None).expect("find_path should succeed");
    println!("✓ [find_path] Matched {} file(s) for pattern `*main.rs`:", find_res.matches.len());
    for f in &find_res.matches {
        println!("    {}", f);
    }

    // B. grep
    let grep_res = execute_grep(ws, "port 8080", Some("**/*.rs"), None, None, None).expect("grep should succeed");
    println!("✓ [grep] Found {} match(es) for regex `port 8080`:", grep_res.matches.len());
    for m in &grep_res.matches {
        println!("    {}:#L{} -> {}", m.path, m.line_number, m.line_content.trim());
    }

    // ---------------------------------------------------------------------
    // 4. Sandboxed Terminal Tool Execution
    // ---------------------------------------------------------------------
    println!("\n--- 4. Sandboxed Terminal Execution & Security ---");
    // A. Benign command
    let term_res = execute_terminal(&env, None, "echo 'Zenthree Agent Tool Execution Verified'", None, None, None, None)
        .expect("Benign command should succeed");
    println!("✓ [terminal] Command `{}` -> Output: {}", term_res.command, term_res.output.trim());

    // B. Hardcoded security denial (rm -rf /)
    let attack_cmd = "rm -rf /";
    match execute_terminal(&env, None, attack_cmd, None, None, None, None) {
        Ok(_) => panic!("SECURITY FAILURE: Dangerous command was not blocked!"),
        Err(e) => println!("✓ [terminal] Blocked destructive attack `{}`: {}", attack_cmd, e),
    }

    // C. Protected path modification rejection
    match execute_write_file(ws, Some(&env.jail), ".git/config", "hacked") {
        Ok(_) => panic!("SECURITY FAILURE: Protected path write was not blocked!"),
        Err(e) => println!("✓ [write_file] Blocked invariant path write `.git/config`: {}", e),
    }

    println!("\n============================================================");
    println!("       PHASE 4 UNIFIED TOOL SUITE FULLY OPERATIONAL         ");
    println!("============================================================\n");
}
