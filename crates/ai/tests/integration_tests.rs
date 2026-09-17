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
fn test_deliberation_bridge() {
    let (conf, dt, res) = DeliberationBridge::deliberate_intent("Implement new API endpoint");
    assert!(conf >= 0.0 && conf <= 1.0);
    assert!(dt > 0);
    assert!(res.is_some());
}

#[test]
fn test_agent_session_lazy_in_memory() {
    let tmp = std::env::temp_dir().join(format!("ai_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let session = AgentSession::new(&tmp, Some("Lazy Session"));
    assert_eq!(session.title, "Lazy Session");
    // Zero files on disk until saved
    assert!(!tmp.join(".zenthree").exists());
}
