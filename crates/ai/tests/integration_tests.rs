use ai::*;

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
    let original = "impl Service {\n        pub fn execute() {\n        let val = 1;\n        println!(\"{}\", val);\n    }\n}";
    let blocks = vec![SearchReplaceBlock {
        search: "    let val = 1;\n    println!(\"{}\", val);".to_string(),
        replace: "    let val = 100;\n    log::info!(\"val={}\", val);".to_string(),
    }];

    let result = apply_search_replace(original, &blocks).unwrap();
    assert!(result.contains("        let val = 100;"));
    assert!(result.contains("        log::info!(\"val={}\", val);"));
}

#[test]
fn test_parse_search_replace_blocks() {
    let text = r#"
Here is the code change:

```
<<<<<<< SEARCH
let x = 1;
=======
let x = 2;
>>>>>>> REPLACE
```
"#;
    let blocks = parse_search_replace_blocks(text);
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].search, "let x = 1;");
    assert_eq!(blocks[0].replace, "let x = 2;");
}

#[test]
fn test_parse_agent_tool_calls_json() {
    let text = r#"
I will search for the symbol:
```json
{
  "tool": "search_code",
  "query": "ConnectomeRNN"
}
```
"#;
    let calls = parse_agent_tool_calls(text);
    assert_eq!(calls.len(), 1);
    match &calls[0] {
        AgentToolCall::SearchCode { query } => assert_eq!(query, "ConnectomeRNN"),
        _ => panic!("Expected SearchCode call"),
    }
}

#[test]
fn test_parse_agent_tool_calls_command_variants() {
    let json_bash = r#"{"tool": "bash", "command": "cargo test"}"#;
    let calls = parse_agent_tool_calls(json_bash);
    assert_eq!(calls.len(), 1);
    match &calls[0] {
        AgentToolCall::RunCommand { cmd } => assert_eq!(cmd, "cargo test"),
        _ => panic!("Expected RunCommand call"),
    }

    let xml_cmd = r#"I will run tests: <run_command>bun test</run_command>"#;
    let calls_xml = parse_agent_tool_calls(xml_cmd);
    assert_eq!(calls_xml.len(), 1);
    match &calls_xml[0] {
        AgentToolCall::RunCommand { cmd } => assert_eq!(cmd, "bun test"),
        _ => panic!("Expected RunCommand call"),
    }

    let directive = "### Run: `npm run build`";
    let calls_dir = parse_agent_tool_calls(directive);
    assert_eq!(calls_dir.len(), 1);
    match &calls_dir[0] {
        AgentToolCall::RunCommand { cmd } => assert_eq!(cmd, "npm run build"),
        _ => panic!("Expected RunCommand call"),
    }

    // Fenced bash code block with arbitrary command (not whitelisted)
    let fenced = "```bash\npython3 scripts/benchmark.py --verbose\n```";
    let calls_fenced = parse_agent_tool_calls(fenced);
    assert_eq!(calls_fenced.len(), 1);
    match &calls_fenced[0] {
        AgentToolCall::RunCommand { cmd } => assert_eq!(cmd, "python3 scripts/benchmark.py --verbose"),
        _ => panic!("Expected RunCommand call"),
    }
}

#[test]
fn test_dangerous_commands_require_permission() {
    use ai::tools::command::{is_dangerous_command, execute_run_command};
    use std::path::Path;

    // Dangerous commands must be caught by safety guard
    assert!(is_dangerous_command("sudo apt-get install curl").is_some());
    assert!(is_dangerous_command("rm -rf /").is_some());
    assert!(is_dangerous_command("rm -rf ~").is_some());
    assert!(is_dangerous_command("mkfs.ext4 /dev/sda1").is_some());
    assert!(is_dangerous_command("shutdown now").is_some());
    assert!(is_dangerous_command("reboot").is_some());
    assert!(is_dangerous_command(":(){ :|:& };:").is_some());
    assert!(is_dangerous_command("git clean -fdx").is_some());

    // Safe / ordinary project commands must be permitted
    assert!(is_dangerous_command("cargo check").is_none());
    assert!(is_dangerous_command("bun run dev").is_none());
    assert!(is_dangerous_command("npm install").is_none());
    assert!(is_dangerous_command("python3 main.py").is_none());
    assert!(is_dangerous_command("echo 'hello world'").is_none());
    assert!(is_dangerous_command("rm -rf target").is_none());
    assert!(is_dangerous_command("rm -rf node_modules").is_none());

    // Verify execution output contains permission notice
    let res = execute_run_command(Path::new("."), "sudo rm -f /etc/hosts");
    assert!(!res.ok);
    assert!(res.output.contains("Permission required"));
}

#[test]
fn test_sub_agent_roles() {
    assert_eq!(SubAgentRole::Planner.badge(), "📋 Planner");
    assert_eq!(SubAgentRole::Explorer.badge(), "🔍 Explorer");
    assert_eq!(SubAgentRole::Detector.badge(), "🧠 Detector");
    assert_eq!(SubAgentRole::Builder.badge(), "⚡ Builder");
    assert_eq!(SubAgentRole::Tester.badge(), "🛡️ Tester");
}

#[test]
fn test_user_intent_classification() {
    assert_eq!(classify_user_intent("can you explain this project architecture?"), UserIntent::CodeExplanation);
    assert_eq!(classify_user_intent("refactor the auth handler"), UserIntent::Refactoring);
    assert_eq!(classify_user_intent("fix compilation bug in session.rs"), UserIntent::BugFix);
    assert_eq!(classify_user_intent("create a new route for dashboard"), UserIntent::CodeGeneration);
    assert_eq!(classify_user_intent("plan the implementation steps"), UserIntent::ArchitecturePlanning);
}

#[test]
fn test_mission_non_halting_task_failure() {
    let mut mission = create_mission_for_prompt("Refactor database engine", &[]);
    assert_eq!(mission.tasks.len(), 4);
    assert_eq!(mission.tasks[0].sub_agent, Some(SubAgentRole::Explorer));
    assert_eq!(mission.tasks[1].sub_agent, Some(SubAgentRole::Detector));
    assert_eq!(mission.tasks[2].sub_agent, Some(SubAgentRole::Builder));
    assert_eq!(mission.tasks[3].sub_agent, Some(SubAgentRole::Tester));

    // Mark task 2 failed - mission should advance to task 3 and continue
    mission.mark_task_failed_and_continue(2, "Compilation failed: syntax error".to_string());
    assert_eq!(mission.active_task_index, Some(3));
    assert!(mission.is_active);
    assert!(mission.status_message.contains("Task 3 failed"));
}

#[test]
fn test_output_truncation_guard() {
    let mut big_output = String::new();
    for i in 0..100 {
        big_output.push_str(&format!("Line {}\n", i));
    }
    let truncated = truncate_output(&big_output, 10, 500);
    assert!(truncated.lines().count() <= 12);
    assert!(truncated.contains("additional lines truncated"));
}


#[test]
fn test_agent_loop_context_and_cancellation() {
    let mut ctx = AgentLoopContext::new(std::path::PathBuf::from("/tmp"));
    ctx.max_iterations = 5;
    assert_eq!(ctx.max_iterations, 5);
    assert!(!ctx.cancellation_token.is_cancelled());

    ctx.cancellation_token.cancel();
    assert!(ctx.cancellation_token.is_cancelled());

    ctx.cancellation_token.reset();
    assert!(!ctx.cancellation_token.is_cancelled());
}

#[test]
fn test_continuous_server_command_detection() {
    use ai::tools::command::is_continuous_server_command;

    // Scaffolding / setup / build / test commands MUST be false (run and finish)
    assert!(!is_continuous_server_command("bun create vite calculator-app --template react"));
    assert!(!is_continuous_server_command("npm create vite@latest my-app"));
    assert!(!is_continuous_server_command("npm install"));
    assert!(!is_continuous_server_command("bun install"));
    assert!(!is_continuous_server_command("cargo build"));
    assert!(!is_continuous_server_command("cargo check"));
    assert!(!is_continuous_server_command("bun test"));

    // Long-running dev servers MUST be true
    assert!(is_continuous_server_command("bun run dev"));
    assert!(is_continuous_server_command("npm run dev"));
    assert!(is_continuous_server_command("npm start"));
    assert!(is_continuous_server_command("vite"));
    assert!(is_continuous_server_command("python3 -m http.server 8000"));
}

