use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;

use super::rule::{Rule, RuleScope};

/// Discovers all active rules across workspace root, workspace modular directories, and global user configs.
pub fn discover_rules(workspace: &Path) -> Vec<Rule> {
    let mut rules_map: HashMap<String, Rule> = HashMap::new();

    // 1. Global User Rules (~/.config/zene/rules, ~/.zene/rules, ~/.gemini/config/rules)
    let global_dirs = resolve_global_rule_dirs();
    for dir in global_dirs {
        scan_rules_dir(&dir, RuleScope::Global, &mut rules_map);
    }

    // 2. Workspace Modular Rules (<workspace>/.gemini/rules, <workspace>/.agents/rules)
    let ws_gemini_rules = workspace.join(".gemini").join("rules");
    scan_rules_dir(&ws_gemini_rules, RuleScope::WorkspaceModular, &mut rules_map);

    let ws_agents_rules = workspace.join(".agents").join("rules");
    scan_rules_dir(&ws_agents_rules, RuleScope::WorkspaceModular, &mut rules_map);

    // 3. Primary Workspace Root Rules (AGENTS.md, GEMINI.md, .cursorrules)
    scan_workspace_root_rules(workspace, &mut rules_map);

    let mut rules: Vec<Rule> = rules_map.into_values().collect();
    // Sort so higher-scope rules appear first (WorkspaceRoot -> WorkspaceModular -> Global)
    rules.sort_by(|a, b| b.scope.cmp(&a.scope).then_with(|| a.name.cmp(&b.name)));
    rules
}

fn resolve_global_rule_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        dirs.push(config_dir.join("zene").join("rules"));
        dirs.push(config_dir.join("gemini").join("rules"));
    }

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".zene").join("rules"));
        dirs.push(home.join(".gemini").join("config").join("rules"));
    }

    dirs
}

fn scan_rules_dir(dir: &Path, scope: RuleScope, out: &mut HashMap<String, Rule>) {
    if !dir.is_dir() {
        return;
    }

    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext.eq_ignore_ascii_case("md") || ext.eq_ignore_ascii_case("markdown") || ext.eq_ignore_ascii_case("txt") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if let Ok(content) = fs::read_to_string(&path) {
                        let trimmed = content.trim();
                        if !trimmed.is_empty() {
                            let rule = Rule::new(file_name, path.clone(), trimmed, scope);
                            // Higher scope overrides lower scope with same filename
                            match out.get(file_name) {
                                Some(existing) if existing.scope > scope => {}
                                _ => {
                                    out.insert(file_name.to_string(), rule);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn scan_workspace_root_rules(workspace: &Path, out: &mut HashMap<String, Rule>) {
    let candidate_names = ["AGENTS.md", "GEMINI.md", ".cursorrules"];

    for name in &candidate_names {
        let path = workspace.join(name);
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(&path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    let rule = Rule::new(*name, path, trimmed, RuleScope::WorkspaceRoot);
                    out.insert(name.to_string(), rule);
                }
            }
        }
    }
}
