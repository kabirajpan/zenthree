use super::{BenchmarkCategory, BenchmarkDifficulty, BenchmarkTask, ValidationRule};

pub fn get_standard_benchmark_suite() -> Vec<BenchmarkTask> {
    vec![
        // 1. BugFix: Off-by-one calculation in Rust algorithm
        BenchmarkTask {
            id: "bench_bugfix_off_by_one".to_string(),
            name: "Rust Binary Search Off-by-One Fix".to_string(),
            category: BenchmarkCategory::BugFix,
            difficulty: BenchmarkDifficulty::Easy,
            prompt: "Fix the off-by-one bug in binary_search where the upper bound calculation causes an out-of-bounds error".to_string(),
            initial_files: vec![
                (
                    "src/search.rs".to_string(),
                    r#"pub fn binary_search(arr: &[i32], target: i32) -> Option<usize> {
    let mut low = 0;
    let mut high = arr.len(); // BUG: should be arr.len().checked_sub(1) or handled with bounds
    while low <= high {
        let mid = (low + high) / 2;
        if arr[mid] == target {
            return Some(mid);
        } else if arr[mid] < target {
            low = mid + 1;
        } else {
            if mid == 0 { break; }
            high = mid - 1;
        }
    }
    None
}"#.to_string(),
                ),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("src/search.rs".to_string()),
                ValidationRule::ContentContains {
                    path: "src/search.rs".to_string(),
                    expected: "pub fn binary_search".to_string(),
                },
                ValidationRule::ContentNotContains {
                    path: "src/search.rs".to_string(),
                    unexpected: "let mut high = arr.len(); // BUG".to_string(),
                },
            ],
        },

        // 2. BugFix: Null dereference in TypeScript service
        BenchmarkTask {
            id: "bench_bugfix_null_deref".to_string(),
            name: "TypeScript Optional Chaining & Null Guard".to_string(),
            category: BenchmarkCategory::BugFix,
            difficulty: BenchmarkDifficulty::Easy,
            prompt: "Prevent null pointer crash when user profile or avatar is missing in formatUserDisplay".to_string(),
            initial_files: vec![
                (
                    "src/user.ts".to_string(),
                    r#"export interface User { id: string; profile?: { avatar?: string; displayName?: string }; }
export function formatUserDisplay(user: User): string {
    return user.profile.displayName.toUpperCase(); // BUG: crashes if profile is undefined
}"#.to_string(),
                ),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("src/user.ts".to_string()),
                ValidationRule::ContentContains {
                    path: "src/user.ts".to_string(),
                    expected: "user.profile?.displayName".to_string(),
                },
            ],
        },

        // 3. CodeGeneration: React Calculator Component
        BenchmarkTask {
            id: "bench_codegen_react_calculator".to_string(),
            name: "React Calculator Full Implementation".to_string(),
            category: BenchmarkCategory::CodeGeneration,
            difficulty: BenchmarkDifficulty::Medium,
            prompt: "Build me a sleek react calculator with display and keypad".to_string(),
            initial_files: vec![
                ("package.json".to_string(), r#"{"name": "react-calc", "dependencies": {"react": "^18.2.0"}}"#.to_string()),
                ("src/App.jsx".to_string(), "// Starter App".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("src/App.jsx".to_string()),
                ValidationRule::FileExists("src/components/Display.jsx".to_string()),
                ValidationRule::FileExists("src/components/Keypad.jsx".to_string()),
                ValidationRule::ContentContains {
                    path: "src/components/Display.jsx".to_string(),
                    expected: "export default function Display".to_string(),
                },
                ValidationRule::ContentContains {
                    path: "src/components/Keypad.jsx".to_string(),
                    expected: "export default function Keypad".to_string(),
                },
            ],
        },

        // 4. CodeGeneration: Rust System Monitor CLI
        BenchmarkTask {
            id: "bench_codegen_rust_sysmon".to_string(),
            name: "Rust System Monitor CLI".to_string(),
            category: BenchmarkCategory::CodeGeneration,
            difficulty: BenchmarkDifficulty::Medium,
            prompt: "Create a rust cli tool that monitors system memory and cpu with clap".to_string(),
            initial_files: vec![
                ("Cargo.toml".to_string(), "[package]\nname = \"sysmon\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[dependencies]\nclap = \"4\"".to_string()),
                ("src/main.rs".to_string(), "fn main() {}".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("src/main.rs".to_string()),
                ValidationRule::FileExists("src/monitor.rs".to_string()),
                ValidationRule::ContentContains {
                    path: "src/monitor.rs".to_string(),
                    expected: "pub struct SystemStats".to_string(),
                },
            ],
        },

        // 5. CodeGeneration: Python FastAPI Microservice
        BenchmarkTask {
            id: "bench_codegen_python_fastapi".to_string(),
            name: "Python FastAPI REST Service".to_string(),
            category: BenchmarkCategory::CodeGeneration,
            difficulty: BenchmarkDifficulty::Medium,
            prompt: "Build a python fastapi backend for a todo app with pydantic schemas".to_string(),
            initial_files: vec![
                ("pyproject.toml".to_string(), "[project]\nname = \"todo-api\"\nversion = \"0.1.0\"".to_string()),
                ("app/main.py".to_string(), "from fastapi import FastAPI\napp = FastAPI()".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("app/main.py".to_string()),
                ValidationRule::FileExists("app/models.py".to_string()),
                ValidationRule::FileExists("app/routes.py".to_string()),
                ValidationRule::ContentContains {
                    path: "app/models.py".to_string(),
                    expected: "class TodoItem(BaseModel):".to_string(),
                },
            ],
        },

        // 6. Refactoring: Monolith Module Extraction
        BenchmarkTask {
            id: "bench_refactor_monolith".to_string(),
            name: "Decompose Monolithic Handler into Clean Submodules".to_string(),
            category: BenchmarkCategory::Refactoring,
            difficulty: BenchmarkDifficulty::Medium,
            prompt: "Refactor this monolithic server into separate routes, auth, and database modules".to_string(),
            initial_files: vec![
                (
                    "server.js".to_string(),
                    r#"const express = require('express');
const app = express();
// Database logic
function queryUser(id) { return { id, name: 'Alice' }; }
// Auth logic
function authenticate(token) { return token === 'secret'; }
// Route handlers
app.get('/user', (req, res) => { res.json(queryUser(1)); });
app.listen(3000);
"#.to_string(),
                ),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("server.js".to_string()),
                ValidationRule::FileExists("db.js".to_string()),
                ValidationRule::FileExists("auth.js".to_string()),
                ValidationRule::ContentContains {
                    path: "db.js".to_string(),
                    expected: "module.exports = { queryUser };".to_string(),
                },
                ValidationRule::ContentContains {
                    path: "auth.js".to_string(),
                    expected: "module.exports = { authenticate };".to_string(),
                },
            ],
        },

        // 7. Scaffolding & Anti-Root Dumping
        BenchmarkTask {
            id: "bench_scaffold_anti_root_dumping".to_string(),
            name: "Subproject Scaffolding & Zero Root Pollution".to_string(),
            category: BenchmarkCategory::BoundaryAndRouting,
            difficulty: BenchmarkDifficulty::Hard,
            prompt: "Scaffold a modern react application in 'frontend' and create components without polluting workspace root".to_string(),
            initial_files: vec![
                ("frontend/package.json".to_string(), r#"{"name": "frontend"}"#.to_string()),
                ("frontend/src/App.jsx".to_string(), "export default function App() {}".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("frontend/src/App.jsx".to_string()),
                ValidationRule::FileExists("frontend/src/components/Card.jsx".to_string()),
                ValidationRule::NoRootDumping {
                    forbidden_root_dirs: vec!["src".to_string(), "components".to_string()],
                },
            ],
        },

        // 8. Boundary & Routing: SvelteKit Nested Route Isolation
        BenchmarkTask {
            id: "bench_route_isolation_sveltekit".to_string(),
            name: "SvelteKit Nested Route File Collision Defense".to_string(),
            category: BenchmarkCategory::BoundaryAndRouting,
            difficulty: BenchmarkDifficulty::Hard,
            prompt: "Create a blog post page in sveltekit at src/routes/posts/+page.svelte without overwriting root +page.svelte".to_string(),
            initial_files: vec![
                ("personal-blog/package.json".to_string(), r#"{"name": "personal-blog"}"#.to_string()),
                ("personal-blog/src/routes/+page.svelte".to_string(), "<h1>Home Page</h1>".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("personal-blog/src/routes/+page.svelte".to_string()),
                ValidationRule::FileExists("personal-blog/src/routes/posts/+page.svelte".to_string()),
                ValidationRule::ContentContains {
                    path: "personal-blog/src/routes/+page.svelte".to_string(),
                    expected: "<h1>Home Page</h1>".to_string(),
                },
                ValidationRule::ContentContains {
                    path: "personal-blog/src/routes/posts/+page.svelte".to_string(),
                    expected: "<h1>Blog Post</h1>".to_string(),
                },
            ],
        },

        // 9. Boundary & Routing: Next.js App Router Page Isolation
        BenchmarkTask {
            id: "bench_route_isolation_nextjs".to_string(),
            name: "Next.js App Router Nested Page Isolation".to_string(),
            category: BenchmarkCategory::BoundaryAndRouting,
            difficulty: BenchmarkDifficulty::Hard,
            prompt: "Create a dashboard page at app/dashboard/page.tsx without overwriting the landing page at app/page.tsx".to_string(),
            initial_files: vec![
                ("next-app/package.json".to_string(), r#"{"name": "next-app"}"#.to_string()),
                ("next-app/app/page.tsx".to_string(), "export default function Home() { return <h1>Landing</h1>; }".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("next-app/app/page.tsx".to_string()),
                ValidationRule::FileExists("next-app/app/dashboard/page.tsx".to_string()),
                ValidationRule::ContentContains {
                    path: "next-app/app/page.tsx".to_string(),
                    expected: "<h1>Landing</h1>".to_string(),
                },
                ValidationRule::ContentContains {
                    path: "next-app/app/dashboard/page.tsx".to_string(),
                    expected: "<h1>Dashboard</h1>".to_string(),
                },
            ],
        },

        // 10. Self-Healing: Compiler Error Repair Loop
        BenchmarkTask {
            id: "bench_self_healing_compile_error".to_string(),
            name: "Automated Compiler Self-Healing Repair".to_string(),
            category: BenchmarkCategory::SelfHealing,
            difficulty: BenchmarkDifficulty::Medium,
            prompt: "Fix the type mismatch in compute_metrics to repair compiler error".to_string(),
            initial_files: vec![
                (
                    "src/metrics.rs".to_string(),
                    r#"pub fn compute_metrics(values: &[f64]) -> f64 {
    let sum: f64 = values.iter().sum();
    sum / values.len() // TYPE ERROR: cannot divide f64 by usize
}"#.to_string(),
                ),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("src/metrics.rs".to_string()),
                ValidationRule::ContentContains {
                    path: "src/metrics.rs".to_string(),
                    expected: "sum / values.len() as f64".to_string(),
                },
            ],
        },

        // 11. Context Compaction & Token Preservation
        BenchmarkTask {
            id: "bench_compaction_token_efficiency".to_string(),
            name: "Rolling Context History Compaction Efficiency".to_string(),
            category: BenchmarkCategory::Refactoring,
            difficulty: BenchmarkDifficulty::Easy,
            prompt: "Verify that long conversation turns compact into bullet summaries without losing turn 1".to_string(),
            initial_files: vec![
                ("README.md".to_string(), "# Project".to_string()),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("README.md".to_string()),
            ],
        },

        // 12. Surgical SEARCH/REPLACE CRLF Line Ending Precision
        BenchmarkTask {
            id: "bench_search_replace_crlf_precision".to_string(),
            name: "CRLF & Multi-Line Indentation Patch Precision".to_string(),
            category: BenchmarkCategory::BugFix,
            difficulty: BenchmarkDifficulty::Easy,
            prompt: "Apply a search and replace patch cleanly across CRLF Windows line endings".to_string(),
            initial_files: vec![
                (
                    "src/config.ini".to_string(),
                    "[settings]\r\ntimeout = 30\r\nretries = 3\r\n".to_string(),
                ),
            ],
            validation_rules: vec![
                ValidationRule::FileExists("src/config.ini".to_string()),
                ValidationRule::ContentContains {
                    path: "src/config.ini".to_string(),
                    expected: "timeout = 60".to_string(),
                },
            ],
        },
    ]
}
