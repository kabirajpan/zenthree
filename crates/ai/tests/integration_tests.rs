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
        AgentToolCall::RunCommand { cmd, .. } => assert_eq!(cmd, "cargo test"),
        _ => panic!("Expected RunCommand call"),
    }

    let xml_cmd = r#"I will run tests: <run_command>bun test</run_command>"#;
    let calls_xml = parse_agent_tool_calls(xml_cmd);
    assert_eq!(calls_xml.len(), 1);
    match &calls_xml[0] {
        AgentToolCall::RunCommand { cmd, .. } => assert_eq!(cmd, "bun test"),
        _ => panic!("Expected RunCommand call"),
    }

    let directive = "### Run: `npm run build`";
    let calls_dir = parse_agent_tool_calls(directive);
    assert_eq!(calls_dir.len(), 1);
    match &calls_dir[0] {
        AgentToolCall::RunCommand { cmd, .. } => assert_eq!(cmd, "npm run build"),
        _ => panic!("Expected RunCommand call"),
    }

    // Fenced bash code block with arbitrary command (not whitelisted)
    let fenced = "```bash\npython3 scripts/benchmark.py --verbose\n```";
    let calls_fenced = parse_agent_tool_calls(fenced);
    assert_eq!(calls_fenced.len(), 1);
    match &calls_fenced[0] {
        AgentToolCall::RunCommand { cmd, .. } => assert_eq!(cmd, "python3 scripts/benchmark.py --verbose"),
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

    // Conversational, advisory, and brainstorming queries MUST be DiscussionAndGuide, never CodeGeneration!
    assert_eq!(classify_user_intent("can we discuss whether to make a microservice or monolith?"), UserIntent::DiscussionAndGuide);
    assert_eq!(classify_user_intent("what do you think about Rust vs Go?"), UserIntent::DiscussionAndGuide);
    assert_eq!(classify_user_intent("let's talk about our team design process"), UserIntent::DiscussionAndGuide);
    assert_eq!(classify_user_intent("tell me about quantum computing"), UserIntent::DiscussionAndGuide);

    // Action-oriented "how to" queries MUST route to BugFix or CodeGeneration, NOT CodeExplanation!
    assert_eq!(classify_user_intent("how to fix compilation error in main.rs"), UserIntent::BugFix);
    assert_eq!(classify_user_intent("how do i fix this panic?"), UserIntent::BugFix);
    assert_eq!(classify_user_intent("how to add a dark mode toggle"), UserIntent::CodeGeneration);
    assert_eq!(classify_user_intent("how to implement JWT authentication"), UserIntent::CodeGeneration);

    // Explicit action command overrides conversational prefix!
    assert_eq!(classify_user_intent("can we discuss how to fix this bug? fix it now"), UserIntent::BugFix);

    // Pure explanations remain CodeExplanation
    assert_eq!(classify_user_intent("how does the project index cache work?"), UserIntent::CodeExplanation);
    assert_eq!(classify_user_intent("how to understand rust lifetimes?"), UserIntent::CodeExplanation);

    // Greetings and non-code chit-chat
    assert_eq!(classify_user_intent("hello, how are you?"), UserIntent::GeneralChat);
    assert_eq!(classify_user_intent("hi Zene"), UserIntent::GeneralChat);
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

#[test]
fn test_smart_file_classification_and_llm_fallback() {
    use ai::tools::environment::{DetectorAgent, execute_detect_environment};
    use ai::tools::read::execute_read_file;

    let tmp = std::env::temp_dir().join("zenthree_file_detect_test");
    let _ = std::fs::create_dir_all(&tmp);

    // 1. Extensionless shebang script
    let script_path = tmp.join("deploy_service");
    let _ = std::fs::write(&script_path, "#!/bin/bash\necho 'Deploying'\n");
    let res = execute_detect_environment(&tmp, Some("deploy_service"));
    assert!(res.ok);
    assert!(res.output.contains("Shell Script"));
    assert!(res.output.contains("Shebang"));

    // 2. Extensionless Python shebang script
    let py_path = tmp.join("worker");
    let _ = std::fs::write(&py_path, "#!/usr/bin/env python3\nimport sys\nprint('running')\n");
    let (name, method, _) = DetectorAgent::classify_file(&py_path);
    assert_eq!(name, "Python Script");
    assert_eq!(method, "Shebang");

    // 3. Extensionless Dockerfile
    let docker_path = tmp.join("Dockerfile");
    let _ = std::fs::write(&docker_path, "FROM alpine\nCMD [\"sh\"]\n");
    let (name, method, _) = DetectorAgent::classify_file(&docker_path);
    assert_eq!(name, "Dockerfile");
    assert_eq!(method, "Filename");

    // 4. Extensionless file with syntax markers (Rust)
    let rust_frag = tmp.join("engine_core");
    let _ = std::fs::write(&rust_frag, "pub fn execute_work() {\n    let mut x = 1;\n}\n");
    let (name, method, _) = DetectorAgent::classify_file(&rust_frag);
    assert_eq!(name, "Rust");
    assert!(method.contains("Syntax"));

    // 5. Completely unrecognized custom DSL file -> triggers LLM excerpt fallback
    let unknown_file = tmp.join("agent_policy.weird");
    let _ = std::fs::write(&unknown_file, "AGENT_RULE 99 {\n    PRIORITY: HIGH\n    ACTION: DELIBERATE\n}\n");
    let res_unknown = execute_detect_environment(&tmp, Some("agent_policy.weird"));
    assert!(res_unknown.ok);
    assert!(res_unknown.output.contains("Unrecognized"));
    assert!(res_unknown.output.contains("LLM Inspection Excerpt"));
    assert!(res_unknown.output.contains("AGENT_RULE 99"));

    // 6. read_file output includes type annotation in header
    let read_res = execute_read_file(&tmp, "deploy_service", None, None);
    assert!(read_res.ok);
    assert!(read_res.output.contains("[Type: Shell Script (via Shebang)]"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_ground_workspace_path_anti_root_dumping_and_resolution() {
    let tmp = std::env::temp_dir().join(format!("zenthree_path_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let nested_dir = tmp.join("crates").join("app").join("src");
    let _ = std::fs::create_dir_all(&nested_dir);

    // Create existing nested source file
    let real_controller = nested_dir.join("controller.rs");
    let _ = std::fs::write(&real_controller, "pub fn handle() {}\n");

    // 1. Exact filename resolution: bare filename "controller.rs" finds real nested path!
    let found = ground_workspace_path_with_context(&tmp, "controller.rs", None);
    assert_eq!(found, real_controller);

    // 2. Suffix match: "src/controller.rs" finds "crates/app/src/controller.rs"!
    let suffix_found = ground_workspace_path_with_context(&tmp, "src/controller.rs", None);
    assert_eq!(suffix_found, real_controller);

    // 3. Sibling placement: active file is in crates/app/src/controller.rs, creating new "service.rs"
    let new_sibling = ground_workspace_path_with_context(&tmp, "service.rs", Some(&real_controller));
    assert_eq!(new_sibling, nested_dir.join("service.rs"));

    // 4. Anti-root-dumping when src/ exists and no active file
    let src_dir = tmp.join("src");
    let _ = std::fs::create_dir_all(&src_dir);
    let new_component = ground_workspace_path_with_context(&tmp, "Button.tsx", None);
    assert_eq!(new_component, src_dir.join("Button.tsx"));

    // 5. Root configuration files remain on workspace root
    let readme = ground_workspace_path_with_context(&tmp, "README.md", None);
    assert_eq!(readme, tmp.join("README.md"));
    let dockerfile = ground_workspace_path_with_context(&tmp, "Dockerfile", None);
    assert_eq!(dockerfile, tmp.join("Dockerfile"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_verify_or_relocate_target_prevents_wrong_file_modification() {
    let tmp = std::env::temp_dir().join(format!("zenthree_wrong_file_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let mod_a_dir = tmp.join("crates").join("connectome").join("src");
    let mod_b_dir = tmp.join("crates").join("ai").join("src").join("tools");
    let _ = std::fs::create_dir_all(&mod_a_dir);
    let _ = std::fs::create_dir_all(&mod_b_dir);

    // Two files both named `mod.rs` in different modules
    let mod_a = mod_a_dir.join("mod.rs");
    let mod_b = mod_b_dir.join("mod.rs");
    let _ = std::fs::write(&mod_a, "pub fn connectome_init() -> bool {\n    true\n}\n");
    let _ = std::fs::write(&mod_b, "pub fn tools_init() -> bool {\n    true\n}\n");

    // The AI outputs a patch targeting ambiguous "mod.rs", but the SEARCH block belongs to `tools/mod.rs`
    let response = "### File: mod.rs\n```rust\n<<<<<<< SEARCH\npub fn tools_init() -> bool {\n    true\n}\n=======\npub fn tools_init() -> bool {\n    false\n}\n>>>>>>> REPLACE\n```";

    // Call extract_all_file_modifications
    let mods = extract_all_file_modifications(response, &tmp, None);
    assert_eq!(mods.len(), 1);

    // Verify it automatically targeted mod_b (the correct file) and NOT mod_a!
    assert_eq!(mods[0].file_path, mod_b);

    // Apply the modification
    let original_b = std::fs::read_to_string(&mod_b).unwrap();
    let new_b = mods[0].apply_to(&original_b).unwrap();
    assert!(new_b.contains("false"));

    // Verify mod_a was NEVER touched or corrupted
    let content_a = std::fs::read_to_string(&mod_a).unwrap();
    assert!(content_a.contains("connectome_init"));
    assert!(content_a.contains("true"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_github_workflows_and_docker_configuration_search_read_edit() {
    let tmp = std::env::temp_dir().join(format!("zenthree_config_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let github_workflows = tmp.join(".github").join("workflows");
    let _ = std::fs::create_dir_all(&github_workflows);

    // 1. Create .github/workflows/ci.yml
    let ci_yml = github_workflows.join("ci.yml");
    let _ = std::fs::write(&ci_yml, "name: CI\non: [push]\njobs:\n  build:\n    runs-on: ubuntu-latest\n    steps:\n      - uses: actions/checkout@v4\n");

    // 2. Create Dockerfile and .dockerignore
    let dockerfile = tmp.join("Dockerfile");
    let _ = std::fs::write(&dockerfile, "FROM rust:1.80-alpine\nWORKDIR /app\nCOPY . .\nRUN cargo build --release\nCMD [\"./target/release/app\"]\n");
    let dockerignore = tmp.join(".dockerignore");
    let _ = std::fs::write(&dockerignore, "target/\n.git/\n.env\n");

    // 3. Create .env
    let env_file = tmp.join(".env");
    let _ = std::fs::write(&env_file, "DATABASE_URL=postgres://localhost/mydb\nPORT=8080\n");

    // TEST SEARCH_CODE: Can search inside .github/workflows/ci.yml, Dockerfile, and .env!
    let search_ci = execute_search_code(&tmp, "actions/checkout");
    assert!(search_ci.ok);
    assert!(search_ci.output.contains(".github/workflows/ci.yml"), "Search should find checkout in ci.yml: {}", search_ci.output);

    let search_docker = execute_search_code(&tmp, "cargo build --release");
    assert!(search_docker.ok);
    assert!(search_docker.output.contains("Dockerfile"), "Search should find Dockerfile: {}", search_docker.output);

    let search_env = execute_search_code(&tmp, "DATABASE_URL");
    assert!(search_env.ok);
    assert!(search_env.output.contains(".env"), "Search should find .env: {}", search_env.output);

    // TEST LIST_DIRECTORY: Correctly displays .github/ and Dockerfile
    let list_root = execute_list_directory(&tmp, None);
    assert!(list_root.ok);
    assert!(list_root.output.contains(".github/"));
    assert!(list_root.output.contains("Dockerfile"));
    assert!(list_root.output.contains(".dockerignore"));

    let list_github = execute_list_directory(&tmp, Some(".github"));
    assert!(list_github.ok);
    assert!(list_github.output.contains("workflows/"));

    // TEST READ_FILE: Can read .github/workflows/ci.yml directly and via bare filename "ci.yml"
    let read_direct = execute_read_file(&tmp, ".github/workflows/ci.yml", None, None);
    assert!(read_direct.ok);
    assert!(read_direct.output.contains("runs-on: ubuntu-latest"));

    let read_bare = execute_read_file(&tmp, "ci.yml", None, None);
    assert!(read_bare.ok);
    assert!(read_bare.output.contains("runs-on: ubuntu-latest"));

    let read_docker = execute_read_file(&tmp, "Dockerfile", None, None);
    assert!(read_docker.ok);
    assert!(read_docker.output.contains("FROM rust:1.80-alpine"));

    // TEST SURGICAL EDIT ON .github/workflows/ci.yml
    let patch_ci = "### File: .github/workflows/ci.yml\n```yaml\n<<<<<<< SEARCH\n      - uses: actions/checkout@v4\n=======\n      - uses: actions/checkout@v4\n      - name: Run Tests\n        run: cargo test\n>>>>>>> REPLACE\n```";
    let mods = extract_all_file_modifications(patch_ci, &tmp, None);
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].file_path, ci_yml);
    let original_ci = std::fs::read_to_string(&ci_yml).unwrap();
    let updated_ci = mods[0].apply_to(&original_ci).unwrap();
    assert!(updated_ci.contains("cargo test"));

    // TEST SURGICAL EDIT ON Dockerfile
    let patch_docker = "### File: Dockerfile\n```dockerfile\n<<<<<<< SEARCH\nFROM rust:1.80-alpine\n=======\nFROM rust:1.85-alpine\n>>>>>>> REPLACE\n```";
    let mods_docker = extract_all_file_modifications(patch_docker, &tmp, None);
    assert_eq!(mods_docker.len(), 1);
    assert_eq!(mods_docker[0].file_path, dockerfile);
    let original_df = std::fs::read_to_string(&dockerfile).unwrap();
    let updated_df = mods_docker[0].apply_to(&original_df).unwrap();
    assert!(updated_df.contains("FROM rust:1.85-alpine"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_find_nearest_manifest_dir_and_cwd_resolution() {
    use ai::tools::command::{find_nearest_manifest_dir, resolve_command_cwd, parse_cd_statement};
    use std::fs;

    let tmp = std::env::temp_dir().join("zenthree_test_subproject_manifest");
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).unwrap();

    let subproject = tmp.join("react-calculator");
    let src_dir = subproject.join("src");
    fs::create_dir_all(&src_dir).unwrap();
    fs::write(subproject.join("package.json"), r#"{"name": "calculator"}"#).unwrap();

    let app_jsx = src_dir.join("App.jsx");
    fs::write(&app_jsx, "export default function App() {}").unwrap();

    // 1. find_nearest_manifest_dir from nested file walks upward to react-calculator
    let found = find_nearest_manifest_dir(&app_jsx, &tmp);
    assert_eq!(found, Some(subproject.clone()));

    // 2. resolve_command_cwd with single subproject heuristic:
    // Root directory has no manifest, but react-calculator has a package.json!
    let resolved_cwd = resolve_command_cwd(&tmp, None, None);
    assert_eq!(resolved_cwd, subproject);

    // 3. resolve_command_cwd with explicit cwd
    let explicit_cwd = resolve_command_cwd(&tmp, Some("react-calculator/src"), None);
    assert_eq!(explicit_cwd, src_dir);

    // 4. parse_cd_statement test cases
    assert_eq!(parse_cd_statement("cd react-calculator"), Some(("react-calculator".to_string(), None)));
    assert_eq!(
        parse_cd_statement("cd react-calculator && npm run dev"),
        Some(("react-calculator".to_string(), Some("npm run dev".to_string())))
    );
    assert_eq!(
        parse_cd_statement("cd 'frontend app'; bun test"),
        Some(("frontend app".to_string(), Some("bun test".to_string())))
    );
    assert_eq!(parse_cd_statement("cargo test"), None);

    let _ = fs::remove_dir_all(&tmp);
}

#[test]
fn test_compact_conversation_history() {
    use ai::orchestrator::loop_engine::compact_conversation_history;
    use ai::types::ChatMessage;

    let mut history = vec![
        ChatMessage {
            role: "user".to_string(),
            content: "Create a full stack calculator with React and Vite".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "I will initialize the project using Vite.\n{\"tool\": \"run_command\", \"cmd\": \"npm create vite@latest react-calc\"}".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Observation Feedback:\n- Tool `run_command` (Status: SUCCESS)".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Now installing dependencies.\n{\"tool\": \"run_command\", \"cmd\": \"cd react-calc && npm install\"}".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Observation Feedback:\n### File Updates Applied to Disk:\n- Wrote `react-calc/src/App.jsx` (+50 / -10 lines)".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Writing calculator UI\n### File: react-calc/src/App.jsx\n```jsx\n// code\n```".to_string(),
        },
        ChatMessage {
            role: "user".to_string(),
            content: "Observation Feedback:\n- Wrote `react-calc/src/App.jsx` (+50 / -10 lines)".to_string(),
        },
        ChatMessage {
            role: "assistant".to_string(),
            content: "Now verifying application builds\n{\"tool\": \"run_command\", \"cmd\": \"npm run build\"}".to_string(),
        },
    ];

    assert_eq!(history.len(), 8);
    compact_conversation_history(&mut history);

    // Compacting preserves turn 1 (2 messages), adds 1 compacted summary turn (2 messages),
    // and preserves the 2 most recent messages (4 messages total from slice to_compact_end..)
    // Total should be compacted down
    assert!(history.len() < 8);
    assert_eq!(history[0].content, "Create a full stack calculator with React and Vite");
    assert!(history[3].content.contains("Previous Steps Summary"));
    assert!(history.last().unwrap().content.contains("npm run build"));
}


