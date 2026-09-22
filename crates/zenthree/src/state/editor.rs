use crate::state::types::*;
use crate::state::{ZenthreeState, path_matches};
use ::editor::buffer::Buffer;
use ::editor::editor::Editor;
use std::path::Path;

impl ZenthreeState {
    pub fn open_file(&mut self, path: &Path) {
        self.selected_sidebar_path = Some(path.to_path_buf());
        let pending = self.pending_reviews.iter().find(|r| path_matches(&r.path, path)).cloned();

        let disk_content = std::fs::read_to_string(path).ok();
        let content = if let Some(ref r) = pending {
            r.diff_lines.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n")
        } else if let Some(c) = disk_content {
            c
        } else {
            self.set_status(format!("Failed to open file: {}", path.display()));
            return;
        };

        let (lines_added, lines_removed) = if let Some(ref r) = pending {
            (r.total_added_lines(), r.total_removed_lines())
        } else {
            (0, 0)
        };

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
        for (idx, tab) in self.tabs.iter_mut().enumerate() {
            let is_match = if let Some(ref tp) = tab.path {
                path_matches(tp, path)
            } else {
                tab.name == name
            };
            if is_match {
                let name_clone = name.clone();
                // Drop tab borrow before calling set_active_tab (needs &mut self)
                let _ = tab;
                self.set_active_tab(Some(idx));
                // Re-borrow to apply pending review
                if let Some(ref r) = pending {
                    let tab = &mut self.tabs[idx];
                    tab.lines_added = r.total_added_lines();
                    tab.lines_removed = r.total_removed_lines();
                    let preview_text = r.diff_lines.iter().map(|(l, _)| l.as_str()).collect::<Vec<_>>().join("\n");
                    tab.editor = Editor::new(Buffer::from_text(&preview_text));
                    tab.shaping_cache.clear();
                    tab.is_dirty = true;
                }
                self.sync_active_review_for_tab(idx);
                self.set_status(format!("Switched to {}", name_clone));
                self.check_language_support_prompt(path);
                self.save_persistent_state();
                return;
            }
        }

        self.tabs.push(EditorTab {
            name,
            path: Some(path.to_path_buf()),
            editor: Editor::new(Buffer::from_text(&content)),
            is_dirty: pending.is_some(),
            scroll_y: 0.0,
            shaping_cache: std::collections::HashMap::new(),
            version: 0,
            lines_added,
            lines_removed,
        });
        let new_tab_idx = self.tabs.len() - 1;
        self.set_active_tab(Some(new_tab_idx));
        self.sync_active_review_for_tab(new_tab_idx);
        self.set_status(format!("Opened file: {}", path.display()));

            let ws_root = if self.has_workspace {
                Some(self.current_dir.as_path())
            } else {
                None
            };
            self.lsp_manager.on_document_open(path, &content, ws_root);
            self.check_language_support_prompt(path);
            self.save_persistent_state();
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
            self.set_status(format!("Installed {} successfully!", ext.name));
        } else {
            self.set_status(format!("Installed extension: {}", id));
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
            self.set_status(format!("Uninstalled {}", ext.name));
        } else {
            self.set_status(format!("Uninstalled extension: {}", id));
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
                // Extract name + write result before borrowing self for set_status
                let (ok, msg) = {
                    let tab = &mut self.tabs[idx];
                    if let Some(ref path) = tab.path {
                        if std::fs::write(path, tab.editor.buffer.to_string()).is_ok() {
                            tab.is_dirty = false;
                            (true, format!("Saved file: {}", tab.name))
                        } else {
                            (false, format!("Failed to save file: {}", tab.name))
                        }
                    } else {
                        return;
                    }
                };
                let _ = ok; // suppress unused warning
                self.set_status(msg);
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
                    let msg = {
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
                            format!("Saved as: {}", tab.name)
                        } else {
                            format!("Failed to save file: {}", path.display())
                        }
                    };
                    self.set_status(msg);
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
                    self.set_status("Copied to clipboard");
                }
            }
        }
    }

    pub fn cut_selection(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let text = self.tabs[idx].editor.cut_selection();
                if !text.is_empty() {
                    self.on_buffer_changed(idx);
                    self.set_clipboard(text);
                    self.set_status("Cut to clipboard");
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
                        self.tabs[idx].editor.paste(&text);
                        self.on_buffer_changed(idx);
                        self.set_status("Pasted from clipboard");
                    }
                }
            }
        }
    }

    pub fn undo_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let did_undo = self.tabs[idx].editor.undo();
                if did_undo {
                    self.on_buffer_changed(idx);
                    self.set_status("Undo");
                } else {
                    self.set_status("Already at oldest change");
                }
            }
        }
    }

    pub fn redo_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let did_redo = self.tabs[idx].editor.redo();
                if did_redo {
                    self.on_buffer_changed(idx);
                    self.set_status("Redo");
                } else {
                    self.set_status("Already at newest change");
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
                self.set_active_tab(None);
                self.selected_sidebar_path = None;
            } else if let Some(active) = self.active_tab {
                let new_active = if active >= self.tabs.len() {
                    self.tabs.len() - 1
                } else {
                    active
                };
                self.set_active_tab(Some(new_active));
                if let Some(ref path) = self.tabs[new_active].path {
                    self.selected_sidebar_path = Some(path.clone());
                }
            }
            self.set_status("Tab closed");
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
            lines_added: 0,
            lines_removed: 0,
        });
        self.set_active_tab(Some(self.tabs.len() - 1));
        self.set_status("New untitled file created");
    }

    pub fn format_active_tab(&mut self) {
        if let Some(idx) = self.active_tab {
            if idx < self.tabs.len() {
                let path = self.tabs[idx].path.clone();
                let before = self.tabs[idx].editor.buffer.to_string();
                let msg = match self.formatter.format_editor(&mut self.tabs[idx].editor, path.as_deref()) {
                    Ok(_) => {
                        let after = self.tabs[idx].editor.buffer.to_string();
                        if before != after {
                            self.on_buffer_changed(idx);
                            format!("Formatted {}", self.tabs[idx].name)
                        } else {
                            format!("{} is already formatted", self.tabs[idx].name)
                        }
                    }
                    Err(err) => err,
                };
                self.set_status(msg);
            }
        }
    }
}
