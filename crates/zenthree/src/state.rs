use std::path::{Path, PathBuf};
use crate::theme::{ThemeMode, ThemeColors};
use editor::buffer::Buffer;
use editor::editor::Editor;

#[derive(Clone)]
pub struct EditorTab {
    pub name: String,
    pub path: Option<PathBuf>,
    pub editor: Editor,
    pub is_dirty: bool,
    pub scroll_y: f32,
    pub shaping_cache: std::collections::HashMap<usize, (String, zenthra_text::prelude::ShapedBuffer)>,
    pub version: u64,
}

#[derive(Debug, Clone)]
pub struct FileTreeNode {
    pub path: PathBuf,
    pub name: String,
    pub depth: usize,
    pub is_dir: bool,
    pub is_expanded: bool,
}

pub struct ZenthreeState {
    pub theme: ThemeMode,
    pub accent_color: String,
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub status_bar_visible: bool,
    pub line_numbers_visible: bool,
    pub zoom_level: f32,
    pub active_resize_sidebar: bool,
    pub tabs: Vec<EditorTab>,
    pub active_tab: Option<usize>,
    pub current_dir: PathBuf,
    pub recent_files: Vec<PathBuf>,
    pub sidebar_items: Vec<FileTreeNode>,
    pub expanded_dirs: std::collections::HashSet<PathBuf>,
    pub status_message: String,
    pub about_open: bool,
    pub ctrl_pressed: bool,
    pub shift_pressed: bool,
    pub alt_pressed: bool,
    pub highlighter: syntax::SyntaxHighlighter,
    pub formatter: formatter::Formatter,
    pub indent_calc: syntax::IndentCalculator,
    pub clipboard: Option<String>,
    pub last_cursor_activity: std::time::Instant,
}

impl ZenthreeState {
    pub fn new() -> Self {
        // Try workspace argument, fallback to std::env::current_dir(), fallback to home dir
        let args: Vec<String> = std::env::args().collect();
        let mut current_dir = std::env::current_dir().unwrap_or_else(|_| {
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
        });

        if args.len() > 1 {
            let path = PathBuf::from(&args[1]);
            if path.exists() {
                current_dir = path;
            }
        }

        let highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::dark());
        let formatter = formatter::Formatter::new();
        let indent_calc = syntax::IndentCalculator::new();

        let mut state = Self {
            theme: ThemeMode::Dark,
            accent_color: "blue".to_string(),
            sidebar_visible: true,
            sidebar_width: 250.0,
            status_bar_visible: true,
            line_numbers_visible: true,
            zoom_level: 1.0,
            active_resize_sidebar: false,
            tabs: Vec::new(),
            active_tab: None,
            current_dir,
            recent_files: Vec::new(),
            sidebar_items: Vec::new(),
            expanded_dirs: std::collections::HashSet::new(),
            status_message: "Welcome to Zenthree!".to_string(),
            about_open: false,
            ctrl_pressed: false,
            shift_pressed: false,
            alt_pressed: false,
            highlighter,
            formatter,
            indent_calc,
            clipboard: None,
            last_cursor_activity: std::time::Instant::now(),
        };
        state.rebuild_tree();
        state
    }

    pub fn colors(&self) -> ThemeColors {
        ThemeColors::get_custom(self.theme, &self.accent_color)
    }

    pub fn rebuild_tree(&mut self) {
        self.sidebar_items.clear();
        let root = self.current_dir.clone();
        self.scan_dir_recursive(&root, 0);
    }

    fn scan_dir_recursive(&mut self, dir: &Path, depth: usize) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            // Sort: folders first, then files alphabetically
            paths.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir != b_is_dir {
                    b_is_dir.cmp(&a_is_dir)
                } else {
                    a.file_name().cmp(&b.file_name())
                }
            });

            for path in paths {
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                // Skip hidden files/directories to prevent scanning massive config/cache folders
                if name.starts_with('.') {
                    continue;
                }

                let is_dir = path.is_dir();
                let is_expanded = is_dir && self.expanded_dirs.contains(&path);

                self.sidebar_items.push(FileTreeNode {
                    path: path.clone(),
                    name,
                    depth,
                    is_dir,
                    is_expanded,
                });

                if is_expanded {
                    self.scan_dir_recursive(&path, depth + 1);
                }
            }
        }
    }

    pub fn open_file(&mut self, path: &Path) {
        if let Ok(content) = std::fs::read_to_string(path) {
            let name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();

            // Track recent files
            self.recent_files.retain(|p| p != path);
            self.recent_files.insert(0, path.to_path_buf());
            if self.recent_files.len() > 10 {
                self.recent_files.truncate(10);
            }

            // Check if already open
            for (idx, tab) in self.tabs.iter().enumerate() {
                if tab.path.as_ref() == Some(&path.to_path_buf()) {
                    self.active_tab = Some(idx);
                    self.status_message = format!("Switched to {}", name);
                    return;
                }
            }

            self.tabs.push(EditorTab {
                name,
                path: Some(path.to_path_buf()),
                editor: Editor::new(Buffer::from_text(&content)),
                is_dirty: false,
                scroll_y: 0.0,
                shaping_cache: std::collections::HashMap::new(),
                version: 0,
            });
            self.active_tab = Some(self.tabs.len() - 1);
            self.status_message = format!("Opened file: {}", path.display());
        } else {
            self.status_message = format!("Failed to open file: {}", path.display());
        }
    }

    pub fn save_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                if self.tabs[idx].path.is_none() {
                    self.save_active_tab_as();
                    return;
                }
                let tab = &mut self.tabs[idx];
                if let Some(ref path) = tab.path {
                    if std::fs::write(path, tab.editor.buffer.to_string()).is_ok() {
                        tab.is_dirty = false;
                        self.status_message = format!("Saved file: {}", tab.name);
                    } else {
                        self.status_message = format!("Failed to save file: {}", tab.name);
                    }
                }
            }
        }
    }

    pub fn save_active_tab_as(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let initial_dir = self.tabs[idx].path.as_ref()
                    .and_then(|p| p.parent())
                    .unwrap_or(&self.current_dir);
                let mut dialog = rfd::FileDialog::new().set_directory(initial_dir);
                if let Some(ref p) = self.tabs[idx].path {
                    if let Some(file_name) = p.file_name().and_then(|n| n.to_str()) {
                        dialog = dialog.set_file_name(file_name);
                    }
                } else {
                    dialog = dialog.set_file_name(&self.tabs[idx].name);
                }

                if let Some(path) = dialog.save_file() {
                    let tab = &mut self.tabs[idx];
                    if std::fs::write(&path, tab.editor.buffer.to_string()).is_ok() {
                        tab.name = path.file_name().and_then(|n| n.to_str()).unwrap_or("Unknown").to_string();
                        tab.path = Some(path.clone());
                        tab.is_dirty = false;
                        if !self.recent_files.contains(&path) {
                            self.recent_files.insert(0, path);
                            if self.recent_files.len() > 10 {
                                self.recent_files.pop();
                            }
                        }
                        self.status_message = format!("Saved as: {}", tab.name);
                    } else {
                        self.status_message = format!("Failed to save file: {}", path.display());
                    }
                }
            }
        }
    }

    pub fn set_clipboard(&mut self, text: String) {
        self.clipboard = Some(text.clone());
        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(text);
        }
    }

    pub fn get_clipboard(&self) -> Option<String> {
        if let Ok(mut cb) = arboard::Clipboard::new() {
            if let Ok(text) = cb.get_text() {
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        self.clipboard.clone()
    }

    pub fn copy_selection(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let text = self.tabs[idx].editor.selected_text();
                if !text.is_empty() {
                    self.set_clipboard(text);
                    self.status_message = "Copied to clipboard".to_string();
                }
            }
        }
    }

    pub fn cut_selection(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let text = {
                    let tab = &mut self.tabs[idx];
                    let text = tab.editor.cut_selection();
                    if !text.is_empty() {
                        tab.is_dirty = true;
                        tab.version = tab.version.wrapping_add(1);
                        tab.shaping_cache.clear();
                    }
                    text
                };
                if !text.is_empty() {
                    self.set_clipboard(text);
                    self.status_message = "Cut to clipboard".to_string();
                }
            }
        }
    }

    pub fn paste_clipboard(&mut self) {
        let text = self.get_clipboard();
        if let Some(text) = text {
            if !text.is_empty() {
                if let Some(idx) = self.active_tab {
                    if idx < self.tabs.len() {
                        let tab = &mut self.tabs[idx];
                        tab.editor.paste(&text);
                        tab.is_dirty = true;
                        tab.version = tab.version.wrapping_add(1);
                        tab.shaping_cache.clear();
                        self.status_message = "Pasted from clipboard".to_string();
                    }
                }
            }
        }
    }

    pub fn undo_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let tab = &mut self.tabs[idx];
                if tab.editor.undo() {
                    tab.is_dirty = true;
                    tab.version = tab.version.wrapping_add(1);
                    tab.shaping_cache.clear();
                    self.status_message = "Undo".to_string();
                } else {
                    self.status_message = "Already at oldest change".to_string();
                }
            }
        }
    }

    pub fn redo_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let tab = &mut self.tabs[idx];
                if tab.editor.redo() {
                    tab.is_dirty = true;
                    tab.version = tab.version.wrapping_add(1);
                    tab.shaping_cache.clear();
                    self.status_message = "Redo".to_string();
                } else {
                    self.status_message = "Already at newest change".to_string();
                }
            }
        }
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if self.tabs.is_empty() {
                self.active_tab = None;
            } else if let Some(active) = self.active_tab {
                if active >= self.tabs.len() {
                    self.active_tab = Some(self.tabs.len() - 1);
                }
            }
            self.status_message = "Tab closed".to_string();
        }
    }

    pub fn new_untitled_file(&mut self) {
        let count = self.tabs.iter()
            .filter(|t| t.name.starts_with("Untitled"))
            .count() + 1;
        let name = format!("Untitled-{}", count);
        self.tabs.push(EditorTab {
            name,
            path: None,
            editor: Editor::new(Buffer::new()),
            is_dirty: false,
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            version: 0,
        });
        self.active_tab = Some(self.tabs.len() - 1);
        self.status_message = "New untitled file created".to_string();
    }

    pub fn format_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let tab = &mut self.tabs[idx];
                let path = tab.path.as_deref();
                let before = tab.editor.buffer.to_string();
                match self.formatter.format_editor(&mut tab.editor, path) {
                    Ok(_) => {
                        let after = tab.editor.buffer.to_string();
                        if before != after {
                            tab.is_dirty = true;
                            tab.version = tab.version.wrapping_add(1);
                            tab.shaping_cache.clear();
                            self.status_message = format!("Formatted {}", tab.name);
                        } else {
                            self.status_message = format!("{} is already formatted", tab.name);
                        }
                    }
                    Err(err) => {
                        self.status_message = err;
                    }
                }
            }
        }
    }
}
