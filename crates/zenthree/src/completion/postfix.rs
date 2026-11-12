use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PostfixTemplate {
    pub label: String,
    pub detail: String,
    pub template: String, // "{expr}" will be replaced with previous expression
}

static POSTFIX_CACHE: RwLock<Option<HashMap<String, Vec<PostfixTemplate>>>> = RwLock::new(None);

/// Dynamically retrieves postfix completion templates from ~/.local/share/zenthree/postfix/<lang>.json
/// Fully dynamic: No hardcoded snippet/postfix tables in the binary. Users can customize JSON directly.
pub fn get_postfix_templates(path: Option<&Path>) -> Vec<PostfixTemplate> {
    let ext = path
        .and_then(|p| p.extension())
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if ext.is_empty() {
        return Vec::new();
    }

    let lang = match ext.as_str() {
        "rs" => "rust",
        "ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "py" => "python",
        "c" | "h" => "c",
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => "cpp",
        "java" => "java",
        "go" => "go",
        other => other,
    };

    if let Ok(read_guard) = POSTFIX_CACHE.read() {
        if let Some(ref map) = *read_guard {
            if let Some(cached) = map.get(lang) {
                return cached.clone();
            }
        }
    }

    let templates = load_postfix_from_disk(lang);

    if let Ok(mut write_guard) = POSTFIX_CACHE.write() {
        let map = write_guard.get_or_insert_with(HashMap::new);
        map.insert(lang.to_string(), templates.clone());
    }

    templates
}

fn load_postfix_from_disk(lang: &str) -> Vec<PostfixTemplate> {
    let Some(data_dir) = dirs::data_local_dir().or_else(dirs::data_dir) else {
        return Vec::new();
    };
    let postfix_file = data_dir.join("zenthree").join("postfix").join(format!("{}.json", lang));

    if let Ok(content) = std::fs::read_to_string(&postfix_file) {
        if let Ok(items) = serde_json::from_str::<Vec<PostfixTemplate>>(&content) {
            return items;
        }
    }

    Vec::new()
}
