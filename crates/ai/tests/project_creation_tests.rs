use std::fs;
use std::path::PathBuf;
use ai::tools::command::{
    find_nearest_manifest_dir, resolve_command_cwd, is_scaffolding_command,
};
use ai::tools::parser::{ground_workspace_path_with_context, extract_all_file_modifications};

fn create_test_workspace(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("zenthree_test_20_{}", name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

// 1. React + Vite
#[test]
fn test_project_1_react_vite() {
    let ws = create_test_workspace("react_vite");
    let proj = ws.join("calculator");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "calculator"}"#).unwrap();
    fs::write(proj.join("src/App.jsx"), "export default function App() {}").unwrap();

    assert!(is_scaffolding_command("npm create vite@latest calculator -- --template react"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/App.jsx"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    // Grounding a new component like "src/components/Keypad.jsx" MUST stay inside calculator/
    let grounded = ground_workspace_path_with_context(&ws, "src/components/Keypad.jsx", None);
    assert_eq!(grounded, proj.join("src/components/Keypad.jsx"));
    assert!(!ws.join("src").exists());

    let _ = fs::remove_dir_all(&ws);
}

// 2. Next.js App Router
#[test]
fn test_project_2_nextjs_app_router() {
    let ws = create_test_workspace("nextjs_app");
    let proj = ws.join("portfolio");
    fs::create_dir_all(proj.join("app")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "portfolio", "dependencies": {"next": "latest"}}"#).unwrap();
    fs::write(proj.join("app/page.tsx"), "export default function Page() {}").unwrap();

    assert!(is_scaffolding_command("npx create-next-app@latest portfolio"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("app/page.tsx"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "app/components/Hero.tsx", None);
    assert_eq!(grounded, proj.join("app/components/Hero.tsx"));

    let _ = fs::remove_dir_all(&ws);
}

// 3. Vue 3 + Vite
#[test]
fn test_project_3_vue_vite() {
    let ws = create_test_workspace("vue_vite");
    let proj = ws.join("vue-dashboard");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "vue-dashboard"}"#).unwrap();
    fs::write(proj.join("src/App.vue"), "<template><div>Dashboard</div></template>").unwrap();

    assert!(is_scaffolding_command("npm create vue@latest vue-dashboard"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/App.vue"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/components/Chart.vue", None);
    assert_eq!(grounded, proj.join("src/components/Chart.vue"));

    let _ = fs::remove_dir_all(&ws);
}

// 4. SvelteKit
#[test]
fn test_project_4_sveltekit() {
    let ws = create_test_workspace("sveltekit");
    let proj = ws.join("svelte-store");
    fs::create_dir_all(proj.join("src/routes")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "svelte-store"}"#).unwrap();
    fs::write(proj.join("src/routes/+page.svelte"), "<h1>Store</h1>").unwrap();

    assert!(is_scaffolding_command("npm create svelte@latest svelte-store"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/routes/+page.svelte"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/lib/Cart.svelte", None);
    assert_eq!(grounded, proj.join("src/lib/Cart.svelte"));

    let _ = fs::remove_dir_all(&ws);
}

// 5. SolidJS
#[test]
fn test_project_5_solidjs() {
    let ws = create_test_workspace("solidjs");
    let proj = ws.join("solid-app");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "solid-app"}"#).unwrap();
    fs::write(proj.join("src/App.tsx"), "export default function App() {}").unwrap();

    assert_eq!(find_nearest_manifest_dir(&proj.join("src/App.tsx"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/components/Counter.tsx", None);
    assert_eq!(grounded, proj.join("src/components/Counter.tsx"));

    let _ = fs::remove_dir_all(&ws);
}

// 6. Astro
#[test]
fn test_project_6_astro() {
    let ws = create_test_workspace("astro");
    let proj = ws.join("astro-blog");
    fs::create_dir_all(proj.join("src/pages")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "astro-blog"}"#).unwrap();
    fs::write(proj.join("src/pages/index.astro"), "--- // Astro ---").unwrap();

    assert!(is_scaffolding_command("npm create astro@latest astro-blog"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/pages/index.astro"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/components/Post.astro", None);
    assert_eq!(grounded, proj.join("src/components/Post.astro"));

    let _ = fs::remove_dir_all(&ws);
}

// 7. Rust Binary CLI
#[test]
fn test_project_7_rust_cli() {
    let ws = create_test_workspace("rust_cli");
    let proj = ws.join("rust-cli");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("Cargo.toml"), "[package]\nname = \"rust-cli\"\nversion = \"0.1.0\"").unwrap();
    fs::write(proj.join("src/main.rs"), "fn main() {}").unwrap();

    assert!(is_scaffolding_command("cargo new rust-cli --bin"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/main.rs"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/commands/run.rs", None);
    assert_eq!(grounded, proj.join("src/commands/run.rs"));

    let _ = fs::remove_dir_all(&ws);
}

// 8. Rust Library Crate
#[test]
fn test_project_8_rust_lib() {
    let ws = create_test_workspace("rust_lib");
    let proj = ws.join("rust-lib");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("Cargo.toml"), "[package]\nname = \"rust-lib\"\nversion = \"0.1.0\"").unwrap();
    fs::write(proj.join("src/lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();

    assert!(is_scaffolding_command("cargo new rust-lib --lib"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/lib.rs"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/engine/core.rs", None);
    assert_eq!(grounded, proj.join("src/engine/core.rs"));

    let _ = fs::remove_dir_all(&ws);
}

// 9. Python FastAPI Service
#[test]
fn test_project_9_python_fastapi() {
    let ws = create_test_workspace("python_fastapi");
    let proj = ws.join("py-api");
    fs::create_dir_all(proj.join("app")).unwrap();
    fs::write(proj.join("pyproject.toml"), "[project]\nname = \"py-api\"").unwrap();
    fs::write(proj.join("app/main.py"), "from fastapi import FastAPI\napp = FastAPI()").unwrap();

    assert_eq!(find_nearest_manifest_dir(&proj.join("app/main.py"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "app/routers/items.py", None);
    assert_eq!(grounded, proj.join("app/routers/items.py"));

    let _ = fs::remove_dir_all(&ws);
}

// 10. Python Django Project
#[test]
fn test_project_10_python_django() {
    let ws = create_test_workspace("python_django");
    let proj = ws.join("django-site");
    fs::create_dir_all(proj.join("django_site")).unwrap();
    fs::write(proj.join("manage.py"), "#!/usr/bin/env python").unwrap();
    fs::write(proj.join("django_site/settings.py"), "SECRET_KEY = 'xyz'").unwrap();

    assert!(is_scaffolding_command("django-admin startproject django-site"));
    // Since manage.py is project root
    let grounded = ground_workspace_path_with_context(&ws, "django-site/blog/models.py", None);
    assert_eq!(grounded, proj.join("blog/models.py"));

    let _ = fs::remove_dir_all(&ws);
}

// 11. Go HTTP Web Server
#[test]
fn test_project_11_go_service() {
    let ws = create_test_workspace("go_service");
    let proj = ws.join("go-service");
    fs::create_dir_all(&proj).unwrap();
    fs::write(proj.join("go.mod"), "module go-service\ngo 1.22").unwrap();
    fs::write(proj.join("main.go"), "package main\nfunc main() {}").unwrap();

    assert!(is_scaffolding_command("go mod init go-service"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("main.go"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "handlers/user.go", None);
    assert_eq!(grounded, proj.join("handlers/user.go"));

    let _ = fs::remove_dir_all(&ws);
}

// 12. Node / Express REST API
#[test]
fn test_project_12_node_express() {
    let ws = create_test_workspace("node_express");
    let proj = ws.join("express-api");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "express-api"}"#).unwrap();
    fs::write(proj.join("src/server.js"), "const express = require('express');").unwrap();

    assert!(is_scaffolding_command("npm init -y"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/server.js"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/routes/api.js", None);
    assert_eq!(grounded, proj.join("src/routes/api.js"));

    let _ = fs::remove_dir_all(&ws);
}

// 13. Bun Fullstack App
#[test]
fn test_project_13_bun_app() {
    let ws = create_test_workspace("bun_app");
    let proj = ws.join("bun-app");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("package.json"), r#"{"name": "bun-app"}"#).unwrap();
    fs::write(proj.join("src/index.ts"), "Bun.serve({ fetch() {} });").unwrap();

    assert!(is_scaffolding_command("bun create vite bun-app"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/index.ts"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/db/client.ts", None);
    assert_eq!(grounded, proj.join("src/db/client.ts"));

    let _ = fs::remove_dir_all(&ws);
}

// 14. Deno Runtime Project
#[test]
fn test_project_14_deno_runtime() {
    let ws = create_test_workspace("deno_runtime");
    let proj = ws.join("deno-server");
    fs::create_dir_all(&proj).unwrap();
    fs::write(proj.join("deno.json"), r#"{"tasks": {"start": "deno run main.ts"}}"#).unwrap();
    fs::write(proj.join("main.ts"), "Deno.serve((req) => new Response('Hello'));").unwrap();

    assert_eq!(find_nearest_manifest_dir(&proj.join("main.ts"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "routes/greet.ts", None);
    assert_eq!(grounded, proj.join("routes/greet.ts"));

    let _ = fs::remove_dir_all(&ws);
}

// 15. Flutter / Dart Mobile App
#[test]
fn test_project_15_flutter_dart() {
    let ws = create_test_workspace("flutter_dart");
    let proj = ws.join("flutter-app");
    fs::create_dir_all(proj.join("lib")).unwrap();
    fs::write(proj.join("pubspec.yaml"), "name: flutter_app\ndependencies:\n  flutter:\n    sdk: flutter").unwrap();
    fs::write(proj.join("lib/main.dart"), "void main() => runApp(MyApp());").unwrap();

    assert!(is_scaffolding_command("flutter create flutter-app"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("lib/main.dart"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "lib/screens/home.dart", None);
    assert_eq!(grounded, proj.join("lib/screens/home.dart"));

    let _ = fs::remove_dir_all(&ws);
}

// 16. C++ CMake Application
#[test]
fn test_project_16_cpp_cmake() {
    let ws = create_test_workspace("cpp_cmake");
    let proj = ws.join("cpp-game");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("CMakeLists.txt"), "cmake_minimum_required(VERSION 3.10)\nproject(CppGame)").unwrap();
    fs::write(proj.join("src/main.cpp"), "int main() { return 0; }").unwrap();

    assert_eq!(find_nearest_manifest_dir(&proj.join("src/main.cpp"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "include/game.h", None);
    assert_eq!(grounded, proj.join("include/game.h"));

    let _ = fs::remove_dir_all(&ws);
}

// 17. Java Maven Spring Boot
#[test]
fn test_project_17_java_maven() {
    let ws = create_test_workspace("java_maven");
    let proj = ws.join("java-service");
    let src_java = proj.join("src/main/java");
    fs::create_dir_all(&src_java).unwrap();
    fs::write(proj.join("pom.xml"), "<project><modelVersion>4.0.0</modelVersion></project>").unwrap();
    fs::write(src_java.join("App.java"), "public class App { public static void main(String[] args) {} }").unwrap();

    assert_eq!(find_nearest_manifest_dir(&src_java.join("App.java"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/main/java/Controller.java", None);
    assert_eq!(grounded, src_java.join("Controller.java"));

    let _ = fs::remove_dir_all(&ws);
}

// 18. Ruby on Rails App
#[test]
fn test_project_18_ruby_rails() {
    let ws = create_test_workspace("ruby_rails");
    let proj = ws.join("rails-app");
    fs::create_dir_all(proj.join("app/controllers")).unwrap();
    fs::write(proj.join("Gemfile"), "source 'https://rubygems.org'\ngem 'rails'").unwrap();
    fs::write(proj.join("app/controllers/application_controller.rb"), "class ApplicationController < ActionController::Base; end").unwrap();

    assert!(is_scaffolding_command("rails new rails-app"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("app/controllers/application_controller.rb"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "app/models/user.rb", None);
    assert_eq!(grounded, proj.join("app/models/user.rb"));

    let _ = fs::remove_dir_all(&ws);
}

// 19. PHP Composer App
#[test]
fn test_project_19_php_composer() {
    let ws = create_test_workspace("php_composer");
    let proj = ws.join("php-api");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("composer.json"), r#"{"name": "vendor/php-api"}"#).unwrap();
    fs::write(proj.join("src/index.php"), "<?php echo 'Hello World';").unwrap();

    assert!(is_scaffolding_command("composer create-project laravel/laravel php-api"));
    assert_eq!(find_nearest_manifest_dir(&proj.join("src/index.php"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/Controllers/Home.php", None);
    assert_eq!(grounded, proj.join("src/Controllers/Home.php"));

    let _ = fs::remove_dir_all(&ws);
}

// 20. Zig Systems Project
#[test]
fn test_project_20_zig_project() {
    let ws = create_test_workspace("zig_project");
    let proj = ws.join("zig-tool");
    fs::create_dir_all(proj.join("src")).unwrap();
    fs::write(proj.join("build.zig"), "const std = @import(\"std\");").unwrap();
    fs::write(proj.join("src/main.zig"), "pub fn main() void {}").unwrap();

    assert_eq!(find_nearest_manifest_dir(&proj.join("src/main.zig"), &ws), Some(proj.clone()));
    assert_eq!(resolve_command_cwd(&ws, None, None), proj);

    let grounded = ground_workspace_path_with_context(&ws, "src/math.zig", None);
    assert_eq!(grounded, proj.join("src/math.zig"));

    // Verify surgical patch extraction routes into the zig subproject
    let patch = "### File: src/math.zig\n```zig\npub fn add(a: i32, b: i32) i32 { return a + b; }\n```";
    let mods = extract_all_file_modifications(patch, &ws, None);
    assert_eq!(mods.len(), 1);
    assert_eq!(mods[0].file_path, proj.join("src/math.zig"));

    let _ = fs::remove_dir_all(&ws);
}
