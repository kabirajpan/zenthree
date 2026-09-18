use crate::environment::{ProtectedPathGuard, WorkspaceJail};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Input parameters for the `write_file` tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileInput {
    pub path: String,
    pub content: String,
}

/// Output returned by the `write_file` tool.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WriteFileOutput {
    pub path: String,
    pub bytes_written: usize,
    pub created: bool,
}

/// Creates a new file or overwrites an existing file atomically.
pub fn execute_write_file(
    workspace_root: &Path,
    jail: Option<&WorkspaceJail>,
    relative_path: &str,
    content: &str,
) -> Result<WriteFileOutput, String> {
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

    // 2. Protected paths validation (blocking .git/, .env*, credentials)
    if ProtectedPathGuard::is_write_blocked(&resolved_path) {
        return Err(format!(
            "Protected path violation: writes to `{}` are strictly prohibited",
            resolved_path.display()
        ));
    }

    // 3. Ensure parent directory exists
    if let Some(parent) = resolved_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create parent directory `{}`: {}", parent.display(), e))?;
        }
    }

    let existed_before = resolved_path.exists();

    // 4. Atomic write using temporary sibling file
    let parent_dir = resolved_path.parent().unwrap_or(workspace_root);
    let file_name = resolved_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let temp_name = format!(".{}.tmp_{}", file_name, std::process::id());
    let temp_path = parent_dir.join(&temp_name);

    if let Err(e) = std::fs::write(&temp_path, content) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!("Failed to write temporary file `{}`: {}", temp_path.display(), e));
    }

    if let Err(e) = std::fs::rename(&temp_path, &resolved_path) {
        let _ = std::fs::remove_file(&temp_path);
        return Err(format!(
            "Failed to atomically commit file `{}`: {}",
            resolved_path.display(),
            e
        ));
    }

    Ok(WriteFileOutput {
        path: relative_path.to_string(),
        bytes_written: content.len(),
        created: !existed_before,
    })
}

/// Returns the JSON Schema specification for `write_file`.
pub fn write_file_schema() -> serde_json::Value {
    serde_json::json!({
        "name": "write_file",
        "description": "Creates a new file or overwrites an existing file with complete new content.\n\nTo make surgical edits to existing files, prefer `edit_file` instead.\nParent directories are created automatically if they do not exist.\n\n<example>\n`crates/agent/src/tools/sample.rs`\nNotice the path is project-relative. Never use leading slashes `/`.\n</example>",
        "parameters": {
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Project-relative path of the file to create or overwrite."
                },
                "content": {
                    "type": "string",
                    "description": "The complete contents of the file."
                }
            },
            "required": ["path", "content"]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_execute_write_file_creation_and_overwrite() {
        let dir = tempdir().unwrap();
        let rel_file = "src/sub/new_file.rs";

        // 1. Create file with auto directory creation
        let res = execute_write_file(dir.path(), None, rel_file, "pub fn hello() {}\n").unwrap();
        assert!(res.created);
        assert_eq!(res.bytes_written, 18);
        assert!(dir.path().join(rel_file).exists());

        // 2. Overwrite file
        let res2 = execute_write_file(dir.path(), None, rel_file, "pub fn world() {}\n").unwrap();
        assert!(!res2.created);
        let content = std::fs::read_to_string(dir.path().join(rel_file)).unwrap();
        assert_eq!(content, "pub fn world() {}\n");
    }

    #[test]
    fn test_execute_write_file_protected_path_blocked() {
        let dir = tempdir().unwrap();
        let git_hook = ".git/hooks/pre-commit";
        let res = execute_write_file(dir.path(), None, git_hook, "#!/bin/sh\nexit 0");
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Protected path violation"));

        let env_file = ".env";
        let res_env = execute_write_file(dir.path(), None, env_file, "SECRET=123");
        assert!(res_env.is_err());
        assert!(res_env.unwrap_err().contains("Protected path violation"));
    }
}
