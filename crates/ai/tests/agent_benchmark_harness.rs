use std::fs;
use std::path::Path;
use ai::benchmark::{get_standard_benchmark_suite, BenchmarkRunner, BenchmarkResult};
use ai::tools::patch::apply_search_replace;
use ai::tools::parser::{
    extract_all_file_modifications, ground_workspace_path_with_context, parse_search_replace_blocks,
};
use ai::tools::command::resolve_command_cwd;
use ai::types::ChatMessage;
use ai::orchestrator::loop_engine::compact_conversation_history;

#[test]
fn test_agent_benchmark_harness_suite() {
    let tasks = get_standard_benchmark_suite();
    let runner = BenchmarkRunner::new(tasks.clone());
    let mut results: Vec<BenchmarkResult> = Vec::new();

    for task in &tasks {
        let task_clone = task.clone();
        let res = runner.run_task(&task_clone, |ws: &Path| {
            match task_clone.id.as_str() {
                // 1. BugFix: Off-by-one
                "bench_bugfix_off_by_one" => {
                    let search_file = ws.join("src/search.rs");
                    let content = fs::read_to_string(&search_file).map_err(|e| e.to_string())?;
                    let patch = r#"
<<<<<<< SEARCH
    let mut high = arr.len(); // BUG: should be arr.len().checked_sub(1) or handled with bounds
    while low <= high {
=======
    if arr.is_empty() { return None; }
    let mut high = arr.len() - 1;
    while low <= high {
>>>>>>> REPLACE
"#;
                    let blocks = parse_search_replace_blocks(patch);
                    let patched = apply_search_replace(&content, &blocks).map_err(|e| e.to_string())?;
                    fs::write(&search_file, patched).map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 2. BugFix: Null deref
                "bench_bugfix_null_deref" => {
                    let user_file = ws.join("src/user.ts");
                    let content = fs::read_to_string(&user_file).map_err(|e| e.to_string())?;
                    let patch = r#"
<<<<<<< SEARCH
export function formatUserDisplay(user: User): string {
    return user.profile.displayName.toUpperCase(); // BUG: crashes if profile is undefined
}
=======
export function formatUserDisplay(user: User): string {
    return user.profile?.displayName ? user.profile.displayName.toUpperCase() : "ANONYMOUS";
}
>>>>>>> REPLACE
"#;
                    let blocks = parse_search_replace_blocks(patch);
                    let patched = apply_search_replace(&content, &blocks).map_err(|e| e.to_string())?;
                    fs::write(&user_file, patched).map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 3. CodeGen: React Calculator
                "bench_codegen_react_calculator" => {
                    let agent_output = r#"
### File: src/components/Display.jsx
```jsx
export default function Display({ value }) {
    return <div className="calc-display">{value}</div>;
}
```

### File: src/components/Keypad.jsx
```jsx
export default function Keypad({ onInput }) {
    return <div className="calc-keypad"><button onClick={() => onInput('7')}>7</button></div>;
}
```
"#;
                    let mods = extract_all_file_modifications(agent_output, ws, None);
                    for m in mods {
                        if let Some(parent) = m.file_path.parent() {
                            let _ = fs::create_dir_all(parent);
                        }
                        let code = m.apply_to("").map_err(|e| e.to_string())?;
                        fs::write(&m.file_path, code).map_err(|e| e.to_string())?;
                    }
                    Ok(())
                }

                // 4. CodeGen: Rust Sysmon CLI
                "bench_codegen_rust_sysmon" => {
                    let monitor_code = r#"
#[derive(Debug, Clone)]
pub struct SystemStats {
    pub cpu_usage_pct: f32,
    pub memory_used_mb: u64,
}
impl SystemStats {
    pub fn collect() -> Self {
        Self { cpu_usage_pct: 12.5, memory_used_mb: 4096 }
    }
}
"#;
                    let grounded = ground_workspace_path_with_context(ws, "src/monitor.rs", None);
                    fs::write(grounded, monitor_code).map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 5. CodeGen: Python FastAPI
                "bench_codegen_python_fastapi" => {
                    let models_code = "from pydantic import BaseModel\nclass TodoItem(BaseModel):\n    id: int\n    title: str\n";
                    let routes_code = "from fastapi import APIRouter\nrouter = APIRouter()\n";
                    let grounded_models = ground_workspace_path_with_context(ws, "app/models.py", None);
                    let grounded_routes = ground_workspace_path_with_context(ws, "app/routes.py", None);
                    fs::write(grounded_models, models_code).map_err(|e| e.to_string())?;
                    fs::write(grounded_routes, routes_code).map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 6. Refactor: Monolith Extraction
                "bench_refactor_monolith" => {
                    fs::write(ws.join("db.js"), "function queryUser(id) { return { id, name: 'Alice' }; }\nmodule.exports = { queryUser };\n").map_err(|e| e.to_string())?;
                    fs::write(ws.join("auth.js"), "function authenticate(token) { return token === 'secret'; }\nmodule.exports = { authenticate };\n").map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 7. Scaffolding & Anti-Root Dumping
                "bench_scaffold_anti_root_dumping" => {
                    let grounded = ground_workspace_path_with_context(ws, "src/components/Card.jsx", None);
                    if let Some(parent) = grounded.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    fs::write(&grounded, "export default function Card() {}").map_err(|e| e.to_string())?;
                    // Verify CWD resolution also binds to subproject
                    let resolved_cwd = resolve_command_cwd(ws, None, None);
                    assert_eq!(resolved_cwd, ws.join("frontend"));
                    Ok(())
                }

                // 8. Route Isolation SvelteKit
                "bench_route_isolation_sveltekit" => {
                    let grounded_post = ground_workspace_path_with_context(ws, "src/routes/posts/+page.svelte", None);
                    if let Some(parent) = grounded_post.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    fs::write(&grounded_post, "<h1>Blog Post</h1>").map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 9. Route Isolation Next.js
                "bench_route_isolation_nextjs" => {
                    let grounded_dash = ground_workspace_path_with_context(ws, "app/dashboard/page.tsx", None);
                    if let Some(parent) = grounded_dash.parent() {
                        let _ = fs::create_dir_all(parent);
                    }
                    fs::write(&grounded_dash, "export default function Dashboard() { return <h1>Dashboard</h1>; }").map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 10. Self-Healing Compile Error Repair
                "bench_self_healing_compile_error" => {
                    let metrics_file = ws.join("src/metrics.rs");
                    let content = fs::read_to_string(&metrics_file).map_err(|e| e.to_string())?;
                    let patch = r#"
<<<<<<< SEARCH
    sum / values.len() // TYPE ERROR: cannot divide f64 by usize
=======
    if values.is_empty() { 0.0 } else { sum / values.len() as f64 }
>>>>>>> REPLACE
"#;
                    let blocks = parse_search_replace_blocks(patch);
                    let patched = apply_search_replace(&content, &blocks).map_err(|e| e.to_string())?;
                    fs::write(&metrics_file, patched).map_err(|e| e.to_string())?;
                    Ok(())
                }

                // 11. Compaction Efficiency
                "bench_compaction_token_efficiency" => {
                    let mut history = vec![
                        ChatMessage { role: "user".to_string(), content: "Turn 1: Build the app".to_string() },
                        ChatMessage { role: "assistant".to_string(), content: "Turn 1 reply".to_string() },
                        ChatMessage { role: "user".to_string(), content: "Turn 2: Run npm install".to_string() },
                        ChatMessage { role: "assistant".to_string(), content: "Turn 2 reply with verbose logs".to_string() },
                        ChatMessage { role: "user".to_string(), content: "Turn 3: Run npm test".to_string() },
                        ChatMessage { role: "assistant".to_string(), content: "Turn 3 reply".to_string() },
                        ChatMessage { role: "user".to_string(), content: "Turn 4: Add feature".to_string() },
                        ChatMessage { role: "assistant".to_string(), content: "Turn 4 reply".to_string() },
                    ];
                    compact_conversation_history(&mut history);
                    assert_eq!(history.len(), 6, "Compaction must bound conversation size to 6 messages");
                    assert!(history[0].content.contains("Turn 1"), "Turn 1 user request must be preserved");
                    Ok(())
                }

                // 12. Surgical SEARCH/REPLACE CRLF Precision
                "bench_search_replace_crlf_precision" => {
                    let config_file = ws.join("src/config.ini");
                    let content = fs::read_to_string(&config_file).map_err(|e| e.to_string())?;
                    let patch = "<<<<<<< SEARCH\r\ntimeout = 30\r\n=======\r\ntimeout = 60\r\n>>>>>>> REPLACE";
                    let blocks = parse_search_replace_blocks(patch);
                    let patched = apply_search_replace(&content, &blocks).map_err(|e| e.to_string())?;
                    fs::write(&config_file, patched).map_err(|e| e.to_string())?;
                    Ok(())
                }

                _ => Err(format!("Unrecognized task id: {}", task_clone.id)),
            }
        });

        results.push(res);
    }

    let report = runner.generate_report(results);
    let markdown_summary = report.to_markdown_summary();
    let json_export = report.to_json_export().unwrap();

    // Print the visual scorecard to test output
    println!("\n{}\n", markdown_summary);

    // Save JSON benchmark export to /tmp/zenthree_agent_benchmark.json
    let export_path = std::env::temp_dir().join("zenthree_agent_benchmark.json");
    let _ = fs::write(&export_path, &json_export);
    println!("Exported benchmark JSON report to: {}", export_path.display());

    // Assert overall 100% pass rate
    assert_eq!(report.passed_tasks, report.total_tasks, "All benchmark tasks must pass!");
    assert_eq!(report.pass_rate, 100.0, "Benchmark pass rate must be 100%!");
}
