use std::collections::HashMap;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectRecipe {
    #[serde(default)]
    pub build_cmd: Option<String>,
    #[serde(default)]
    pub test_cmd: Option<String>,
    #[serde(default)]
    pub run_cmd: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceMemory {
    /// Custom file pattern to language/format mapping (e.g. "*.dsl" -> "yaml")
    #[serde(default)]
    pub file_associations: HashMap<String, String>,

    /// Verified working project run/build recipes
    #[serde(default)]
    pub recipes: ProjectRecipe,

    /// Key developer conventions or preferences (capped at 5 items)
    #[serde(default)]
    pub user_preferences: Vec<String>,

    /// Most frequently worked files / hotspots (path -> access count, capped at 25)
    #[serde(default)]
    pub frequent_files: HashMap<String, usize>,
}

pub struct ZenthreeDb {
    pub memory: WorkspaceMemory,
    storage_path: PathBuf,
}

impl ZenthreeDb {
    /// Initializes or loads the DB for the given workspace
    pub fn open_or_create(workspace_root: &Path) -> Self {
        let storage_path = Self::compute_storage_path(workspace_root);
        let memory = if storage_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&storage_path) {
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                WorkspaceMemory::default()
            }
        } else {
            WorkspaceMemory::default()
        };

        Self {
            memory,
            storage_path,
        }
    }

    fn compute_storage_path(workspace_root: &Path) -> PathBuf {
        let dot_dir = workspace_root.join(".zenthree");
        if dot_dir.exists() || std::fs::create_dir_all(&dot_dir).is_ok() {
            return dot_dir.join("memory.json");
        }

        // Fallback to user config dir
        if let Some(config) = dirs::config_dir() {
            let user_dir = config.join("zenthree").join("workspaces");
            let _ = std::fs::create_dir_all(&user_dir);
            let safe_name: String = workspace_root
                .to_string_lossy()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '_' })
                .collect();
            return user_dir.join(format!("{}.json", safe_name));
        }

        workspace_root.join(".zenthree_memory.json")
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(parent) = self.storage_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let json = serde_json::to_string_pretty(&self.memory)?;
        std::fs::write(&self.storage_path, json)?;
        Ok(())
    }

    /// Record an active file visit/edit, maintaining a strict LRU cap of 25 files
    pub fn record_file_activity(&mut self, rel_path: &str) {
        let entry = self.memory.frequent_files.entry(rel_path.to_string()).or_insert(0);
        *entry = entry.saturating_add(1);

        if self.memory.frequent_files.len() > 25 {
            // Find key with minimum count and remove
            if let Some((min_key, _)) = self.memory.frequent_files.iter().min_by_key(|(_, &v)| v) {
                let to_remove = min_key.clone();
                self.memory.frequent_files.remove(&to_remove);
            }
        }
        let _ = self.save();
    }

    /// Record or update custom file association (e.g. "*.rule" -> "yaml")
    pub fn set_file_association(&mut self, pattern: &str, lang: &str) {
        self.memory.file_associations.insert(pattern.to_string(), lang.to_string());
        if self.memory.file_associations.len() > 30 {
            if let Some(first_key) = self.memory.file_associations.keys().next().cloned() {
                self.memory.file_associations.remove(&first_key);
            }
        }
        let _ = self.save();
    }

    /// Record a discovered or confirmed build/test/run command
    pub fn set_recipe(&mut self, build: Option<&str>, test: Option<&str>, run: Option<&str>) {
        if let Some(b) = build {
            self.memory.recipes.build_cmd = Some(b.to_string());
        }
        if let Some(t) = test {
            self.memory.recipes.test_cmd = Some(t.to_string());
        }
        if let Some(r) = run {
            self.memory.recipes.run_cmd = Some(r.to_string());
        }
        let _ = self.save();
    }

    /// DEMAND-DRIVEN INJECTION:
    /// Only returns stored memory if the user's prompt is explicitly asking for or related to it.
    /// NEVER dumps memory if the user is asking a direct, self-contained, or unrelated question.
    pub fn get_relevant_context_for_prompt(&self, prompt: &str) -> Option<String> {
        let lower = prompt.to_lowercase();
        let mut notes = Vec::new();

        let is_build_query = lower.contains("build") || lower.contains("compile") || lower.contains("check");
        let is_test_query = lower.contains("test") || lower.contains("spec");
        let is_run_query = lower.contains("run") || lower.contains("start") || lower.contains("launch") || lower.contains("serve");
        let is_convention_query = lower.contains("convention") || lower.contains("preference") || lower.contains("style");

        // 1. Recipes: only when user is asking to build/test/run
        if is_build_query {
            if let Some(ref b) = self.memory.recipes.build_cmd {
                notes.push(format!("Saved Build Command: `{}`", b));
            }
        }
        if is_test_query {
            if let Some(ref t) = self.memory.recipes.test_cmd {
                notes.push(format!("Saved Test Command: `{}`", t));
            }
        }
        if is_run_query {
            if let Some(ref r) = self.memory.recipes.run_cmd {
                notes.push(format!("Saved Run/Dev Command: `{}`", r));
            }
        }

        // 2. Custom file associations: only if prompt mentions a matching pattern
        for (pattern, lang) in &self.memory.file_associations {
            let clean_pat = pattern.trim_start_matches('*').trim_start_matches('.');
            if !clean_pat.is_empty() && lower.contains(&clean_pat.to_lowercase()) {
                notes.push(format!("Custom File Association: `{}` files are treated as `{}`", pattern, lang));
            }
        }

        // 3. User conventions: only if asking about conventions or styling
        if is_convention_query && !self.memory.user_preferences.is_empty() {
            for pref in &self.memory.user_preferences {
                notes.push(format!("User Preference: {}", pref));
            }
        }

        if notes.is_empty() {
            None
        } else {
            Some(format!(
                "### Project Memory (On-Demand Context):\n{}\n*(Note: User's direct instructions in this turn always override any saved memory)*\n\n",
                notes.iter().map(|n| format!("- {}", n)).collect::<Vec<_>>().join("\n")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_demand_driven_retrieval() {
        let tmp = std::env::temp_dir().join("zenthree_db_test");
        let _ = std::fs::create_dir_all(&tmp);

        let mut db = ZenthreeDb::open_or_create(&tmp);
        db.set_recipe(Some("cargo build --release"), Some("cargo test"), Some("cargo run"));
        db.set_file_association("*.rule", "yaml");

        // Unrelated query -> MUST be None (never dumps memory!)
        assert!(db.get_relevant_context_for_prompt("write a quicksort function").is_none());
        assert!(db.get_relevant_context_for_prompt("explain how async works").is_none());

        // Relevant build query -> returns only build recipe
        let build_ctx = db.get_relevant_context_for_prompt("how do I build this project?").unwrap();
        assert!(build_ctx.contains("cargo build --release"));
        assert!(!build_ctx.contains("cargo test"));

        // File association query -> returns matching rule
        let rule_ctx = db.get_relevant_context_for_prompt("check the .rule file syntax").unwrap();
        assert!(rule_ctx.contains("treated as `yaml`"));

        let _ = std::fs::remove_dir_all(&tmp);
    }
}
