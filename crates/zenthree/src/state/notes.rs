#![allow(dead_code)]
use std::collections::{HashMap, HashSet};
use std::fs;
use chrono::Local;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum NotesVaultScope {
    #[default]
    Project,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, Default)]
pub enum NoteFolderOption {
    #[default]
    Default,
    Star,
    Todo,
    Reminder,
    Calendar,
    Private,
}

impl NoteFolderOption {
    pub fn icon(&self) -> &'static str {
        match self {
            NoteFolderOption::Default => "\u{f0c82}",
            NoteFolderOption::Star => "\u{f069d}",
            NoteFolderOption::Todo => "\u{f197e}",
            NoteFolderOption::Reminder => "\u{f0aba}",
            NoteFolderOption::Calendar => "\u{f0e17}",
            NoteFolderOption::Private => "\u{f0250}",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            NoteFolderOption::Default => "Default Folder",
            NoteFolderOption::Star => "Star Folder",
            NoteFolderOption::Todo => "Todo Folder",
            NoteFolderOption::Reminder => "Reminder Folder",
            NoteFolderOption::Calendar => "Calendar Folder",
            NoteFolderOption::Private => "Private Folder",
        }
    }

    pub fn all() -> &'static [NoteFolderOption] {
        &[
            NoteFolderOption::Default,
            NoteFolderOption::Star,
            NoteFolderOption::Todo,
            NoteFolderOption::Reminder,
            NoteFolderOption::Calendar,
            NoteFolderOption::Private,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineCreateMode {
    File { parent_dir: PathBuf },
    Folder { parent_dir: PathBuf },
}

#[derive(Debug, Clone)]
pub struct NoteTreeNode {
    pub path: PathBuf,
    pub name: String,
    #[allow(dead_code)]
    pub file_name: String,
    pub is_dir: bool,
    pub depth: usize,
    pub is_expanded: bool,
    pub is_pinned: bool,
    pub is_daily: bool,
    pub ext_badge: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NotesState {
    pub scope: NotesVaultScope,
    pub search_query: String,
    pub search_active: bool,
    #[allow(dead_code)]
    pub selected_tag: Option<String>,
    pub expanded_folders: HashSet<PathBuf>,
    pub tree_nodes: Vec<NoteTreeNode>,
    pub all_tags: Vec<(String, usize)>,
    pub inline_create: Option<InlineCreateMode>,
    pub inline_input: String,
    pub pinned_paths: HashSet<PathBuf>,
    pub folder_options: HashMap<PathBuf, NoteFolderOption>,
    pub last_scanned_at: Option<std::time::Instant>,
}

impl Default for NotesState {
    fn default() -> Self {
        Self {
            scope: NotesVaultScope::Project,
            search_query: String::new(),
            search_active: false,
            selected_tag: None,
            expanded_folders: HashSet::new(),
            tree_nodes: Vec::new(),
            all_tags: Vec::new(),
            inline_create: None,
            inline_input: String::new(),
            pinned_paths: HashSet::new(),
            folder_options: HashMap::new(),
            last_scanned_at: None,
        }
    }
}

impl NotesState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_folder_option(&self, path: &Path) -> NoteFolderOption {
        self.folder_options.get(path).copied().unwrap_or(NoteFolderOption::Default)
    }

    /// Resolve the vault directory for the current scope.
    pub fn get_vault_dir(&self, current_dir: &Path) -> PathBuf {
        match self.scope {
            NotesVaultScope::Project => current_dir.join(".zenthree").join("notes"),
            NotesVaultScope::Global => {
                if let Some(config_dir) = dirs::config_dir() {
                    config_dir.join("zenthree").join("notes")
                } else if let Some(home_dir) = dirs::home_dir() {
                    home_dir.join(".config").join("zenthree").join("notes")
                } else {
                    PathBuf::from(".zenthree_notes")
                }
            }
        }
    }

    /// Ensure the vault directory exists on disk and seed sample notes if brand new.
    pub fn ensure_vault_dir(&self, current_dir: &Path) -> std::io::Result<PathBuf> {
        let vault = self.get_vault_dir(current_dir);
        if !vault.exists() {
            fs::create_dir_all(&vault)?;
            self.seed_initial_notes(&vault);
        }
        Ok(vault)
    }

    fn seed_initial_notes(&self, vault_dir: &Path) {
        let today = Local::now().format("%Y-%m-%d").to_string();
        let welcome_path = vault_dir.join("Welcome to Notes.md");
        if !welcome_path.exists() {
            let welcome_content = format!(
                r#"# Welcome to Zenthree Notes 📝

Your local-first Obsidian-style knowledge base right inside your IDE.

## Features
- **Obsidian Folder Hierarchy**: Organize notes into folders and subfolders.
- **Local Markdown Files**: Notes live on disk in your project (`.zenthree/notes/`) or global vault (`~/.config/zenthree/notes/`).
- **Daily Notes**: Click the 📅 icon to open today's dev log.
- **Tags**: Use `#tags` like #architecture, #todo, or #ideas to filter notes instantly.
- **Search**: Fast fuzzy search across all vault notes.

## Quick Checklist
- [x] Explored Zenthree Notes
- [ ] Create my first project note
- [ ] Connect ideas with tags

#guide #welcome
"#
            );
            let _ = fs::write(welcome_path, welcome_content);
        }

        let daily_path = vault_dir.join(format!("{}.md", today));
        if !daily_path.exists() {
            let daily_content = format!(
                r#"# Daily Note - {}

## Standup & Focus
- [ ] Review sprint tasks
- [ ] Code implementation

## Notes & Thoughts
Working on Zenthree developer environment.

#daily #standup
"#,
                today
            );
            let _ = fs::write(daily_path, daily_content);
        }
    }

    /// Scan vault and build the recursive folder/file tree.
    pub fn scan_notes(&mut self, current_dir: &Path) {
        let vault = match self.ensure_vault_dir(current_dir) {
            Ok(v) => v,
            Err(_) => return,
        };

        let mut tag_counts = std::collections::HashMap::new();
        let mut nodes = Vec::new();

        // Scan tags across all markdown files in vault recursively
        scan_tags_recursive(&vault, &mut tag_counts);

        // Build visible tree nodes
        let query = self.search_query.trim().to_lowercase();
        self.build_tree_recursive(&vault, 0, &query, &mut nodes);

        let mut all_tags: Vec<(String, usize)> = tag_counts.into_iter().collect();
        all_tags.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        self.tree_nodes = nodes;
        self.all_tags = all_tags;
        self.last_scanned_at = Some(std::time::Instant::now());
    }

    fn build_tree_recursive(
        &self,
        dir: &Path,
        depth: usize,
        search_query: &str,
        out: &mut Vec<NoteTreeNode>,
    ) {
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut dirs = Vec::new();
        let mut files = Vec::new();

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            if file_name.starts_with('.') {
                continue;
            }

            if path.is_dir() {
                dirs.push((file_name, path));
            } else {
                files.push((file_name, path));
            }
        }

        // Sort folders alphabetically
        dirs.sort_by(|a, b| a.0.to_lowercase().cmp(&b.0.to_lowercase()));

        // Sort files: pinned first, then alphabetical
        files.sort_by(|a, b| {
            let a_pinned = self.pinned_paths.contains(&a.1);
            let b_pinned = self.pinned_paths.contains(&b.1);
            b_pinned.cmp(&a_pinned).then_with(|| a.0.to_lowercase().cmp(&b.0.to_lowercase()))
        });

        // 1. Process directories
        for (dir_name, dir_path) in dirs {
            let is_expanded = self.expanded_folders.contains(&dir_path) || !search_query.is_empty();

            out.push(NoteTreeNode {
                path: dir_path.clone(),
                name: dir_name.clone(),
                file_name: dir_name,
                is_dir: true,
                depth,
                is_expanded,
                is_pinned: false,
                is_daily: false,
                ext_badge: None,
            });

            if is_expanded {
                self.build_tree_recursive(&dir_path, depth + 1, search_query, out);
            }
        }

        // 2. Process files
        for (file_name, file_path) in files {
            let ext = file_path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
            let is_md = ext == "md" || ext == "markdown";

            // Clean display name (hide .md extension like Obsidian)
            let display_name = if is_md {
                file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or(&file_name)
                    .to_string()
            } else {
                file_name.clone()
            };

            let ext_badge = if !is_md && !ext.is_empty() {
                Some(ext.to_uppercase())
            } else {
                None
            };

            // Search filter
            if !search_query.is_empty() {
                let matches_name = display_name.to_lowercase().contains(search_query);
                let matches_file = file_name.to_lowercase().contains(search_query);
                if !matches_name && !matches_file {
                    continue;
                }
            }

            let is_daily = is_daily_note_name(&display_name);
            let is_pinned = self.pinned_paths.contains(&file_path);

            out.push(NoteTreeNode {
                path: file_path,
                name: display_name,
                file_name,
                is_dir: false,
                depth,
                is_expanded: false,
                is_pinned,
                is_daily,
                ext_badge,
            });
        }
    }

    /// Toggle folder expanded/collapsed state.
    pub fn toggle_folder(&mut self, folder_path: &Path, current_dir: &Path) {
        if self.expanded_folders.contains(folder_path) {
            self.expanded_folders.remove(folder_path);
        } else {
            self.expanded_folders.insert(folder_path.to_path_buf());
        }
        self.scan_notes(current_dir);
    }

    /// Collapse all folders in vault.
    pub fn collapse_all(&mut self, current_dir: &Path) {
        self.expanded_folders.clear();
        self.scan_notes(current_dir);
    }

    /// Create a new note file. Returns the created path on success.
    pub fn create_note_file(&mut self, parent_dir: &Path, title: &str, current_dir: &Path) -> Result<PathBuf, String> {
        let trimmed = title.trim();
        if trimmed.is_empty() {
            return Err("Note name cannot be empty".to_string());
        }

        let clean_filename = if trimmed.ends_with(".md") {
            trimmed.to_string()
        } else {
            format!("{}.md", trimmed)
        };

        let file_path = parent_dir.join(&clean_filename);
        if file_path.exists() {
            return Err("A note with this name already exists".to_string());
        }

        let initial_content = format!("# {}\n\n", trimmed.trim_end_matches(".md"));
        fs::write(&file_path, initial_content)
            .map_err(|e| format!("Failed to create note: {}", e))?;

        // Expand parent folder so the new note is visible
        self.expanded_folders.insert(parent_dir.to_path_buf());
        self.scan_notes(current_dir);
        Ok(file_path)
    }

    /// Create a new folder.
    pub fn create_note_folder(&mut self, parent_dir: &Path, folder_name: &str, current_dir: &Path) -> Result<PathBuf, String> {
        let trimmed = folder_name.trim();
        if trimmed.is_empty() {
            return Err("Folder name cannot be empty".to_string());
        }

        let folder_path = parent_dir.join(trimmed);
        if folder_path.exists() {
            return Err("A folder with this name already exists".to_string());
        }

        fs::create_dir_all(&folder_path)
            .map_err(|e| format!("Failed to create folder: {}", e))?;

        self.expanded_folders.insert(parent_dir.to_path_buf());
        self.expanded_folders.insert(folder_path.clone());
        self.scan_notes(current_dir);
        Ok(folder_path)
    }

    /// Create or retrieve today's daily note. Returns the note's PathBuf.
    pub fn create_or_open_daily_note(&mut self, current_dir: &Path) -> Result<PathBuf, String> {
        let vault = self
            .ensure_vault_dir(current_dir)
            .map_err(|e| format!("Failed to access vault: {}", e))?;

        let today = Local::now().format("%Y-%m-%d").to_string();
        let file_path = vault.join(format!("{}.md", today));

        if !file_path.exists() {
            let template = format!(
                r#"# Daily Note - {}

## Standup & Focus
- [ ] 

## Notes & Thoughts


#daily
"#,
                today
            );
            fs::write(&file_path, template)
                .map_err(|e| format!("Failed to create daily note: {}", e))?;
        }

        self.scan_notes(current_dir);
        Ok(file_path)
    }

    /// Toggle pinned status of a note.
    pub fn toggle_pin(&mut self, path: &Path, current_dir: &Path) {
        if self.pinned_paths.contains(path) {
            self.pinned_paths.remove(path);
        } else {
            self.pinned_paths.insert(path.to_path_buf());
        }
        self.scan_notes(current_dir);
    }

    /// Delete a note or folder from disk.
    pub fn delete_item(&mut self, path: &Path, current_dir: &Path) -> Result<(), String> {
        self.pinned_paths.remove(path);
        self.expanded_folders.remove(path);

        if path.is_dir() {
            fs::remove_dir_all(path).map_err(|e| format!("Failed to delete folder: {}", e))?;
        } else if path.exists() {
            fs::remove_file(path).map_err(|e| format!("Failed to delete note: {}", e))?;
        }

        self.scan_notes(current_dir);
        Ok(())
    }
}

fn scan_tags_recursive(dir: &Path, tag_counts: &mut std::collections::HashMap<String, usize>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_tags_recursive(&path, tag_counts);
        } else if path.extension().and_then(|s| s.to_str()) == Some("md") {
            if let Ok(content) = fs::read_to_string(&path) {
                let mut seen_in_file = HashSet::new();
                for word in content.split_whitespace() {
                    if word.starts_with('#') && word.len() > 1 {
                        let candidate = word.trim_start_matches('#').trim_matches(|c: char| !c.is_alphanumeric() && c != '_' && c != '-');
                        if !candidate.is_empty() && !candidate.chars().all(|c| c.is_ascii_digit()) {
                            let tag = format!("#{}", candidate.to_lowercase());
                            if seen_in_file.insert(tag.clone()) {
                                *tag_counts.entry(tag).or_insert(0) += 1;
                            }
                        }
                    }
                }
            }
        }
    }
}

fn is_daily_note_name(name: &str) -> bool {
    let stem = name.trim_end_matches(".md");
    if stem.len() == 10 {
        let chars: Vec<char> = stem.chars().collect();
        chars[0..4].iter().all(|c| c.is_ascii_digit())
            && chars[4] == '-'
            && chars[5..7].iter().all(|c| c.is_ascii_digit())
            && chars[7] == '-'
            && chars[8..10].iter().all(|c| c.is_ascii_digit())
    } else {
        false
    }
}
