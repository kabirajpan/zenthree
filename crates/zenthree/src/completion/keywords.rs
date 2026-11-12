use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;
use syntax::{LanguageId, LanguageRegistry};

pub struct DynamicKeywordItem {
    pub label: String,
}

static KEYWORD_CACHE: RwLock<Option<HashMap<LanguageId, Vec<String>>>> = RwLock::new(None);

/// Dynamically extracts keywords from Tree-sitter queries for whatever language the file is.
/// Completely dynamic: No hardcoded tables! Tree-sitter and LSP provide all tokens!
pub fn get_keywords_for_path(path: Option<&Path>) -> Vec<String> {
    let path = match path {
        Some(p) => p,
        None => return Vec::new(),
    };

    let registry = LanguageRegistry::new();
    let lang = match registry.detect_language(path) {
        Some(l) => l,
        None => return Vec::new(),
    };

    // Check read cache first
    if let Ok(read_guard) = KEYWORD_CACHE.read() {
        if let Some(ref map) = *read_guard {
            if let Some(cached) = map.get(&lang.id) {
                return cached.clone();
            }
        }
    }

    // Extract directly from Tree-sitter highlight query
    let extracted = lang.dynamic_keywords();

    // Populate write cache
    if let Ok(mut write_guard) = KEYWORD_CACHE.write() {
        let map = write_guard.get_or_insert_with(HashMap::new);
        map.insert(lang.id, extracted.clone());
    }

    extracted
}
