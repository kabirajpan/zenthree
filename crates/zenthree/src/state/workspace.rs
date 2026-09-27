use crate::state::types::*;
use crate::state::ZenthreeState;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

impl ZenthreeState {
    pub fn get_tree_root(&self) -> PathBuf {
        if self.sidebar_view == SidebarView::Notes {
            self.notes_state.get_vault_dir(&self.current_dir)
        } else {
            self.current_dir.clone()
        }
    }

    pub fn rebuild_tree(&mut self) {
        self.sidebar_items.clear();
        if !self.has_workspace && self.sidebar_view != SidebarView::Notes {
            return;
        }
        if self.sidebar_view == SidebarView::Notes {
            let _ = self.notes_state.ensure_vault_dir(&self.current_dir);
        }
        let root = self.get_tree_root();
        let git_statuses = if self.explorer_git_status && self.sidebar_view == SidebarView::Explorer {
            self.get_git_statuses(&root)
        } else {
            HashMap::new()
        };
        let mut ancestors = Vec::new();
        self.scan_dir_recursive(&root, 0, &mut ancestors, &git_statuses);
    }

    pub fn poll_file_tree_changes(&mut self) -> bool {
        if self.has_workspace && self.fs_watcher.take_pending() {
            self.rebuild_tree();
            true
        } else {
            false
        }
    }

    pub fn open_workspace(&mut self, path: PathBuf) {
        self.cancel_active_ai();

        // 1. Save prior session (general or previous project) under its own scope before changing directory
        self.save_current_ai_session();

        // 2. Start a fresh blank AI session for the newly opened project
        self.ai_messages.clear();
        self.ai_session_id = None;
        self.ai_session_json = None;
        self.ai_block_cache.clear();
        self.ai_input.clear();
        self.ai_sessions_visible = false;

        // 3. Completely clear all buffer tabs and close LSP documents to release RAM
        for tab in self.tabs.drain(..) {
            if let Some(ref p) = tab.path {
                self.lsp_manager.on_document_close(p);
            }
        }
        self.tabs.clear();
        self.set_active_tab(None);
        self.selected_sidebar_path = None;
        self.pending_reviews.clear();
        self.active_review_idx = None;

        // 4. Switch workspace
        self.current_dir = path;
        self.has_workspace = true;
        self.expanded_dirs.clear();
        self.rebuild_tree();
        self.refresh_git_status();
        self.fs_watcher.watch(&self.current_dir);
        self.set_status(format!("Opened workspace: {}", self.current_dir.display()));
        self.save_persistent_state();
        self.refresh_ai_sessions();
        self.notes_state.scan_notes(&self.current_dir);
    }

    pub fn close_workspace(&mut self) {
        self.cancel_active_ai();

        // 1. Save prior session under current project before leaving workspace
        self.save_current_ai_session();

        // 2. Start a fresh blank AI session for General mode
        self.ai_messages.clear();
        self.ai_session_id = None;
        self.ai_session_json = None;
        self.ai_block_cache.clear();
        self.ai_input.clear();
        self.ai_sessions_visible = false;

        // 3. Completely clear all buffer tabs and close LSP documents to release RAM
        for tab in self.tabs.drain(..) {
            if let Some(ref p) = tab.path {
                self.lsp_manager.on_document_close(p);
            }
        }
        self.tabs.clear();
        self.set_active_tab(None);
        self.selected_sidebar_path = None;
        self.pending_reviews.clear();
        self.active_review_idx = None;

        // 4. Close workspace
        self.fs_watcher.stop();
        self.has_workspace = false;
        self.current_dir = dirs::home_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        self.sidebar_items.clear();
        self.expanded_dirs.clear();
        self.set_status("Workspace closed");
        self.save_persistent_state();
        self.refresh_ai_sessions();
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.tabs.iter().any(|tab| tab.is_dirty)
    }

    pub fn unsaved_tab_names(&self) -> Vec<String> {
        self.tabs
            .iter()
            .filter(|tab| tab.is_dirty)
            .map(|tab| tab.name.clone())
            .collect()
    }

    pub fn request_open_workspace(&mut self, path: PathBuf) {
        if self.has_unsaved_changes() {
            self.unsaved_changes_dialog = Some(UnsavedChangesDialog {
                target_workspace: Some(path),
                unsaved_files: self.unsaved_tab_names(),
                just_opened: true,
            });
        } else {
            self.open_workspace(path);
        }
    }

    pub fn request_close_workspace(&mut self) {
        if self.has_unsaved_changes() {
            self.unsaved_changes_dialog = Some(UnsavedChangesDialog {
                target_workspace: None,
                unsaved_files: self.unsaved_tab_names(),
                just_opened: true,
            });
        } else {
            self.close_workspace();
        }
    }

    pub fn save_unsaved_and_proceed(&mut self) {
        for tab in &mut self.tabs {
            if tab.is_dirty {
                if let Some(ref path) = tab.path {
                    let _ = std::fs::write(path, tab.editor.buffer.to_string());
                    tab.is_dirty = false;
                }
            }
        }
        if let Some(dialog) = self.unsaved_changes_dialog.take() {
            if let Some(target) = dialog.target_workspace {
                self.open_workspace(target);
            } else {
                self.close_workspace();
            }
        }
    }

    pub fn discard_unsaved_and_proceed(&mut self) {
        if let Some(dialog) = self.unsaved_changes_dialog.take() {
            if let Some(target) = dialog.target_workspace {
                self.open_workspace(target);
            } else {
                self.close_workspace();
            }
        }
    }

    pub fn cancel_unsaved_dialog(&mut self) {
        self.unsaved_changes_dialog = None;
    }

    fn get_git_statuses(&self, root: &Path) -> HashMap<PathBuf, GitStatus> {
        let mut map = HashMap::new();
        if let Ok(output) = std::process::Command::new("git")
            .args(["status", "--porcelain=v1", "-uall", "."])
            .current_dir(root)
            .output()
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if line.len() < 4 {
                        continue;
                    }
                    let status_code = &line[0..2];
                    let file_rel = line[3..].trim();
                    let file_clean = if let Some((_, to)) = file_rel.split_once(" -> ") {
                        to.trim()
                    } else {
                        file_rel
                    };
                    let file_clean = file_clean.trim_matches('"');
                    let full_path = root.join(file_clean);
                    let status = if status_code.contains('?') {
                        GitStatus::Untracked
                    } else if status_code.contains('A') {
                        GitStatus::Added
                    } else if status_code.contains('D') {
                        GitStatus::Deleted
                    } else if status_code.contains('R') {
                        GitStatus::Renamed
                    } else if status_code.contains('U') || status_code == "AA" || status_code == "DD" {
                        GitStatus::Conflicted
                    } else {
                        GitStatus::Modified
                    };

                    map.insert(full_path.clone(), status);
                    if let Ok(canon) = std::fs::canonicalize(&full_path) {
                        if canon != full_path {
                            map.insert(canon, status);
                        }
                    }

                    let mut current = full_path.parent();
                    while let Some(parent) = current {
                        if parent == root || !parent.starts_with(root) {
                            break;
                        }
                        map.entry(parent.to_path_buf())
                            .or_insert(GitStatus::Modified);
                        if let Ok(canon_parent) = std::fs::canonicalize(parent) {
                            if canon_parent != parent {
                                map.entry(canon_parent).or_insert(GitStatus::Modified);
                            }
                        }
                        current = parent.parent();
                    }
                }
            }
        }
        map
    }

    fn scan_dir_recursive(
        &mut self,
        dir: &Path,
        depth: usize,
        ancestor_is_last: &mut Vec<bool>,
        git_statuses: &HashMap<PathBuf, GitStatus>,
    ) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
            // Sort: folders first, then files alphabetically (pinned notes first in Notes view)
            paths.sort_by(|a, b| {
                let a_is_dir = a.is_dir();
                let b_is_dir = b.is_dir();
                if a_is_dir != b_is_dir {
                    b_is_dir.cmp(&a_is_dir)
                } else {
                    if self.sidebar_view == SidebarView::Notes {
                        let a_pinned = self.notes_state.pinned_paths.contains(a);
                        let b_pinned = self.notes_state.pinned_paths.contains(b);
                        if a_pinned != b_pinned {
                            return b_pinned.cmp(&a_pinned);
                        }
                    }
                    a.file_name().cmp(&b.file_name())
                }
            });

            // Filter out hidden files unless toggled on
            paths.retain(|p| {
                let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if self.explorer_show_hidden {
                    name != "." && name != ".."
                } else {
                    !name.starts_with('.') || name == ".gitignore" || name == ".github"
                }
            });

            let total = paths.len();
            for (i, path) in paths.into_iter().enumerate() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                let is_dir = path.is_dir();
                let is_expanded = is_dir && self.expanded_dirs.contains(&path);
                let is_last = i + 1 == total;

                let mut guide_prefix = String::new();
                for &last in ancestor_is_last.iter() {
                    if last {
                        guide_prefix.push_str("   ");
                    } else {
                        guide_prefix.push_str("│  ");
                    }
                }
                if is_last {
                    guide_prefix.push_str("└─ ");
                } else {
                    guide_prefix.push_str("├─ ");
                }

                let git_status = git_statuses.get(&path).cloned();

                self.sidebar_items.push(FileTreeNode {
                    path: path.clone(),
                    name,
                    depth,
                    is_dir,
                    is_expanded,
                    is_last,
                    guide_prefix,
                    git_status,
                });

                if is_expanded {
                    ancestor_is_last.push(is_last);
                    self.scan_dir_recursive(&path, depth + 1, ancestor_is_last, git_statuses);
                    ancestor_is_last.pop();
                }
            }
        }
    }

    pub fn collapse_all_folders(&mut self) {
        self.expanded_dirs.clear();
        self.rebuild_tree();
        self.save_persistent_state();
    }

    pub fn expand_all_folders(&mut self) {
        fn collect_dirs(
            dir: &Path,
            acc: &mut std::collections::HashSet<PathBuf>,
            show_hidden: bool,
        ) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.filter_map(Result::ok) {
                    let path = entry.path();
                    if path.is_dir() {
                        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        let allow = if show_hidden {
                            name != "." && name != ".."
                        } else {
                            !name.starts_with('.') || name == ".github"
                        };
                        if allow {
                            acc.insert(path.clone());
                            collect_dirs(&path, acc, show_hidden);
                        }
                    }
                }
            }
        }
        collect_dirs(
            &self.current_dir,
            &mut self.expanded_dirs,
            self.explorer_show_hidden,
        );
        self.rebuild_tree();
        self.save_persistent_state();
    }

    #[allow(dead_code)]
    pub fn new_file_in_workspace(&mut self) {
        if !self.has_workspace {
            self.new_untitled_file();
            return;
        }
        self.start_new_file_inline(None);
    }

    pub fn start_new_file_inline(&mut self, target_dir: Option<PathBuf>) {
        if !self.has_workspace && self.sidebar_view != SidebarView::Notes {
            self.new_untitled_file();
            return;
        }
        if self.sidebar_view == SidebarView::Notes {
            let _ = self.notes_state.ensure_vault_dir(&self.current_dir);
        }
        let root = self.get_tree_root();
        let dir = target_dir.unwrap_or_else(|| {
            if let Some(ref selected) = self.selected_sidebar_path {
                if selected.is_dir() {
                    selected.clone()
                } else {
                    selected.parent().unwrap_or(&root).to_path_buf()
                }
            } else {
                root.clone()
            }
        });
        if dir != root {
            self.expanded_dirs.insert(dir.clone());
            self.rebuild_tree();
        }
        self.inline_tree_input = Some(InlineTreeState {
            mode: InlineTreeInputMode::NewFile { parent_dir: dir },
            text: String::new(),
        });
        self.tree_context_menu = None;
    }

    pub fn start_new_folder_inline(&mut self, target_dir: Option<PathBuf>) {
        if !self.has_workspace && self.sidebar_view != SidebarView::Notes {
            return;
        }
        if self.sidebar_view == SidebarView::Notes {
            let _ = self.notes_state.ensure_vault_dir(&self.current_dir);
        }
        let root = self.get_tree_root();
        let dir = target_dir.unwrap_or_else(|| {
            if let Some(ref selected) = self.selected_sidebar_path {
                if selected.is_dir() {
                    selected.clone()
                } else {
                    selected.parent().unwrap_or(&root).to_path_buf()
                }
            } else {
                root.clone()
            }
        });
        if dir != root {
            self.expanded_dirs.insert(dir.clone());
            self.rebuild_tree();
        }
        self.inline_tree_input = Some(InlineTreeState {
            mode: InlineTreeInputMode::NewFolder { parent_dir: dir },
            text: String::new(),
        });
        self.tree_context_menu = None;
    }

    pub fn start_rename_inline(&mut self, target_path: PathBuf) {
        let mut initial_name = target_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        if self.sidebar_view == SidebarView::Notes && !target_path.is_dir() {
            if let Some(s) = initial_name.strip_suffix(".md") {
                initial_name = s.to_string();
            } else if let Some(s) = initial_name.strip_suffix(".markdown") {
                initial_name = s.to_string();
            }
        }
        self.inline_tree_input = Some(InlineTreeState {
            mode: InlineTreeInputMode::Rename { target_path },
            text: initial_name,
        });
        self.tree_context_menu = None;
    }

    pub fn commit_inline_tree_input(&mut self) -> bool {
        let inline = match self.inline_tree_input.take() {
            Some(i) => i,
            None => return false,
        };

        let trimmed = inline.text.trim();
        if trimmed.is_empty() {
            return false;
        }

        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
            self.set_status("Invalid characters in filename");
            self.inline_tree_input = Some(inline);
            return false;
        }

        match inline.mode {
            InlineTreeInputMode::NewFile { ref parent_dir } => {
                let mut filename = trimmed.to_string();
                if self.sidebar_view == SidebarView::Notes && !filename.ends_with(".md") && !filename.ends_with(".markdown") {
                    filename.push_str(".md");
                }
                let full_path = parent_dir.join(&filename);
                if full_path.exists() {
                    self.set_status(format!("File '{}' already exists", filename));
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                if let Some(parent) = full_path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                if let Err(e) = std::fs::write(&full_path, "") {
                    self.set_status(format!("Failed to create file: {}", e));
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                self.rebuild_tree();
                self.open_file(&full_path);
                self.set_status(format!("Created {}", filename));
                true
            }
            InlineTreeInputMode::NewFolder { ref parent_dir } => {
                let full_path = parent_dir.join(trimmed);
                if full_path.exists() {
                    self.set_status(format!("Folder '{}' already exists", trimmed));
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                if let Err(e) = std::fs::create_dir_all(&full_path) {
                    self.set_status(format!("Failed to create folder: {}", e));
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                self.expanded_dirs.insert(full_path.clone());
                self.selected_sidebar_path = Some(full_path);
                self.rebuild_tree();
                self.set_status(format!("Created folder {}", trimmed));
                true
            }
            InlineTreeInputMode::Rename { ref target_path } => {
                let mut filename = trimmed.to_string();
                if self.sidebar_view == SidebarView::Notes && !target_path.is_dir() && !filename.ends_with(".md") && !filename.ends_with(".markdown") {
                    filename.push_str(".md");
                }
                let parent = target_path.parent().unwrap_or(&self.current_dir);
                let new_path = parent.join(&filename);
                if new_path == *target_path {
                    return true;
                }
                if new_path.exists() {
                    self.set_status(format!("'{}' already exists", filename));
                    self.inline_tree_input = Some(inline);
                    return false;
                }
                if let Err(e) = std::fs::rename(target_path, &new_path) {
                    self.set_status(format!("Failed to rename: {}", e));
                    self.inline_tree_input = Some(inline);
                    return false;
                }

                // Update open tabs if file was renamed, or if it was a folder containing open files
                for tab in &mut self.tabs {
                    if let Some(ref mut tab_path) = tab.path {
                        if *tab_path == *target_path {
                            *tab_path = new_path.clone();
                            tab.name = filename.clone();
                        } else if let Ok(rel) = tab_path.strip_prefix(target_path) {
                            *tab_path = new_path.join(rel);
                        }
                    }
                }

                if self.selected_sidebar_path.as_ref() == Some(target_path) {
                    self.selected_sidebar_path = Some(new_path.clone());
                }

                if self.expanded_dirs.remove(target_path) {
                    self.expanded_dirs.insert(new_path.clone());
                }

                self.rebuild_tree();
                self.set_status(format!("Renamed to {}", trimmed));
                true
            }
        }
    }

    pub fn cancel_inline_tree_input(&mut self) {
        self.inline_tree_input = None;
    }

    pub fn delete_tree_path(&mut self, path: &Path) {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("item")
            .to_string();
        let res = if path.is_dir() {
            std::fs::remove_dir_all(path)
        } else {
            std::fs::remove_file(path)
        };

        match res {
            Ok(_) => {
                let mut i = 0;
                while i < self.tabs.len() {
                    let should_close = self.tabs[i]
                        .path
                        .as_ref()
                        .map(|p| p == path || p.starts_with(path))
                        .unwrap_or(false);
                    if should_close {
                        self.close_tab(i);
                    } else {
                        i += 1;
                    }
                }

                if self.selected_sidebar_path.as_ref() == Some(&path.to_path_buf()) {
                    self.selected_sidebar_path = None;
                }
                self.expanded_dirs.remove(path);
                self.rebuild_tree();
                self.set_status(format!("Deleted {}", name));
            }
            Err(e) => {
                self.set_status(format!("Failed to delete: {}", e));
            }
        }
        self.tree_context_menu = None;
    }

    pub fn copy_tree_path(&mut self, path: &Path) {
        self.tree_clipboard = Some(TreeClipboardAction::Copy(path.to_path_buf()));
        self.set_status(format!(
            "Copied {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("")
        ));
        self.tree_context_menu = None;
    }

    pub fn cut_tree_path(&mut self, path: &Path) {
        self.tree_clipboard = Some(TreeClipboardAction::Cut(path.to_path_buf()));
        self.set_status(format!(
            "Cut {}",
            path.file_name().and_then(|n| n.to_str()).unwrap_or("")
        ));
        self.tree_context_menu = None;
    }

    pub fn paste_into_tree_path(&mut self, dest_dir: &Path) {
        let clip = match self.tree_clipboard.take() {
            Some(c) => c,
            None => return,
        };

        let target_dir = if dest_dir.is_dir() {
            dest_dir.to_path_buf()
        } else {
            dest_dir.parent().unwrap_or(&self.current_dir).to_path_buf()
        };

        match clip {
            TreeClipboardAction::Copy(src) => {
                if src.exists() {
                    let file_name = src.file_name().unwrap_or_default();
                    let mut dest = target_dir.join(file_name);

                    if dest.exists() {
                        let stem = src.file_stem().and_then(|s| s.to_str()).unwrap_or("copy");
                        let ext = src
                            .extension()
                            .and_then(|e| e.to_str())
                            .map(|e| format!(".{}", e))
                            .unwrap_or_default();
                        let mut count = 1;
                        loop {
                            let candidate =
                                target_dir.join(format!("{}_copy{}{}", stem, count, ext));
                            if !candidate.exists() {
                                dest = candidate;
                                break;
                            }
                            count += 1;
                        }
                    }

                    let res = if src.is_dir() {
                        copy_dir_all(&src, &dest)
                    } else {
                        std::fs::copy(&src, &dest).map(|_| ())
                    };

                    if res.is_ok() {
                        self.rebuild_tree();
                        self.set_status(format!(
                            "Pasted into {}",
                            target_dir
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                        ));
                    } else {
                        self.set_status("Failed to paste item");
                    }
                }
                self.tree_clipboard = Some(TreeClipboardAction::Copy(src));
            }
            TreeClipboardAction::Cut(src) => {
                if src.exists() {
                    let file_name = src.file_name().unwrap_or_default();
                    let dest = target_dir.join(file_name);
                    if !dest.exists() {
                        if std::fs::rename(&src, &dest).is_ok() {
                            for tab in &mut self.tabs {
                                if let Some(ref mut tab_path) = tab.path {
                                    if *tab_path == src {
                                        *tab_path = dest.clone();
                                    } else if let Ok(rel) = tab_path.strip_prefix(&src) {
                                        *tab_path = dest.join(rel);
                                    }
                                }
                            }
                            self.rebuild_tree();
                            self.set_status(format!(
                                "Moved to {}",
                                target_dir
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                            ));
                        }
                    } else {
                        self.set_status("Destination already exists");
                    }
                }
            }
        }
        self.tree_context_menu = None;
    }

    pub fn reveal_in_file_manager(&self, path: &Path) {
        let dir = if path.is_dir() {
            path
        } else {
            path.parent().unwrap_or(&self.current_dir)
        };
        #[cfg(target_os = "windows")]
        {
            let _ = std::process::Command::new("explorer").arg(dir).spawn();
        }
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(path)
                .spawn();
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let _ = std::process::Command::new("xdg-open").arg(dir).spawn();
        }
    }
}

pub(crate) fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            std::fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }
    Ok(())
}
