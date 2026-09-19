use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRule {
    pub name: String,
    pub path: String,
    pub absolute_path: PathBuf,
    pub content: String,
}

/// Discovers project rules in the workspace.
/// Supports `.zenthree/rules/*.md`, `ZENTHREE.md`, `RULES.md`, and `.cursorrules`.
pub fn discover_rules(workspace_root: &Path) -> Vec<ProjectRule> {
    let mut rules = Vec::new();

    // 1. Single standalone rule files in root
    let root_candidates = [
        "ZENTHREE.md",
        "RULES.md",
        ".zenthreerules",
        ".cursorrules",
    ];

    for file_name in root_candidates {
        let path = workspace_root.join(file_name);
        if path.is_file() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let trimmed = content.trim();
                if !trimmed.is_empty() {
                    rules.push(ProjectRule {
                        name: file_name.to_string(),
                        path: file_name.to_string(),
                        absolute_path: path,
                        content: trimmed.to_string(),
                    });
                }
            }
        }
    }

    // 2. Rules directory: `.zenthree/rules/` and `.cursor/rules/`
    let rule_dirs = [
        workspace_root.join(".zenthree").join("rules"),
        workspace_root.join(".cursor").join("rules"),
    ];

    for rdir in rule_dirs {
        if !rdir.is_dir() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&rdir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    if ext == "md" || ext == "mdc" || ext == "txt" {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let trimmed = content.trim();
                            if !trimmed.is_empty() {
                                let file_stem = path.file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("rule")
                                    .to_string();

                                let rel = path.strip_prefix(workspace_root)
                                    .map(|p| p.to_string_lossy().to_string())
                                    .unwrap_or_else(|_| path.to_string_lossy().to_string());

                                rules.push(ProjectRule {
                                    name: file_stem,
                                    path: rel,
                                    absolute_path: path,
                                    content: trimmed.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    rules
}

/// Renders discovered project rules cleanly for inclusion in the AI context.
pub fn render_rules_summary(rules: &[ProjectRule]) -> String {
    if rules.is_empty() {
        return String::new();
    }

    let mut out = String::from("### Project Rules & Constraints:\n");
    for r in rules {
        out.push_str(&format!("#### Rule: `{}` (from `{}`)\n", r.name, r.path));
        // Strip frontmatter if present for clean readability
        let clean_content = strip_yaml_frontmatter(&r.content);
        out.push_str(clean_content);
        out.push_str("\n\n");
    }
    out
}

fn strip_yaml_frontmatter(text: &str) -> &str {
    let trimmed = text.trim();
    if trimmed.starts_with("---") {
        let after_first = &trimmed[3..];
        if let Some(end_idx) = after_first.find("---") {
            return after_first[end_idx + 3..].trim();
        }
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discover_and_render_rules() {
        let tmp = std::env::temp_dir().join(format!("zenthree_rules_test_{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        let rules_dir = tmp.join(".zenthree").join("rules");
        let _ = std::fs::create_dir_all(&rules_dir);

        let _ = std::fs::write(rules_dir.join("code-style.md"), "---\ndescription: Code style guidelines\n---\nAlways use strict Rust error handling with `?`.");
        let _ = std::fs::write(tmp.join("ZENTHREE.md"), "# Project Conventions\nNever mutate global state.");

        let discovered = discover_rules(&tmp);
        assert_eq!(discovered.len(), 2);

        let rendered = render_rules_summary(&discovered);
        assert!(rendered.contains("### Project Rules & Constraints:"));
        assert!(rendered.contains("Never mutate global state."));
        assert!(rendered.contains("Always use strict Rust error handling with `?`."));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
