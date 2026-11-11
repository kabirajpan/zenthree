use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

use super::catalog::{find_catalog_model, CatalogModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModelInfo {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub file_path: PathBuf,
    pub file_size_bytes: u64,
    pub file_size_display: String,
    pub is_from_catalog: bool,
    pub catalog_id: Option<String>,
    pub ollama_tag: Option<String>,
    pub downloaded_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LocalModelsManifest {
    #[serde(default)]
    pub models: HashMap<String, ManifestEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub size_bytes: u64,
    pub downloaded_at: String,
    pub ollama_tag: Option<String>,
}

pub struct LocalModelManager;

impl LocalModelManager {
    /// Directory where GGUF models are stored: ~/.zenthree/models
    pub fn models_dir() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join(".zenthree").join("models")
        } else {
            PathBuf::from("models")
        }
    }

    /// Manifest file path: ~/.zenthree/models/manifest.json
    pub fn manifest_path() -> PathBuf {
        Self::models_dir().join("manifest.json")
    }

    /// Ensure ~/.zenthree/models directory exists
    pub fn ensure_dir() -> std::io::Result<PathBuf> {
        let dir = Self::models_dir();
        if !dir.exists() {
            fs::create_dir_all(&dir)?;
        }
        Ok(dir)
    }

    /// Load the manifest from disk
    pub fn load_manifest() -> LocalModelsManifest {
        let path = Self::manifest_path();
        if !path.exists() {
            return LocalModelsManifest::default();
        }
        if let Ok(content) = fs::read_to_string(&path) {
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            LocalModelsManifest::default()
        }
    }

    /// Save the manifest to disk
    pub fn save_manifest(manifest: &LocalModelsManifest) -> Result<(), String> {
        let _ = Self::ensure_dir().map_err(|e| e.to_string())?;
        let path = Self::manifest_path();
        let content = serde_json::to_string_pretty(manifest).map_err(|e| e.to_string())?;
        fs::write(path, content).map_err(|e| e.to_string())
    }

    /// Format bytes to human readable string (e.g. 1.25 GB)
    pub fn format_size(bytes: u64) -> String {
        const MB: u64 = 1024 * 1024;
        const GB: u64 = 1024 * MB;
        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.1} MB", bytes as f64 / MB as f64)
        } else {
            format!("{} KB", bytes / 1024)
        }
    }

    /// Check if a given model (by catalog ID, filename, or ollama tag) is downloaded on disk
    pub fn is_downloaded(id_or_tag_or_file: &str) -> bool {
        Self::get_model_path(id_or_tag_or_file).is_some()
    }

    /// Find the on-disk file path for a model ID, filename, or ollama tag
    pub fn get_model_path(id_or_tag_or_file: &str) -> Option<PathBuf> {
        let dir = Self::models_dir();
        if !dir.exists() {
            return None;
        }

        // Direct path check if user passed full path
        let candidate_path = PathBuf::from(id_or_tag_or_file);
        if candidate_path.is_file() && candidate_path.exists() {
            return Some(candidate_path);
        }

        // 1. Check if matches catalog model filename
        if let Some(cat) = find_catalog_model(id_or_tag_or_file) {
            let cat_file = dir.join(&cat.filename);
            if cat_file.is_file() && cat_file.exists() {
                return Some(cat_file);
            }
        }

        // 2. Direct filename in models directory
        let exact_file = dir.join(id_or_tag_or_file);
        if exact_file.is_file() && exact_file.exists() {
            return Some(exact_file);
        }

        // 3. Filename with .gguf appended
        let with_gguf = dir.join(format!("{}.gguf", id_or_tag_or_file));
        if with_gguf.is_file() && with_gguf.exists() {
            return Some(with_gguf);
        }

        // 4. Scan files in models directory for partial match or manifest match
        let manifest = Self::load_manifest();
        for (k, entry) in &manifest.models {
            if k == id_or_tag_or_file || entry.ollama_tag.as_deref() == Some(id_or_tag_or_file) {
                let p = dir.join(&entry.filename);
                if p.is_file() && p.exists() {
                    return Some(p);
                }
            }
        }

        None
    }

    /// List all installed models present in ~/.zenthree/models
    pub fn list_installed() -> Vec<LocalModelInfo> {
        let dir = Self::models_dir();
        if !dir.exists() {
            return Vec::new();
        }

        let manifest = Self::load_manifest();
        let mut installed = Vec::new();

        if let Ok(entries) = fs::read_dir(&dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }

                let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
                // Only consider .gguf files
                if ext != "gguf" {
                    continue;
                }

                let filename = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
                let file_size_bytes = entry.metadata().map(|m| m.len()).unwrap_or(0);
                let file_size_display = Self::format_size(file_size_bytes);

                // Check if matches catalog
                let catalog_match = find_catalog_model(&filename);
                let (id, name, is_from_catalog, catalog_id, ollama_tag) = if let Some(cat) = catalog_match {
                    (cat.id.to_string(), cat.name.to_string(), true, Some(cat.id.to_string()), Some(cat.ollama_tag.to_string()))
                } else if let Some(m_entry) = manifest.models.values().find(|e| e.filename == filename) {
                    (m_entry.id.clone(), m_entry.name.clone(), false, None, m_entry.ollama_tag.clone())
                } else {
                    let base_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or(&filename).to_string();
                    (base_name.clone(), base_name, false, None, None)
                };

                let downloaded_at = manifest.models.values().find(|e| e.filename == filename).map(|e| e.downloaded_at.clone());

                installed.push(LocalModelInfo {
                    id,
                    name,
                    filename,
                    file_path: path,
                    file_size_bytes,
                    file_size_display,
                    is_from_catalog,
                    catalog_id,
                    ollama_tag,
                    downloaded_at,
                });
            }
        }

        // Sort installed by catalog order or name
        installed.sort_by(|a, b| a.name.cmp(&b.name));
        installed
    }

    /// Record a successfully downloaded model into the manifest
    pub fn record_installed(cat: &CatalogModel, actual_size: u64) -> Result<(), String> {
        let mut manifest = Self::load_manifest();
        let entry = ManifestEntry {
            id: cat.id.to_string(),
            name: cat.name.to_string(),
            filename: cat.filename.to_string(),
            size_bytes: actual_size,
            downloaded_at: chrono_like_now(),
            ollama_tag: Some(cat.ollama_tag.to_string()),
        };
        manifest.models.insert(cat.id.to_string(), entry);
        Self::save_manifest(&manifest)
    }

    /// Delete an installed model file and remove from manifest
    pub fn delete_model(id_or_filename: &str) -> Result<(), String> {
        let maybe_path = Self::get_model_path(id_or_filename);
        if let Some(path) = maybe_path {
            if path.exists() {
                fs::remove_file(&path).map_err(|e| format!("Failed to delete file {}: {}", path.display(), e))?;
            }
            let mut manifest = Self::load_manifest();
            manifest.models.remove(id_or_filename);
            let _ = Self::save_manifest(&manifest);
            Ok(())
        } else {
            Err(format!("Model '{}' not found on disk", id_or_filename))
        }
    }
}

fn chrono_like_now() -> String {
    let now = std::time::SystemTime::now();
    let duration = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // Simple ISO-8601-like timestamp
    format!("timestamp-{}", secs)
}
