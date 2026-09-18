//! Phase 3: Skills & Rules System Example
//!
//! Demonstrates how Zenthree discovers, shadows, catalogs, and injects skills & workspace rules:
//! - agentskills.io standard YAML frontmatter parsing and validation
//! - Multi-tier discovery (Built-in -> Global -> Workspace) with shadowing
//! - Progressive disclosure catalog (<available_skills>) under token budgets
//! - Workspace and user rule discovery (AGENTS.md, .cursorrules, .agents/rules/)
//! - Prompt injection (<workspace_rules>) with token budgets and truncation guardrails
//!
//! Run with: `cargo run --example phase3_skills_rules_demo -p agent`

use agent::rules::RulesRegistry;
use agent::skills::{Skill, SkillScope, SkillsRegistry};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn main() {
    println!("============================================================");
    println!("     ZENTHREE AGENT: PHASE 3 SKILLS & RULES SYSTEM DEMO     ");
    println!("============================================================\n");

    // ---------------------------------------------------------------------
    // 1. Built-in Skills & YAML Frontmatter Parsing
    // ---------------------------------------------------------------------
    println!("--- 1. Built-in Skill Parsing (agentskills.io format) ---");
    let sample_skill = r#"---
name: rust-refactor
description: Safely refactors Rust modules and updates visibility modifiers.
---

# Rust Refactoring Guide
1. Run `cargo check` before starting.
2. Maintain public API signatures.
"#;

    let parsed = Skill::parse(sample_skill, Path::new("/skills/rust-refactor"), SkillScope::Workspace)
        .expect("Skill parsing should succeed");

    println!("✓ Successfully parsed skill: `{}`", parsed.name());
    println!("  Scope: {:?}", parsed.scope);
    println!("  Description: {}", parsed.description());
    println!("  Directory: {:?}", parsed.directory_path);

    // ---------------------------------------------------------------------
    // 2. Multi-tier Discovery & Workspace Shadowing
    // ---------------------------------------------------------------------
    println!("\n--- 2. Multi-Tier Discovery & Workspace Shadowing ---");
    let temp_ws = tempdir().expect("Failed to create temp workspace");
    let temp_global = tempdir().expect("Failed to create temp global dir");

    let ws_path = temp_ws.path();
    let global_skills_dir = temp_global.path().join("skills");
    let ws_skills_dir = ws_path.join(".agents").join("skills");

    // Global skill: "code-review"
    let global_review = global_skills_dir.join("code-review");
    fs::create_dir_all(&global_review).unwrap();
    fs::write(
        global_review.join("SKILL.md"),
        "---\nname: code-review\ndescription: Global generic code reviewer.\n---\nGlobal instructions.",
    ).unwrap();

    // Workspace skill overriding "code-review" (Shadowing)
    let ws_review = ws_skills_dir.join("code-review");
    fs::create_dir_all(&ws_review).unwrap();
    fs::write(
        ws_review.join("SKILL.md"),
        "---\nname: code-review\ndescription: Workspace-specific code review with strict clippy checks.\n---\nWorkspace instructions.",
    ).unwrap();

    // Additional workspace skill
    let ws_test = ws_skills_dir.join("run-benchmarks");
    fs::create_dir_all(&ws_test).unwrap();
    fs::write(
        ws_test.join("SKILL.md"),
        "---\nname: run-benchmarks\ndescription: Runs criterion benchmarks and reports regressions.\n---\nBenchmark instructions.",
    ).unwrap();

    let registry = SkillsRegistry::with_custom_global(&[ws_path], Some(&global_skills_dir));
    let mut discovered: Vec<_> = registry.skills().collect();
    discovered.sort_by_key(|s| s.name());

    println!("Discovered {} total unique skills (shadowing applied):", discovered.len());
    for skill in &discovered {
        println!("  - [{:?}] {} (description: {})", skill.scope, skill.name(), skill.description());
    }

    // Verify workspace shadowing
    let review_skill = registry.get_skill("code-review").expect("code-review must exist");
    assert_eq!(review_skill.scope, SkillScope::Workspace);
    assert_eq!(review_skill.description(), "Workspace-specific code review with strict clippy checks.");
    println!("✓ Verified: Workspace `code-review` successfully shadowed Global `code-review`.");

    // ---------------------------------------------------------------------
    // 3. Progressive Disclosure Prompt Catalog
    // ---------------------------------------------------------------------
    println!("\n--- 3. Progressive Disclosure Catalog Generation ---");
    let catalog_prompt = registry.format_prompt_block();
    println!("Catalog Prompt injected into Agent context (metadata only):");
    println!("------------------------------------------------------------");
    println!("{}", catalog_prompt);
    println!("------------------------------------------------------------");

    // Load full skill content on-demand
    println!("\nOn-demand full skill activation for `code-review`:");
    let full_content = &review_skill.body;
    println!("  Content: {}", full_content.trim());

    // ---------------------------------------------------------------------
    // 4. Workspace & User Rules Subsystem
    // ---------------------------------------------------------------------
    println!("\n--- 4. Workspace & User Rules Discovery ---");
    // Create AGENTS.md and .cursorrules in workspace
    fs::write(
        ws_path.join("AGENTS.md"),
        "# Agent Guidelines\n- Never execute destructive disk operations.\n- Verify builds after edits.",
    ).unwrap();
    fs::write(
        ws_path.join(".cursorrules"),
        "- Write documentation for all public functions.\n- Prefer immutable data structures.",
    ).unwrap();

    // Global rule
    let global_rules_dir = temp_global.path().join("rules");
    fs::create_dir_all(&global_rules_dir).unwrap();
    fs::write(
        global_rules_dir.join("company-policy.rules"),
        "- Follow internal coding standards for logging.",
    ).unwrap();

    let rules_registry = RulesRegistry::new(ws_path, Some(global_rules_dir));
    let discovered_rules = rules_registry.discover_rules();

    println!("Discovered {} rule files across workspace and global scopes:", discovered_rules.len());
    for rule in &discovered_rules {
        println!("  - [{:?}] {} ({} bytes)", rule.scope, rule.name, rule.content.len());
    }

    // ---------------------------------------------------------------------
    // 5. Rules Context Injection Prompt Block
    // ---------------------------------------------------------------------
    println!("\n--- 5. Rules Context Injection Prompt Block ---");
    let rules_prompt = rules_registry.generate_rules_prompt().expect("Rules prompt expected");
    println!("Injected <workspace_rules> block:");
    println!("------------------------------------------------------------");
    println!("{}", rules_prompt);
    println!("------------------------------------------------------------");

    println!("\n============================================================");
    println!("       PHASE 3 SKILLS & RULES SYSTEM FULLY OPERATIONAL      ");
    println!("============================================================\n");
}
