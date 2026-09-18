use crate::environment::{ProtectedPathGuard, WorkspaceJail};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Input parameters for the `edit_file` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditFileInput {
    pub path: String,
    pub search: String,
    pub replace: String,
}

/// Output returned by the `edit_file` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditFileOutput {
    pub path: String,
    pub replacements_made: usize,
    pub strategy_used: String,
}

/// Modifies an existing file using exact matching with whitespace-tolerant fallback.
pub fn execute_edit_file(
    workspace_root: &Path,
    jail: Option<&WorkspaceJail>,
    relative_path: &str,
    search: &str,
    replace: &str,
) -> Result<EditFileOutput, String> {
    let jail_storage;
    let effective_jail = match jail {
        Some(j) => j,
        None => {
            jail_storage = WorkspaceJail::new([workspace_root]);
            &jail_storage
        }
    };

    let resolved_path = match effective_jail.validate_write(Path::new(relative_path)) {
        Ok(p) => p,
        Err(violation) => {
            return Err(format!("Security denial: {}", violation));
        }
    };

    // 2. Protected paths validation
    if ProtectedPathGuard::is_write_blocked(&resolved_path) {
        return Err(format!(
            "Protected path violation: modifying `{}` is strictly prohibited",
            resolved_path.display()
        ));
    }

    // 3. Read existing file
    let original = std::fs::read_to_string(&resolved_path)
        .map_err(|e| format!("Failed to read target file `{}`: {}", relative_path, e))?;

    // 4. Apply replacement with fallback strategies
    let (patched_content, replacements, strategy) = apply_replacement(&original, search, replace)?;

    // 5. Write atomically back to target file
    let parent_dir = resolved_path.parent().unwrap_or(workspace_root);
    let file_name = resolved_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let temp_name = format!(".{}.edit_tmp_{}", file_name, std::process::id());
    let temp_path = parent_dir.join(&temp_name);

    if let Err(e) = std::fs::write(&temp_path, &patched_content) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!("Failed to write temporary file `{}`: {}", temp_path.display(), e));
    }

    if let Err(e) = std::fs::rename(&temp_path, &resolved_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!(
            "Failed to atomically commit edit to `{}`: {}",
            resolved_path.display(),
            e
        ));
    }

    Ok(EditFileOutput {
        path: relative_path.to_string(),
        replacements_made: replacements,
        strategy_used: strategy,
    })
}

/// Applies replacement using exact matching, CRLF normalization, or fuzzy whitespace matching.
pub fn apply_replacement(
    original: &str,
    search: &str,
    replace: &str,
) -> Result<(String, usize, String), String> {
    if search.is_empty() {
        return Err("Search string cannot be empty".to_string());
    }

    // Strategy 1: Exact match
    if original.contains(search) {
        let new_content = original.replacen(search, replace, 1);
        return Ok((new_content, 1, "exact".to_string()));
    }

    // Strategy 2: Line ending (CRLF) normalized match
    let norm_orig = original.replace("\r\n", "\n");
    let norm_search = search.replace("\r\n", "\n");
    let norm_replace = replace.replace("\r\n", "\n");

    if norm_orig.contains(&norm_search) {
        let patched = norm_orig.replacen(&norm_search, &norm_replace, 1);
        // Preserve original CRLF if target was originally CRLF
        let final_content = if original.contains("\r\n") {
            patched.replace('\n', "\r\n")
        } else {
            patched
        };
        return Ok((final_content, 1, "crlf_normalized".to_string()));
    }

    // Strategy 3: Whitespace-tolerant line matching
    if let Some(patched) = fuzzy_whitespace_replace(&norm_orig, &norm_search, &norm_replace) {
        let final_content = if original.contains("\r\n") {
            patched.replace('\n', "\r\n")
        } else {
            patched
        };
        return Ok((final_content, 1, "fuzzy_whitespace".to_string()));
    }

    Err(format!(
        "Search block was not found in target file. Ensure line context and indentation match existing file content."
    ))
}

/// Matches lines ignoring leading/trailing whitespace variances while preserving replacement structure.
fn fuzzy_whitespace_replace(original: &str, search: &str, replace: &str) -> Option<String> {
    let orig_lines: Vec<&str> = original.lines().collect();
    let search_lines: Vec<&str> = search.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();

    if search_lines.is_empty() {
        return None;
    }

    for i in 0..orig_lines.len() {
        if i + search_lines.len() > orig_lines.len() {
            break;
        }

        let mut matches = true;
        for j in 0..search_lines.len() {
            if orig_lines[i + j].trim() != search_lines[j] {
                matches = false;
                break;
            }
        }

        if matches {
            let mut result = Vec::new();
            for line in &orig_lines[..i] {
                result.push(*line);
            }
            for rep_line in replace.lines() {
                result.push(rep_line);
            }
            for line in &orig_lines[i + search_lines.len()..] {
                result.push(*line);
            }

            let mut out = result.join("\n");
            if original.ends_with('\n') {
                out.push('\n');
            }
            return Some(out);
        }
    }

    None
}

/// Returns the JSON Schema specification for `edit_file`.
pub fn edit_file_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "edit_file",
        "description": "Applies a surgical search-and-replace modification to an existing file.\n\nThe `search` block must match existing file content exactly (or with minor whitespace tolerance).\nPreserves line indentation and line endings automatically.\n\n<example>\n{\n  \"tool\": \"edit_file\",\n  \"path\": \"crates/agent/src/config.rs\",\n  \"search\": \"pub const TIMEOUT: u64 = 30;\",\n  \"replace\": \"pub const TIMEOUT: u64 = 60;\"\n}\n</example>",
        "parameters": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project-relative path of the file to edit."
                },
                "search": {
                    "type": "string",
                    "description": "Exact text chunk in the file to find and replace."
                },
                "replace": {
                    "type": "string",
                    "description": "New replacement text."
                }
            },
            "required": ["path", "search", "replace"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_execute_edit_file_exact_match() {
        let dir = tempdir().unwrap();
        let rel_file = "src/config.rs";
        let full = dir.path().join(rel_file);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, "const TIMEOUT: u64 = 30;\nconst RETRIES: u32 = 3;\n").unwrap();

        let res = execute_edit_file(
            dir.path(),
            None,
            rel_file,
            "const TIMEOUT: u64 = 30;",
            "const TIMEOUT: u64 = 60;",
        ).unwrap();

        assert_eq!(res.strategy_used, "exact");
        let updated = fs::read_to_string(&full).unwrap();
        assert!(updated.contains("const TIMEOUT: u64 = 60;"));
        assert!(updated.contains("const RETRIES: u32 = 3;"));
    }

    #[test]
    fn test_execute_edit_file_fuzzy_whitespace() {
        let dir = tempdir().unwrap();
        let rel_file = "src/code.rs";
        let full = dir.path().join(rel_file);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, "    fn run() {\n        let x = 1;\n    }\n").unwrap();

        // Search with 2 spaces instead of 8 spaces
        let res = execute_edit_file(
            dir.path(),
            None,
            rel_file,
            "fn run() {\n  let x = 1;\n}",
            "    fn run() {\n        let x = 42;\n    }",
        ).unwrap();

        assert_eq!(res.strategy_used, "fuzzy_whitespace");
        let updated = fs::read_to_string(&full).unwrap();
        assert!(updated.contains("let x = 42;"));
    }

    #[test]
    fn test_execute_edit_file_protected_path_blocked() {
        let dir = tempdir().unwrap();
        let res = execute_edit_file(dir.path(), None, ".git/config", "a", "b");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Protected path violation"));
    }
}
