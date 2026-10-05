//! Keyboard shortcut handling for Zenthree IDE.
//!
//! All shortcuts are documented in `docs/shortcuts.md`.
//!
//! ## Panel shortcuts (smart toggle)
//! - `Ctrl+E`       → Explorer (file tree)
//! - `Ctrl+T`       → Terminal
//! - `Ctrl+Shift+G` → Git panel
//! - `Ctrl+Shift+A` → AI panel
//! - `F11`          → Focus mode
//!
//! ## Tab navigation
//! - `Alt+←` / `Alt+→` → prev / next tab
//! - `Alt+W`            → close current tab
//!
//! ## Focus
//! - `Ctrl+1` → jump to editor
//! - `Escape` → back to editor (panel stays open)

use crate::state::{types::FocusedPanel, SidebarView, ZenthreeState};
use zenthra_platform::event::PlatformEvent;
use zenthra_widgets::ui::Ui;

/// Process all keyboard events for the current frame.
/// Call this once per frame before drawing UI.
pub fn handle_shortcuts(ui: &mut Ui, state: &mut ZenthreeState) {
    // ── Update Modifiers from KeyDown / KeyUp ──────────────────────────
    for event in &ui.input_events {
        match event {
            PlatformEvent::KeyDown { key } => match key {
                winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => state.shift_pressed = true,
                winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => state.ctrl_pressed = true,
                winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => state.alt_pressed = true,
                _ => {}
            },
            PlatformEvent::KeyUp { key } => match key {
                winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => state.shift_pressed = false,
                winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => state.ctrl_pressed = false,
                winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => state.alt_pressed = false,
                _ => {}
            },
            _ => {}
        }
    }

    // ── Exclusive focus for LazyVim Command Line dialog ───────────────
    if state.cmdline_dialog.open {
        let mut dismiss = false;
        ui.input_events.retain(|event| {
            if let PlatformEvent::KeyDown { key } = event {
                if *key == winit::keyboard::KeyCode::Escape
                    || (*key == winit::keyboard::KeyCode::KeyC && state.ctrl_pressed)
                {
                    dismiss = true;
                    return false; // Consume Escape & Ctrl+C completely!
                }
            }
            true
        });

        if dismiss {
            state.close_cmdline();
            ui.needs_redraw = true;
        }
        return;
    }

    // ── Exclusive focus for Delete Confirmation Modal ─────────────────
    if state.delete_confirm_dialog.open {
        let mut dismiss = false;
        ui.input_events.retain(|event| {
            if let PlatformEvent::KeyDown { key } = event {
                if *key == winit::keyboard::KeyCode::Escape
                    || (*key == winit::keyboard::KeyCode::KeyC && state.ctrl_pressed)
                {
                    dismiss = true;
                    return false;
                }
            }
            true
        });

        if dismiss {
            state.close_delete_confirm();
            ui.needs_redraw = true;
        }
        return;
    }

    // ── Exclusive focus for Rename Modal ──────────────────────────────
    if state.rename_modal.open {
        let mut do_execute = false;
        let mut do_dismiss = false;

        ui.input_events.retain(|event| {
            if let PlatformEvent::KeyDown { key } = event {
                match key {
                    winit::keyboard::KeyCode::Escape => {
                        do_dismiss = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyC if state.ctrl_pressed => {
                        do_dismiss = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        do_execute = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        if state.rename_modal.cursor_idx > 0 && !state.rename_modal.input.is_empty() {
                            state.rename_modal.input.remove(state.rename_modal.cursor_idx - 1);
                            state.rename_modal.cursor_idx -= 1;
                            state.rename_modal.error_msg = None;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Delete => {
                        if state.rename_modal.cursor_idx < state.rename_modal.input.len() {
                            state.rename_modal.input.remove(state.rename_modal.cursor_idx);
                            state.rename_modal.error_msg = None;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowLeft => {
                        if state.rename_modal.cursor_idx > 0 {
                            state.rename_modal.cursor_idx -= 1;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowRight => {
                        if state.rename_modal.cursor_idx < state.rename_modal.input.len() {
                            state.rename_modal.cursor_idx += 1;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Home => {
                        state.rename_modal.cursor_idx = 0;
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::End => {
                        state.rename_modal.cursor_idx = state.rename_modal.input.len();
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyV if state.ctrl_pressed => {
                        if let Some(text) = state.get_clipboard() {
                            let clean = text.replace('\n', "").replace('\r', "");
                            state.rename_modal.input.insert_str(state.rename_modal.cursor_idx, &clean);
                            state.rename_modal.cursor_idx += clean.len();
                            state.rename_modal.error_msg = None;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyU if state.ctrl_pressed => {
                        state.rename_modal.input.clear();
                        state.rename_modal.cursor_idx = 0;
                        state.rename_modal.error_msg = None;
                        ui.needs_redraw = true;
                        return false;
                    }
                    _ => {}
                }
            }
            if let PlatformEvent::CharTyped(c) = event {
                if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                    state.rename_modal.input.insert(state.rename_modal.cursor_idx, *c);
                    state.rename_modal.cursor_idx += c.len_utf8();
                    state.rename_modal.error_msg = None;
                    ui.needs_redraw = true;
                    return false;
                }
            }
            true
        });

        if do_dismiss {
            state.close_rename_modal();
            ui.needs_redraw = true;
        } else if do_execute {
            if let Err(e) = state.execute_rename() {
                state.rename_modal.error_msg = Some(e);
                ui.needs_redraw = true;
            } else {
                ui.needs_redraw = true;
            }
        }
        return;
    }

    for event in &ui.input_events.clone() {
        match event {
            // ── Modifier tracking ─────────────────────────────────────────
            PlatformEvent::KeyDown { key } => {
                match key {
                    winit::keyboard::KeyCode::ShiftLeft
                    | winit::keyboard::KeyCode::ShiftRight => {
                        state.shift_pressed = true;
                    }
                    winit::keyboard::KeyCode::ControlLeft
                    | winit::keyboard::KeyCode::ControlRight => {
                        state.ctrl_pressed = true;
                    }
                    winit::keyboard::KeyCode::AltLeft
                    | winit::keyboard::KeyCode::AltRight => {
                        state.alt_pressed = true;
                    }

                    // ── Settings ──────────────────────────────────────────
                    winit::keyboard::KeyCode::Comma if state.ctrl_pressed => {
                        state.editor_settings_open = !state.editor_settings_open;
                        state.editor_settings_just_opened = true;
                        if state.editor_settings_open {
                            state.editor_settings_tab =
                                crate::state::EditorSettingsTab::Appearance;
                        }
                    }

                    // ── F11 → Focus mode ──────────────────────────────────
                    winit::keyboard::KeyCode::F11 => {
                        state.focus_mode = !state.focus_mode;
                        if state.focus_mode {
                            state.activity_bar_visible = false;
                            state.terminal_visible = false;
                            state.ai_panel_visible = false;
                        } else {
                            state.activity_bar_visible = true;
                        }
                        ui.request_redraw();
                    }

                    // ── Ctrl+E → Explorer (smart toggle + focus) ──────────
                    winit::keyboard::KeyCode::KeyE if state.ctrl_pressed => {
                        if state.focused_panel == FocusedPanel::Explorer {
                            // Already focused → close
                            state.sidebar_visible = false;
                            state.focused_panel = FocusedPanel::Editor;
                            ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                        } else if state.sidebar_visible
                            && state.sidebar_view == SidebarView::Explorer
                        {
                            // Visible but not focused → focus it
                            state.focused_panel = FocusedPanel::Explorer;
                            ui.focused_id = None;
                            if state.selected_sidebar_path.is_none() && !state.sidebar_items.is_empty() {
                                let active_path = state.active_tab.and_then(|idx| state.tabs.get(idx)).and_then(|t| t.path.clone());
                                state.selected_sidebar_path = active_path.or_else(|| state.sidebar_items.first().map(|n| n.path.clone()));
                            }
                        } else {
                            // Hidden or wrong view → show + focus
                            state.sidebar_visible = true;
                            state.sidebar_view = SidebarView::Explorer;
                            state.focused_panel = FocusedPanel::Explorer;
                            ui.focused_id = None;
                            if state.selected_sidebar_path.is_none() && !state.sidebar_items.is_empty() {
                                let active_path = state.active_tab.and_then(|idx| state.tabs.get(idx)).and_then(|t| t.path.clone());
                                state.selected_sidebar_path = active_path.or_else(|| state.sidebar_items.first().map(|n| n.path.clone()));
                            }
                        }
                        ui.request_redraw();
                    }

                    // ── Ctrl+T → Terminal (smart toggle + focus) ──────────
                    winit::keyboard::KeyCode::KeyT if state.ctrl_pressed => {
                        if state.focused_panel == FocusedPanel::Terminal {
                            // Already focused → hide
                            state.terminal_visible = false;
                            state.focused_panel = FocusedPanel::Editor;
                            ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                        } else if state.terminal_visible {
                            // Visible but not focused → focus it
                            state.focused_panel = FocusedPanel::Terminal;
                            state.last_cursor_activity = std::time::Instant::now();
                            ui.focused_id = Some(zenthra::Id::from_u64(999999800));
                        } else {
                            // Hidden → show + focus
                            state.terminal_visible = true;
                            state.focused_panel = FocusedPanel::Terminal;
                            state.last_cursor_activity = std::time::Instant::now();
                            ui.focused_id = Some(zenthra::Id::from_u64(999999800));
                        }
                        ui.request_redraw();
                    }

                    // ── Ctrl+Shift+G → Git panel (smart toggle + focus) ───
                    winit::keyboard::KeyCode::KeyG
                        if state.ctrl_pressed && state.shift_pressed =>
                    {
                        if state.focused_panel == FocusedPanel::Git {
                            // Already focused → hide
                            state.sidebar_visible = false;
                            state.focused_panel = FocusedPanel::Editor;
                            ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                        } else if state.sidebar_visible
                            && state.sidebar_view == SidebarView::Git
                        {
                            state.focused_panel = FocusedPanel::Git;
                            ui.focused_id = None;
                        } else {
                            state.sidebar_visible = true;
                            state.sidebar_view = SidebarView::Git;
                            state.focused_panel = FocusedPanel::Git;
                            ui.focused_id = None;
                        }
                        ui.request_redraw();
                    }

                    // ── Ctrl+Shift+A → AI panel (smart toggle + focus) ───
                    winit::keyboard::KeyCode::KeyA
                        if state.ctrl_pressed && state.shift_pressed =>
                    {
                        if state.focused_panel == FocusedPanel::Ai {
                            state.ai_panel_visible = false;
                            state.focused_panel = FocusedPanel::Editor;
                            ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                        } else if state.ai_panel_visible {
                            state.focused_panel = FocusedPanel::Ai;
                            ui.focused_id = None;
                        } else {
                            state.ai_panel_visible = true;
                            state.focused_panel = FocusedPanel::Ai;
                            ui.focused_id = None;
                        }
                        ui.request_redraw();
                    }

                    // ── Zoom In: Ctrl+= or Ctrl++ ──────────────────────────
                    winit::keyboard::KeyCode::Equal | winit::keyboard::KeyCode::NumpadAdd
                        if state.ctrl_pressed =>
                    {
                        state.zoom_in();
                        state.set_status(format!("Zoom: {:.0}%", state.zoom_level * 100.0));
                        ui.request_redraw();
                    }

                    // ── Zoom Out: Ctrl+- ──────────────────────────────────
                    winit::keyboard::KeyCode::Minus | winit::keyboard::KeyCode::NumpadSubtract
                        if state.ctrl_pressed =>
                    {
                        state.zoom_out();
                        state.set_status(format!("Zoom: {:.0}%", state.zoom_level * 100.0));
                        ui.request_redraw();
                    }

                    // ── Zoom Reset: Ctrl+0 ────────────────────────────────
                    winit::keyboard::KeyCode::Digit0 | winit::keyboard::KeyCode::Numpad0
                        if state.ctrl_pressed =>
                    {
                        state.reset_zoom();
                        state.set_status("Zoom: 100%".to_string());
                        ui.request_redraw();
                    }

                    // ── Ctrl+1 → Always jump to editor ───────────────────
                    winit::keyboard::KeyCode::Digit1 if state.ctrl_pressed => {
                        state.focused_panel = FocusedPanel::Editor;
                        ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                        ui.request_redraw();
                    }

                    // ── Escape → Unfocus panel, return to editor ──────────
                    winit::keyboard::KeyCode::Escape
                        if state.focused_panel != FocusedPanel::Editor
                            && state.focused_panel != FocusedPanel::Terminal =>
                    {
                        state.focused_panel = FocusedPanel::Editor;
                        ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                        ui.request_redraw();
                    }

                    // ── Alt+Left → Previous tab ───────────────────────────
                    winit::keyboard::KeyCode::ArrowLeft if state.alt_pressed => {
                        if let Some(idx) = state.active_tab {
                            if idx > 0 {
                                state.active_tab = Some(idx - 1);
                                ui.request_redraw();
                            }
                        }
                    }

                    // ── Alt+Right → Next tab ──────────────────────────────
                    winit::keyboard::KeyCode::ArrowRight if state.alt_pressed => {
                        if let Some(idx) = state.active_tab {
                            if idx + 1 < state.tabs.len() {
                                state.active_tab = Some(idx + 1);
                                ui.request_redraw();
                            }
                        }
                    }

                    // ── Alt+W → Close current tab ─────────────────────────
                    winit::keyboard::KeyCode::KeyW if state.alt_pressed => {
                        if let Some(idx) = state.active_tab {
                            state.close_tab(idx);
                            ui.request_redraw();
                        }
                    }

                    // ── Ctrl+Shift+P → Command Palette / LazyVim Cmdline ──
                    winit::keyboard::KeyCode::KeyP if state.ctrl_pressed && state.shift_pressed => {
                        state.open_cmdline(Some(""), None);
                        ui.request_redraw();
                    }

                    // ── Shift+: → LazyVim Command mode ───────────────────
                    key if *key == winit::keyboard::KeyCode::Semicolon
                        && state.shift_pressed
                        && !state.ctrl_pressed
                        && !state.alt_pressed
                        && state.inline_tree_input.is_none()
                        && !state.is_modal_or_dialog_open()
                        && (state.focused_panel == FocusedPanel::Explorer
                            || (state.focused_panel == FocusedPanel::Editor && state.editor_vim_normal)) =>
                    {
                        let dir = if state.focused_panel == FocusedPanel::Explorer {
                            selected_parent_dir(state)
                        } else {
                            None
                        };
                        state.open_cmdline(Some(""), dir);
                        ui.request_redraw();
                    }

                    // ── File tree navigation (Explorer focused) ───────────
                    key if state.focused_panel == FocusedPanel::Explorer
                        && state.sidebar_visible
                        && state.inline_tree_input.is_none()
                        && state.tree_context_menu.is_none()
                        && !state.is_modal_or_dialog_open() =>
                    {
                        handle_tree_key(ui, state, key);
                    }

                    _ => {}
                }
            }

            // ── CharTyped ':' → LazyVim Command mode ──────────────────────
            PlatformEvent::CharTyped(c) if *c == ':' => {
                if !state.ctrl_pressed
                    && !state.alt_pressed
                    && state.inline_tree_input.is_none()
                    && !state.is_modal_or_dialog_open()
                    && (state.focused_panel == FocusedPanel::Explorer || (state.focused_panel == FocusedPanel::Editor && state.editor_vim_normal))
                {
                    let dir = if state.focused_panel == FocusedPanel::Explorer {
                        selected_parent_dir(state)
                    } else {
                        None
                    };
                    state.open_cmdline(Some(""), dir);
                    ui.request_redraw();
                }
            }

            // ── Modifier release ──────────────────────────────────────────
            PlatformEvent::KeyUp { key } => match key {
                winit::keyboard::KeyCode::ShiftLeft
                | winit::keyboard::KeyCode::ShiftRight => {
                    state.shift_pressed = false;
                }
                winit::keyboard::KeyCode::ControlLeft
                | winit::keyboard::KeyCode::ControlRight => {
                    state.ctrl_pressed = false;
                }
                winit::keyboard::KeyCode::AltLeft
                | winit::keyboard::KeyCode::AltRight => {
                    state.alt_pressed = false;
                }
                _ => {}
            },

            _ => {}
        }
    }
}

/// Handle keyboard input for the file tree when Explorer is focused.
///
/// Navigation: j/↓ = down, k/↑ = up, l/→/Enter = open/expand, h/← = collapse/parent
/// Operations: a = new file, A = new folder, d/Delete = delete, r/F2 = rename, R = refresh
/// Exit:       q/Escape = back to editor
fn handle_tree_key(ui: &mut Ui, state: &mut ZenthreeState, key: &winit::keyboard::KeyCode) {
    use winit::keyboard::KeyCode;

    match key {
        // ── j / ArrowDown → select next visible item ──────────────────
        KeyCode::KeyJ | KeyCode::ArrowDown => {
            let items = &state.sidebar_items;
            if items.is_empty() { return; }
            let new_sel = if let Some(ref sel) = state.selected_sidebar_path.clone() {
                let idx = items.iter().position(|n| &n.path == sel);
                match idx {
                    Some(i) if i + 1 < items.len() => items[i + 1].path.clone(),
                    _ => items[0].path.clone(),
                }
            } else {
                items[0].path.clone()
            };
            state.selected_sidebar_path = Some(new_sel);
            ui.request_redraw();
        }

        // ── k / ArrowUp → select previous visible item ────────────────
        KeyCode::KeyK | KeyCode::ArrowUp => {
            let items = &state.sidebar_items;
            if items.is_empty() { return; }
            let last = items.len() - 1;
            let new_sel = if let Some(ref sel) = state.selected_sidebar_path.clone() {
                let idx = items.iter().position(|n| &n.path == sel);
                match idx {
                    Some(0) | None => items[last].path.clone(),
                    Some(i) => items[i - 1].path.clone(),
                }
            } else {
                items[last].path.clone()
            };
            state.selected_sidebar_path = Some(new_sel);
            ui.request_redraw();
        }

        // ── l / ArrowRight / Enter → open file or expand folder ───────
        KeyCode::KeyL | KeyCode::ArrowRight | KeyCode::Enter | KeyCode::NumpadEnter => {
            if let Some(sel) = state.selected_sidebar_path.clone() {
                if sel.is_dir() {
                    if state.expanded_dirs.contains(&sel) {
                        state.expanded_dirs.remove(&sel);
                    } else {
                        state.expanded_dirs.insert(sel.clone());
                    }
                    state.rebuild_tree();
                } else {
                    // Open the file
                    state.open_file(&sel);
                    state.focused_panel = FocusedPanel::Editor;
                    ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
                }
                ui.request_redraw();
            }
        }

        // ── h / ArrowLeft → collapse folder or jump to parent ─────────
        KeyCode::KeyH | KeyCode::ArrowLeft => {
            if let Some(sel) = state.selected_sidebar_path.clone() {
                if sel.is_dir() && state.expanded_dirs.contains(&sel) {
                    // Collapse this dir
                    state.expanded_dirs.remove(&sel);
                    state.rebuild_tree();
                } else if let Some(parent) = sel.parent() {
                    // Jump to parent
                    let parent = parent.to_path_buf();
                    if parent != state.current_dir {
                        state.selected_sidebar_path = Some(parent);
                    }
                }
                ui.request_redraw();
            }
        }

        // ── a → LazyVim Floating Dialog (New file/folder/cd) ──────────
        KeyCode::KeyA if !state.shift_pressed => {
            let dir = selected_parent_dir(state);
            state.open_cmdline(Some(""), dir);
            ui.request_redraw();
        }

        // ── A (Shift+a) → New folder in selected folder ───────────────
        KeyCode::KeyA if state.shift_pressed => {
            let dir = selected_parent_dir(state);
            state.open_cmdline(Some("mkdir "), dir);
            ui.request_redraw();
        }

        // ── : (Shift+;) → LazyVim Command mode in tree ────────────────
        KeyCode::Semicolon if state.shift_pressed => {
            let dir = selected_parent_dir(state);
            state.open_cmdline(Some(""), dir);
            ui.request_redraw();
        }

        // ── d / Delete → Delete selected ──────────────────────────────
        KeyCode::KeyD | KeyCode::Delete => {
            if let Some(sel) = state.selected_sidebar_path.clone() {
                state.open_delete_confirm(sel);
                ui.request_redraw();
            }
        }

        // ── r / F2 → Rename selected ───────────────────────────────────
        KeyCode::KeyR | KeyCode::F2 if !state.shift_pressed => {
            if let Some(sel) = state.selected_sidebar_path.clone() {
                state.open_rename_modal(sel);
                ui.request_redraw();
            }
        }

        // ── R (Shift+r) → Refresh tree ────────────────────────────────
        KeyCode::KeyR if state.shift_pressed => {
            state.rebuild_tree();
            ui.request_redraw();
        }

        // ── q / Escape → Back to editor (tree stays open) ─────────────
        KeyCode::KeyQ | KeyCode::Escape => {
            state.focused_panel = FocusedPanel::Editor;
            ui.focused_id = state.active_tab.map(|idx| zenthra::Id::from_u64(999996000 + idx as u64));
            ui.request_redraw();
        }

        _ => {}
    }
}

/// Returns the directory to use as the parent for new file/folder creation.
/// If selected item is a dir → use it. If a file → use its parent.
fn selected_parent_dir(state: &ZenthreeState) -> Option<std::path::PathBuf> {
    state.selected_sidebar_path.as_ref().map(|p| {
        if p.is_dir() {
            p.clone()
        } else {
            p.parent().map(|pp| pp.to_path_buf()).unwrap_or(state.current_dir.clone())
        }
    })
}
