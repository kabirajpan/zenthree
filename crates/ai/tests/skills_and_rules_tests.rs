use ai::{
    CustomizationContext, discover_skills, discover_rules,
    render_skills_summary, render_rules_summary,
    execute_read_file,
};

#[test]
fn test_skills_discovery_and_progressive_disclosure() {
    let tmp = std::env::temp_dir().join(format!("zenthree_skills_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let skills_dir = tmp.join(".zenthree").join("skills");
    
    // Create Skill 1: DB Migration
    let db_skill = skills_dir.join("db-migration");
    let _ = std::fs::create_dir_all(&db_skill);
    let db_content = r#"---
name: db-migration
description: PostgreSQL migration playbook with rollback support
---
# Database Migrations
1. Run `cargo sqlx migrate add`
2. Apply changes
"#;
    let _ = std::fs::write(db_skill.join("SKILL.md"), db_content);

    // Create Skill 2: Docker Deploy
    let docker_skill = skills_dir.join("docker-deploy");
    let _ = std::fs::create_dir_all(&docker_skill);
    let docker_content = r#"---
name: docker-deploy
description: Multi-stage Docker production packaging and compose orchestration
---
# Docker Deployment Playbook
Follow security hardening standards...
"#;
    let _ = std::fs::write(docker_skill.join("SKILL.md"), docker_content);

    // Discover skills
    let discovered = discover_skills(&tmp);
    assert_eq!(discovered.len(), 2);

    let names: Vec<String> = discovered.iter().map(|s| s.name.clone()).collect();
    assert!(names.contains(&"db-migration".to_string()));
    assert!(names.contains(&"docker-deploy".to_string()));

    // Verify Progressive Disclosure (discovery only renders summary, NOT full instructions)
    let summary = render_skills_summary(&discovered);
    assert!(summary.contains("### Available Skills:"));
    assert!(summary.contains("`db-migration`"));
    assert!(summary.contains("PostgreSQL migration playbook with rollback support"));
    assert!(summary.contains("`docker-deploy`"));
    assert!(summary.contains("Multi-stage Docker production packaging"));
    // Must NOT leak full body instructions into the top-level index
    assert!(!summary.contains("Follow security hardening standards"));

    // Verify Activation: Agent reads the full SKILL.md using read_file
    let read_res = execute_read_file(&tmp, ".zenthree/skills/db-migration/SKILL.md", None, None);
    assert!(read_res.ok);
    assert!(read_res.output.contains("cargo sqlx migrate add"));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_rules_discovery_and_rendering() {
    let tmp = std::env::temp_dir().join(format!("zenthree_rules_full_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    
    // Create .zenthree/rules/ directory
    let rules_dir = tmp.join(".zenthree").join("rules");
    let _ = std::fs::create_dir_all(&rules_dir);

    // Rule 1: Code style
    let _ = std::fs::write(
        rules_dir.join("error-handling.md"),
        "---\ndescription: Error handling rules\n---\nAll fallible operations must return Result with custom error enums."
    );

    // Rule 2: Root ZENTHREE.md
    let _ = std::fs::write(
        tmp.join("ZENTHREE.md"),
        "Always keep UI components purely stateless and decoupled from store."
    );

    let discovered = discover_rules(&tmp);
    assert_eq!(discovered.len(), 2);

    let rendered = render_rules_summary(&discovered);
    assert!(rendered.contains("### Project Rules & Constraints:"));
    assert!(rendered.contains("All fallible operations must return Result with custom error enums."));
    assert!(rendered.contains("Always keep UI components purely stateless and decoupled from store."));

    let _ = std::fs::remove_dir_all(&tmp);
}

#[test]
fn test_customization_context_integration() {
    let tmp = std::env::temp_dir().join(format!("zenthree_cust_integ_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
    let skills_dir = tmp.join(".zenthree").join("skills").join("security-audit");
    let rules_dir = tmp.join(".zenthree").join("rules");
    let _ = std::fs::create_dir_all(&skills_dir);
    let _ = std::fs::create_dir_all(&rules_dir);

    let _ = std::fs::write(
        skills_dir.join("SKILL.md"),
        "---\nname: security-audit\ndescription: Audit dependencies for CVEs using cargo audit\n---\nRun cargo audit."
    );
    let _ = std::fs::write(
        rules_dir.join("strict-types.md"),
        "Do not use `any` or unchecked casts in TypeScript."
    );

    let cust = CustomizationContext::discover(&tmp);
    assert_eq!(cust.skills.len(), 1);
    assert_eq!(cust.rules.len(), 1);

    let prompt_snippet = cust.render_for_prompt();
    assert!(prompt_snippet.contains("### Project Rules & Constraints:"));
    assert!(prompt_snippet.contains("Do not use `any` or unchecked casts in TypeScript."));
    assert!(prompt_snippet.contains("### Available Skills:"));
    assert!(prompt_snippet.contains("`security-audit`"));
    assert!(prompt_snippet.contains("Audit dependencies for CVEs using cargo audit"));

    let _ = std::fs::remove_dir_all(&tmp);
}
