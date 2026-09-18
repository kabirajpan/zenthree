use std::path::{Path, PathBuf};

/// Scope where the rule document was discovered.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleScope {
    Global,
    Workspace,
}

/// A parsed and bounded rule document ready for prompt injection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleDocument {
    pub name: String,
    pub source_path: PathBuf,
    pub scope: RuleScope,
    pub content: String,
    pub truncated: bool,
}

/// Known root filenames for rule definitions in priority order.
pub const KNOWN_RULE_FILENAMES: &[&str] = &[
    "AGENTS.md",
    "GEMINI.md",
    "CLAUDE.md",
    ".cursorrules",
    ".windsurfrules",
    ".clinerules",
    ".rules",
];

/// Maximum allowed byte size per individual rule file (12 KB).
pub const MAX_RULE_FILE_BYTES: usize = 12 * 1024;

/// Scans a workspace root and optional global rules directory for active rule documents.
#[derive(Debug, Default)]
pub struct RuleScanner;

impl RuleScanner {
    pub fn new() -> Self {
        Self
    }

    /// Scans a workspace root for known rule files and `.agents/rules/*.md`.
    pub fn scan_workspace(&self, workspace_root: &Path) -> Vec<RuleDocument> {
        let mut documents = Vec::new();
        if !workspace_root.exists() || !workspace_root.is_dir() {
            return documents;
        }

        // 1. Scan known root filenames
        for filename in KNOWN_RULE_FILENAMES {
            let path = workspace_root.join(filename);
            if path.is_file() {
                if let Some(doc) = self.read_rule_file(&path, RuleScope::Workspace, filename) {
                    documents.push(doc);
                }
            }
        }

        // 2. Scan workspace .agents/rules/*.md
        let workspace_agents_rules = workspace_root.join(".agents").join("rules");
        if workspace_agents_rules.is_dir() {
            self.scan_rules_dir(&workspace_agents_rules, RuleScope::Workspace, &mut documents);
        }

        documents
    }

    /// Scans a global directory (e.g. ~/.agents/rules/) for global user rules.
    pub fn scan_global(&self, global_rules_dir: &Path) -> Vec<RuleDocument> {
        let mut documents = Vec::new();
        if global_rules_dir.is_dir() {
            self.scan_rules_dir(global_rules_dir, RuleScope::Global, &mut documents);
        }
        documents
    }

    /// Recursively scans a directory for rule files (.md, .txt, .rules).
    fn scan_rules_dir(&self, dir: &Path, scope: RuleScope, out: &mut Vec<RuleDocument>) {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with(".md") || name.ends_with(".txt") || name.ends_with(".rules") {
                    if let Some(doc) = self.read_rule_file(&path, scope, &name) {
                        out.push(doc);
                    }
                }
            }
        }
    }

    /// Safely reads a rule file enforcing the byte size ceiling.
    pub fn read_rule_file(&self, path: &Path, scope: RuleScope, name: &str) -> Option<RuleDocument> {
        let content_raw = std::fs::read(path).ok()?;
        let (content, truncated) = if content_raw.len() > MAX_RULE_FILE_BYTES {
            let truncated_bytes = &content_raw[..MAX_RULE_FILE_BYTES];
            let safe_str = String::from_utf8_lossy(truncated_bytes).into_owned();
            let mut final_str = safe_str;
            final_str.push_str("\n\n<!-- [Truncated: rule exceeded size limit of 12KB] -->");
            (final_str, true)
        } else {
            (String::from_utf8_lossy(&content_raw).into_owned(), false)
        };

        if content.trim().is_empty() {
            return None;
        }

        Some(RuleDocument {
            name: name.to_string(),
            source_path: path.to_path_buf(),
            scope,
            content,
            truncated,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_scan_workspace_known_files() {
        let dir = tempdir().unwrap();
        let ws = dir.path();

        fs::write(ws.join("AGENTS.md"), "# Agent Instructions\nAlways be concise.").unwrap();
        fs::write(ws.join(".cursorrules"), "Prefer functional style.").unwrap();
        fs::write(ws.join("random.txt"), "Not a rule").unwrap();

        let scanner = RuleScanner::new();
        let rules = scanner.scan_workspace(ws);

        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].name, "AGENTS.md");
        assert_eq!(rules[0].scope, RuleScope::Workspace);
        assert!(!rules[0].truncated);
        assert_eq!(rules[1].name, ".cursorrules");
    }

    #[test]
    fn test_scan_nested_agents_rules_directory() {
        let dir = tempdir().unwrap();
        let ws = dir.path();
        let agents_rules = ws.join(".agents").join("rules");
        fs::create_dir_all(&agents_rules).unwrap();

        fs::write(agents_rules.join("rust-style.md"), "Use idiomatic Rust.").unwrap();

        let scanner = RuleScanner::new();
        let rules = scanner.scan_workspace(ws);

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].name, "rust-style.md");
        assert!(rules[0].content.contains("Use idiomatic Rust."));
    }

    #[test]
    fn test_rule_truncation_when_exceeding_max_bytes() {
        let dir = tempdir().unwrap();
        let ws = dir.path();

        let large_content = "A".repeat(MAX_RULE_FILE_BYTES + 500);
        fs::write(ws.join("AGENTS.md"), large_content).unwrap();

        let scanner = RuleScanner::new();
        let rules = scanner.scan_workspace(ws);

        assert_eq!(rules.len(), 1);
        assert!(rules[0].truncated);
        assert!(rules[0].content.contains("[Truncated: rule exceeded size limit of 12KB]"));
    }
}
