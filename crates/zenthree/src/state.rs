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
    pub sidebar_visible: bool,
    pub sidebar_width: f32,
    pub active_resize_sidebar: bool,
    pub tabs: Vec<EditorTab>,
    pub active_tab: Option<usize>,
    pub current_dir: PathBuf,
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
            sidebar_visible: true,
            sidebar_width: 250.0,
            active_resize_sidebar: false,
            tabs: Vec::new(),
            active_tab: None,
            current_dir,
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
        };
        state.rebuild_tree();
        state
    }

    pub fn colors(&self) -> ThemeColors {
        ThemeColors::get(self.theme)
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
                let tab = &mut self.tabs[idx];
                if let Some(ref path) = tab.path {
                    if std::fs::write(path, tab.editor.buffer.to_string()).is_ok() {
                        tab.is_dirty = false;
                        self.status_message = format!("Saved file: {}", tab.name);
                    } else {
                        self.status_message = format!("Failed to save file: {}", tab.name);
                    }
                } else {
                    self.status_message = "Cannot save: File has no path yet".to_string();
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
}
