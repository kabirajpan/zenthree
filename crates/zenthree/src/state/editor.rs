use crate::state::types::*;
use crate::state::ZenthreeState;
use ::editor::buffer::Buffer;
use ::editor::editor::Editor;
use std::path::Path;

impl ZenthreeState {
    pub fn open_file(&mut self, path: &Path) {
        self.selected_sidebar_path = Some(path.to_path_buf());
        if let Ok(content) = std::fs::read_to_string(path) {
            let name = path
                .file_name()
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
                    self.check_language_support_prompt(path);
                    self.save_persistent_state();
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
            let ws_root = if self.has_workspace {
                Some(self.current_dir.as_path())
            } else {
                None
            };
            self.lsp_manager.on_document_open(path, &content, ws_root);
            self.check_language_support_prompt(path);
            self.save_persistent_state();
        } else {
            self.status_message = format!("Failed to open file: {}", path.display());
        }
    }

    pub fn check_language_support_prompt(&mut self, path: &Path) {
        if let Some(ext) = crate::extensions::ExtensionCatalog::find_by_path(path) {
            // Check if installed or explicitly ignored
            if !self.installed_extensions.contains(ext.id)
                && !self.ignored_language_prompts.contains(ext.id)
            {
                let file_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("file")
                    .to_string();
                self.active_language_prompt = Some(LanguagePromptDialog {
                    extension_id: ext.id.to_string(),
                    language_name: ext.name.to_string(),
                    file_name,
                    dont_show_again_checked: false,
                });
            }
        }
    }

    pub fn install_extension(&mut self, id: &str) {
        self.installed_extensions.insert(id.to_string());
        self.highlighter.reset();
        self.save_persistent_state();
        if let Some(ext) = crate::extensions::ExtensionCatalog::find_by_id(id) {
            self.status_message = format!("Installed {} successfully!", ext.name);
        } else {
            self.status_message = format!("Installed extension: {}", id);
        }
        if let Some(ref prompt) = self.active_language_prompt {
            if prompt.extension_id == id {
                self.active_language_prompt = None;
            }
        }
    }

    pub fn uninstall_extension(&mut self, id: &str) {
        self.installed_extensions.remove(id);
        self.highlighter.reset();
        self.save_persistent_state();
        if let Some(ext) = crate::extensions::ExtensionCatalog::find_by_id(id) {
            self.status_message = format!("Uninstalled {}", ext.name);
        } else {
            self.status_message = format!("Uninstalled extension: {}", id);
        }
    }

    pub fn is_extension_installed(&self, id: &str) -> bool {
        self.installed_extensions.contains(id)
    }

    pub fn dismiss_language_prompt(&mut self, save_dont_show_again: bool) {
        if let Some(ref dialog) = self.active_language_prompt {
            if save_dont_show_again || dialog.dont_show_again_checked {
                self.ignored_language_prompts
                    .insert(dialog.extension_id.clone());
                self.save_persistent_state();
            }
        }
        self.active_language_prompt = None;
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
                let initial_dir = self.tabs[idx]
                    .path
                    .as_ref()
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
                        tab.name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Unknown")
                            .to_string();
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
            let removed = self.tabs.remove(idx);
            if let Some(ref path) = removed.path {
                self.lsp_manager.on_document_close(path);
            }
            if self.tabs.is_empty() {
                self.active_tab = None;
                self.selected_sidebar_path = None;
            } else if let Some(active) = self.active_tab {
                let new_active = if active >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    active
                };
                self.active_tab = Some(new_active);
                if let Some(ref path) = self.tabs[new_active].path {
                    self.selected_sidebar_path = Some(path.clone());
                }
            }
            self.status_message = "Tab closed".to_string();
            self.save_persistent_state();
        }
    }

    pub fn new_untitled_file(&mut self) {
        let count = self
            .tabs
            .iter()
            .filter(|t| t.name.starts_with("Untitled"))
            .count()
            + 1;
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
