use std::fs;
use std::path::PathBuf;
use ai::types::{classify_user_intent, UserIntent};
use ai::orchestrator::factory::create_mission_for_prompt;
use ai::tools::command::{
    find_nearest_manifest_dir, resolve_command_cwd, is_scaffolding_command,
};
use ai::tools::parser::{ground_workspace_path_with_context, extract_all_file_modifications};

fn create_test_workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zenthree_human_test_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// 1. React Pomodoro App from informal human prompt
#[test]
fn test_human_prompt_1_react_pomodoro() {
    let prompt = "hey, can you build me a sleek react pomodoro timer app with tailwind and history?";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let mission = create_mission_for_prompt(prompt, &[]);
    assert!(mission.tasks[2].title.contains("Scaffold") || mission.tasks[2].title.contains("implement"));

    let ws = create_test_workspace("react_pomodoro");
    let proj = ws.join("pomodoro-app");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "pomodoro-app"}"#).unwrap();
    fs::write(proj.join("src/App.jsx"), "export default function App() {}").unwrap();

    let scaffold_cmd = "npm create vite@latest pomodoro-app -- --template react";
    assert!(is_scaffolding_command(scaffold_cmd));

    // Working directory must automatically resolve to pomodoro-app
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    // Grounding newly generated files must stay inside pomodoro-app/
    let timer_path = ground_workspace_path_with_context(&ws, "src/components/Timer.jsx", None);
    assert_eq!(timer_path, proj.join("src/components/Timer.jsx"));
    assert!(!ws.join("src").exists(), "Must not pollute workspace root with src!");

    let _ = fs::remove_dir_all(&ws);
}

// 2. Rust CLI System Monitor from informal human prompt
#[test]
fn test_human_prompt_2_rust_syspulse() {
    let prompt = "please make me a rust cli tool called 'syspulse' that checks cpu and memory usage";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("rust_syspulse");
    let proj = ws.join("syspulse");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("Cargo.toml"), "[package]\nname = \"syspulse\"\nversion = \"0.1.0\"").unwrap();
    fs::write(proj.join("src/main.rs"), "fn main() {}").unwrap();

    let scaffold_cmd = "cargo new syspulse --bin";
    assert!(is_scaffolding_command(scaffold_cmd));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let monitor_file = ground_workspace_path_with_context(&ws, "src/monitor.rs", None);
    assert_eq!(monitor_file, proj.join("src/monitor.rs"));

    let _ = fs::remove_dir_all(&ws);
}

// 3. Python FastAPI Backend from conversational prompt
#[test]
fn test_human_prompt_3_python_fastapi() {
    let prompt = "i want to build a python fastapi backend for an ecommerce store with sqlite";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("python_fastapi");
    let proj = ws.join("ecommerce-api");
    fs::create_dir_all(proj.join("app")).unwrap();
    fs::write(proj.join("pyproject.toml"), "[project]\nname = \"ecommerce-api\"").unwrap();
    fs::write(proj.join("app/main.py"), "from fastapi import FastAPI").unwrap();

    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let routes_file = ground_workspace_path_with_context(&ws, "app/routes/products.py", None);
    assert_eq!(routes_file, proj.join("app/routes/products.py"));

    let _ = fs::remove_dir_all(&ws);
}

// 4. Vue 3 Kanban Board from human prompt
#[test]
fn test_human_prompt_4_vue_kanban() {
    let prompt = "can you build a vue 3 kanban board app with drag and drop columns?";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("vue_kanban");
    let proj = ws.join("kanban-board");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "kanban-board"}"#).unwrap();
    fs::write(proj.join("src/App.vue"), "<template><div>Kanban</div></template>").unwrap();

    let scaffold_cmd = "npm create vue@latest kanban-board";
    assert!(is_scaffolding_command(scaffold_cmd));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let col_file = ground_workspace_path_with_context(&ws, "src/components/Column.vue", None);
    assert_eq!(col_file, proj.join("src/components/Column.vue"));

    let _ = fs::remove_dir_all(&ws);
}

// 5. Go Microservice from human prompt
#[test]
fn test_human_prompt_5_go_microservice() {
    let prompt = "hey can you setup a go microservice with gin, health check, and a user crud api?";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("go_microservice");
    let proj = ws.join("user-service");
    fs::create_dir_all(proj.join("handlers")).unwrap();
    fs::write(proj.join("go.mod"), "module user-service\n\ngo 1.22").unwrap();
    fs::write(proj.join("main.go"), "package main\n\nfunc main() {}").unwrap();

    assert!(is_scaffolding_command("go mod init user-service"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let handler_file = ground_workspace_path_with_context(&ws, "handlers/user.go", None);
    assert_eq!(handler_file, proj.join("handlers/user.go"));

    let _ = fs::remove_dir_all(&ws);
}

// 6. SvelteKit Blog from human prompt
#[test]
fn test_human_prompt_6_sveltekit_blog() {
    let prompt = "build me a modern sveltekit personal blog with markdown posts";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("svelte_blog");
    let proj = ws.join("personal-blog");
    fs::create_dir_all(proj.join("src/routes")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "personal-blog"}"#).unwrap();
    fs::write(proj.join("src/routes/+page.svelte"), "<h1>Blog</h1>").unwrap();

    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let post_page = ground_workspace_path_with_context(&ws, "src/routes/posts/+page.svelte", None);
    assert_eq!(post_page, proj.join("src/routes/posts/+page.svelte"));

    let _ = fs::remove_dir_all(&ws);
}

// 7. Next.js SaaS Landing Page from human prompt
#[test]
fn test_human_prompt_7_nextjs_saas() {
    let prompt = "i need a nextjs 14 app for a SaaS landing page with pricing table and dark mode";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("nextjs_saas");
    let proj = ws.join("saas-landing");
    fs::create_dir_all(proj.join("app")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "saas-landing"}"#).unwrap();
    fs::write(proj.join("app/page.tsx"), "export default function Page() {}").unwrap();

    assert!(is_scaffolding_command("npx create-next-app@latest saas-landing"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let pricing = ground_workspace_path_with_context(&ws, "app/components/Pricing.tsx", None);
    assert_eq!(pricing, proj.join("app/components/Pricing.tsx"));

    let _ = fs::remove_dir_all(&ws);
}

// 8. Node Express REST API from human prompt
#[test]
fn test_human_prompt_8_express_notes() {
    let prompt = "make me a node express rest api for note taking with sqlite";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("express_notes");
    let proj = ws.join("notes-api");
    fs::create_dir_all(proj.join("routes")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "notes-api"}"#).unwrap();
    fs::write(proj.join("server.js"), "const express = require('express');").unwrap();

    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let routes = ground_workspace_path_with_context(&ws, "routes/notes.js", None);
    assert_eq!(routes, proj.join("routes/notes.js"));

    let _ = fs::remove_dir_all(&ws);
}

// 9. Zig Systems Utility from human prompt
#[test]
fn test_human_prompt_9_zig_counter() {
    let prompt = "set up a zig system utility to count lines of code in a directory";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("zig_counter");
    let proj = ws.join("loc-counter");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("build.zig"), "// zig build file").unwrap();
    fs::write(proj.join("src/main.zig"), "pub fn main() !void {}").unwrap();

    assert!(is_scaffolding_command("zig init"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let walker = ground_workspace_path_with_context(&ws, "src/walker.zig", None);
    assert_eq!(walker, proj.join("src/walker.zig"));

    let _ = fs::remove_dir_all(&ws);
}

// 10. Flutter Mobile App from human prompt
#[test]
fn test_human_prompt_10_flutter_habits() {
    let prompt = "can you create a flutter mobile app for habit tracking with daily streaks?";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("flutter_habits");
    let proj = ws.join("habit_tracker");
    fs::create_dir_all(proj.join("lib/screens")).unwrap();
    fs::write(proj.join("pubspec.yaml"), "name: habit_tracker").unwrap();
    fs::write(proj.join("lib/main.dart"), "void main() => runApp(MyApp());").unwrap();

    assert!(is_scaffolding_command("flutter create habit_tracker"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let screen = ground_workspace_path_with_context(&ws, "lib/screens/dashboard.dart", None);
    assert_eq!(screen, proj.join("lib/screens/dashboard.dart"));

    let _ = fs::remove_dir_all(&ws);
}

// 11. C++ Game Engine Loop from human prompt
#[test]
fn test_human_prompt_11_cpp_engine() {
    let prompt = "build me a c++ 2d game engine loop using cmake and sdl2";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("cpp_engine");
    let proj = ws.join("game-engine");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::create_dir_all(proj.join("include")).unwrap();
    fs::write(proj.join("CMakeLists.txt"), "cmake_minimum_required(VERSION 3.20)").unwrap();
    fs::write(proj.join("src/main.cpp"), "int main() { return 0; }").unwrap();

    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let header = ground_workspace_path_with_context(&ws, "include/engine.hpp", None);
    assert_eq!(header, proj.join("include/engine.hpp"));

    let _ = fs::remove_dir_all(&ws);
}

// 12. SolidJS Real-Time Dashboard from human prompt
#[test]
fn test_human_prompt_12_solidjs_dashboard() {
    let prompt = "make a solidjs dashboard with real-time websocket metrics";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("solid_metrics");
    let proj = ws.join("metrics-dash");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "metrics-dash"}"#).unwrap();
    fs::write(proj.join("src/App.tsx"), "export default function App() {}").unwrap();

    assert!(is_scaffolding_command("npm create vite@latest metrics-dash -- --template solid"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let chart = ground_workspace_path_with_context(&ws, "src/components/Chart.tsx", None);
    assert_eq!(chart, proj.join("src/components/Chart.tsx"));

    let _ = fs::remove_dir_all(&ws);
}

// 13. Astro Docs Site from human prompt
#[test]
fn test_human_prompt_13_astro_docs() {
    let prompt = "i need an astro documentation site for my open-source project with starlight";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("astro_docs");
    let proj = ws.join("docs-site");
    fs::create_dir_all(proj.join("src/pages")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "docs-site"}"#).unwrap();
    fs::write(proj.join("src/pages/index.astro"), "--- // Astro ---").unwrap();

    assert!(is_scaffolding_command("npm create astro@latest docs-site"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let guide = ground_workspace_path_with_context(&ws, "src/pages/guide.astro", None);
    assert_eq!(guide, proj.join("src/pages/guide.astro"));

    let _ = fs::remove_dir_all(&ws);
}

// 14. Bun Fullstack Chat App from human prompt
#[test]
fn test_human_prompt_14_bun_chat() {
    let prompt = "can you develop a bun fullstack web app with real-time websockets?";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("bun_chat");
    let proj = ws.join("chat-app");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "chat-app"}"#).unwrap();
    fs::write(proj.join("src/index.ts"), "console.log('Bun chat');").unwrap();

    assert!(is_scaffolding_command("bun create vite chat-app"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let server = ground_workspace_path_with_context(&ws, "src/server.ts", None);
    assert_eq!(server, proj.join("src/server.ts"));

    let _ = fs::remove_dir_all(&ws);
}

// 15. Deno REST Microservice from human prompt
#[test]
fn test_human_prompt_15_deno_oak() {
    let prompt = "scaffold a deno rest service with oak framework and token authentication";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("deno_oak");
    let proj = ws.join("deno-auth-service");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("deno.json"), r#"{"tasks": {"start": "deno run src/main.ts"}}"#).unwrap();
    fs::write(proj.join("src/main.ts"), "import { Application } from 'oak';").unwrap();

    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let auth_route = ground_workspace_path_with_context(&ws, "src/routes/auth.ts", None);
    assert_eq!(auth_route, proj.join("src/routes/auth.ts"));

    let _ = fs::remove_dir_all(&ws);
}

// 16. Java Maven Spring Service from human prompt
#[test]
fn test_human_prompt_16_java_spring() {
    let prompt = "build me a java spring boot microservice for inventory tracking with maven";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("java_spring");
    let proj = ws.join("inventory-service");
    fs::create_dir_all(proj.join("src/main/java/com/app")).unwrap();
    fs::write(proj.join("pom.xml"), "<project></project>").unwrap();
    fs::write(proj.join("src/main/java/com/app/InventoryApp.java"), "class InventoryApp {}").unwrap();

    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let controller = ground_workspace_path_with_context(&ws, "src/main/java/com/app/InventoryController.java", None);
    assert_eq!(controller, proj.join("src/main/java/com/app/InventoryController.java"));

    let _ = fs::remove_dir_all(&ws);
}

// 17. Ruby on Rails API from human prompt
#[test]
fn test_human_prompt_17_rails_blog() {
    let prompt = "create a ruby on rails api for blog posts and comments";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("rails_blog");
    let proj = ws.join("blog-api");
    fs::create_dir_all(proj.join("app/controllers")).unwrap();
    fs::write(proj.join("Gemfile"), "source 'https://rubygems.org'").unwrap();
    fs::write(proj.join("app/controllers/posts_controller.rb"), "class PostsController < ActionController::API; end").unwrap();

    assert!(is_scaffolding_command("rails new blog-api --api"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let model = ground_workspace_path_with_context(&ws, "app/models/post.rb", None);
    assert_eq!(model, proj.join("app/models/post.rb"));

    let _ = fs::remove_dir_all(&ws);
}

// 18. PHP Laravel REST API from human prompt
#[test]
fn test_human_prompt_18_php_laravel() {
    let prompt = "make me a php laravel rest api for user profile management with composer";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("php_laravel");
    let proj = ws.join("user-api");
    fs::create_dir_all(proj.join("app/Http/Controllers")).unwrap();
    fs::write(proj.join("composer.json"), r#"{"name": "user-api"}"#).unwrap();
    fs::write(proj.join("app/Http/Controllers/UserController.php"), "<?php namespace App\\Http\\Controllers;").unwrap();

    assert!(is_scaffolding_command("composer create-project laravel/laravel user-api"));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let routes = ground_workspace_path_with_context(&ws, "routes/api.php", None);
    assert_eq!(routes, proj.join("routes/api.php"));

    let _ = fs::remove_dir_all(&ws);
}

// 19. Django Recipe App from human prompt
#[test]
fn test_human_prompt_19_django_recipe() {
    let prompt = "hey, can you create a python django web app for recipe sharing with models and views?";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("django_recipe");
    let proj = ws.join("recipe_hub");
    fs::create_dir_all(proj.join("recipes")).unwrap();
    fs::write(proj.join("manage.py"), "#!/usr/bin/env python").unwrap();
    fs::write(proj.join("recipes/views.py"), "from django.shortcuts import render").unwrap();

    assert!(is_scaffolding_command("django-admin startproject recipe_hub"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("recipes/views.py"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let models = ground_workspace_path_with_context(&ws, "recipes/models.py", None);
    assert_eq!(models, proj.join("recipes/models.py"));

    let _ = fs::remove_dir_all(&ws);
}

// 20. End-to-End Human Prompt with File Modifications and Relative Path Extraction
#[test]
fn test_human_prompt_20_e2e_file_modifications_flow() {
    let prompt = "hey please build a modern sleek calculator in react with glassmorphic buttons";
    assert_eq!(classify_user_intent(prompt), UserIntent::CodeGeneration);

    let ws = create_test_workspace("e2e_calc");
    let proj = ws.join("calculator-app");
    fs::create_dir_all(proj.join("src/components")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "calculator-app"}"#).unwrap();
    fs::write(proj.join("src/App.jsx"), "// starter template").unwrap();

    // Model emits code for multiple files using relative paths:
    let assistant_response = r#"
I will scaffold the calculator app and create the component structure.

### File: src/components/Display.jsx
```jsx
export default function Display({ value }) {
    return <div className="display">{value}</div>;
}
```

### File: src/components/Keypad.jsx
```jsx
export default function Keypad({ onInput }) {
    return <div className="keypad"><button onClick={() => onInput('1')}>1</button></div>;
}
```
"#;

    let mods = extract_all_file_modifications(assistant_response, &ws, None);
    assert_eq!(mods.len(), 2);

    // Both files must be grounded inside calculator-app/ and NOT in the workspace root!
    let display_mod = &mods[0];
    assert_eq!(display_mod.file_path, proj.join("src/components/Display.jsx"));
    let display_code = display_mod.apply_to("").unwrap();
    assert!(display_code.contains("export default function Display({ value })"));
    assert!(display_code.contains(r#"<div className="display">{value}</div>"#));

    let keypad_mod = &mods[1];
    assert_eq!(keypad_mod.file_path, proj.join("src/components/Keypad.jsx"));
    let keypad_code = keypad_mod.apply_to("").unwrap();
    assert!(keypad_code.contains("export default function Keypad({ onInput })"));
    assert!(keypad_code.contains(r#"<button onClick={() => onInput('1')}>1</button>"#));

    // Verify on-disk file creation and content integrity
    fs::write(&display_mod.file_path, &display_code).unwrap();
    fs::write(&keypad_mod.file_path, &keypad_code).unwrap();
    assert!(proj.join("src/components/Display.jsx").is_file());
    assert_eq!(fs::read_to_string(&display_mod.file_path).unwrap(), display_code);
    assert!(proj.join("src/components/Keypad.jsx").is_file());
    assert_eq!(fs::read_to_string(&keypad_mod.file_path).unwrap(), keypad_code);

    assert!(!ws.join("src").exists(), "Workspace root must never have root 'src/' created!");

    let _ = fs::remove_dir_all(&ws);
}
