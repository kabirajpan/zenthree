use crate::state::{ZenthreeState, InlineTreeInputMode, TreeContextMenu};
use zenthra::{Color, Ui, Align, FontWeight, Id, Style, use_context};
use crate::theme::ThemeColors;
use std::path::PathBuf;

pub fn draw_sidebar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let sidebar_w = state.sidebar_width;
    let is_modal_open = ui.modal_active || state.is_modal_or_dialog_open();

    // Sync modifiers state so widgets know when Ctrl/Shift/Alt are held
    let ctrl_key = Id::from_u64(0xFEED_C071);
    let shift_key = Id::from_u64(0xFEED_581F);
    let alt_key = Id::from_u64(0xFEED_A170);
    ui.interaction_state.insert(ctrl_key, if state.ctrl_pressed { 1.0 } else { 0.0 });
    ui.interaction_state.insert(shift_key, if state.shift_pressed { 1.0 } else { 0.0 });
    ui.interaction_state.insert(alt_key, if state.alt_pressed { 1.0 } else { 0.0 });

    // 1. Keyboard event handling for inline file/folder input or tree shortcuts
    if state.inline_tree_input.is_some() {
        let mut commit = false;
        let mut cancel = false;

        ui.input_events.retain(|event| {
            if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
                match key {
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        commit = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Escape => {
                        cancel = true;
                        return false;
                    }
                    _ => {}
                }
            }
            true
        });

        if commit {
            state.commit_inline_tree_input();
            ui.request_redraw();
        } else if cancel {
            state.cancel_inline_tree_input();
            ui.request_redraw();
        }

        // Keep cursor blinking smoothly without spinning the CPU
        ui.request_redraw_after(std::time::Duration::from_millis(500));
    } else if state.tree_context_menu.is_none() {
        // F2 shortcut for rename on selected item
        if let Some(ref selected_path) = state.selected_sidebar_path.clone() {
            let mut trigger_rename = false;
            ui.input_events.retain(|event| {
                if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
                    if *key == winit::keyboard::KeyCode::F2 {
                        trigger_rename = true;
                        return false;
                    }
                }
                true
            });
            if trigger_rename {
                state.open_rename_modal(selected_path.clone());
                ui.request_redraw();
            }
        }
    }

    let mut right_clicked_node: Option<(PathBuf, bool)> = None;

    let sidebar_start_x = ui.cursor_x;
    let sidebar_start_y = ui.cursor_y;
    if !is_modal_open && (ui.clicked || (ui.mouse_down && ui.active_drag.is_none())) {
        if ui.mouse_in_rect(sidebar_start_x, sidebar_start_y, sidebar_w, ui.height) {
            state.focused_panel = if state.sidebar_view == crate::state::SidebarView::Git {
                crate::state::types::FocusedPanel::Git
            } else {
                crate::state::types::FocusedPanel::Explorer
            };
        }
    }

    ui.container()
        .width(sidebar_w)
        .fill_y()
        .clip(true)
        .bg(colors.bg_sidebar)
        .column()
        .show(|ui| {
            if state.sidebar_view == crate::state::SidebarView::Extensions {
                crate::ui_layout::draw_extensions_panel(ui, state, sidebar_w);
            } else if state.sidebar_view == crate::state::SidebarView::Git {
                crate::ui_layout::draw_git_panel(ui, state, sidebar_w);
            } else if !state.has_workspace && state.sidebar_view != crate::state::SidebarView::Notes {
                draw_no_folder_view(ui, state, sidebar_w);
            } else {
                let is_notes_view = state.sidebar_view == crate::state::SidebarView::Notes;

                // Notes Vault Scope Switcher Bar: [f024b <project_name>]  [f059f Global]
                if is_notes_view {
                    let project_name = state.current_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("project")
                        .to_string();
                    let is_proj = state.notes_state.scope == crate::state::NotesVaultScope::Project;
                    let is_glob = state.notes_state.scope == crate::state::NotesVaultScope::Global;
                    let mut switch_scope: Option<crate::state::NotesVaultScope> = None;

                    let total_avail = (sidebar_w - 16.0 - 4.0 - 2.0).max(60.0);
                    let tab_proj_w = (total_avail * 0.60).floor();
                    let tab_glob_w = (total_avail - tab_proj_w).floor();

                    let max_chars = ((tab_proj_w - 24.0) / 7.0).max(4.0) as usize;
                    let proj_label = truncate_filename(&project_name, max_chars);

                    ui.container()
                        .fill_x()
                        .padding(6.0, 8.0, 3.0, 8.0)
                        .show(|ui| {
                            ui.container()
                                .fill_x()
                                .height(26.0)
                                .bg(colors.bg_base)
                                .border((colors.border.with_alpha(0.4), 1.0))
                                .radius_all(4.0)
                                .padding(2.0, 2.0, 2.0, 2.0)
                                .row()
                                .valign(Align::Center)
                                .gap(2.0)
                                .show(|ui| {
                                    // Tab 1: Project Workspace (60%)
                                    let btn_proj = ui.button(&format!("\u{f024b} {}", proj_label))
                                        .id(Id::from_u64(999999010))
                                        .height(22.0)
                                        .width(tab_proj_w)
                                        .padding(0.0, 4.0, 0.0, 4.0)
                                        .bg(if is_proj { colors.accent.with_alpha(0.20) } else { Color::TRANSPARENT })
                                        .hover(Style::bg(if is_proj { colors.accent.with_alpha(0.28) } else { Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.07) }))
                                        .color(if is_proj { colors.accent } else { colors.text_muted })
                                        .radius_all(3.0)
                                        .size(10.5)
                                        .show();
                                    if btn_proj.hovered {
                                        ui.cursor_icon = zenthra::CursorIcon::Pointer;
                                    }
                                    if btn_proj.clicked && !is_proj {
                                        switch_scope = Some(crate::state::NotesVaultScope::Project);
                                    }

                                    // Tab 2: Global (40%)
                                    let btn_glob = ui.button("\u{f059f} Global")
                                        .id(Id::from_u64(999999011))
                                        .height(22.0)
                                        .width(tab_glob_w)
                                        .padding(0.0, 4.0, 0.0, 4.0)
                                        .bg(if is_glob { colors.accent.with_alpha(0.20) } else { Color::TRANSPARENT })
                                        .hover(Style::bg(if is_glob { colors.accent.with_alpha(0.28) } else { Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.07) }))
                                        .color(if is_glob { colors.accent } else { colors.text_muted })
                                        .radius_all(3.0)
                                        .size(10.5)
                                        .show();
                                    if btn_glob.hovered {
                                        ui.cursor_icon = zenthra::CursorIcon::Pointer;
                                    }
                                    if btn_glob.clicked && !is_glob {
                                        switch_scope = Some(crate::state::NotesVaultScope::Global);
                                    }
                                });
                        });

                    if let Some(scope) = switch_scope {
                        state.notes_state.scope = scope;
                        state.selected_sidebar_path = None;
                        state.rebuild_tree();
                        ui.request_redraw();
                    }
                }

                let total_count = state.sidebar_items.len();
                let count_str = format!("{}/{}", total_count, total_count);

                match state.explorer_preset {
                crate::state::ExplorerPreset::NeovimSnacks => {
                    let mut do_new_file = false;
                    let mut do_new_folder = false;
                    let _do_refresh = false;
                    let _do_collapse = false;

                    ui.container()
                        .fill_x()
                        .padding(8.0, 10.0, 6.0, 10.0)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                                                .show(|ui| {
                            // Child 1: Left prompt and title box
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    // Prompt arrow '>' in Gruvbox Red
                                    ui.text(">")
                                        .size(11.5)
                                        .font_weight(FontWeight::Bold)
                                        .color(Color::rgb(251.0 / 255.0, 73.0 / 255.0, 52.0 / 255.0))
                                        .show();

                                     // Box with title and border: ╭────── Explorer ──────╮
                                    ui.container()
                                        .padding(2.0, 10.0, 2.0, 10.0)
                                        .border((Color::rgb(235.0 / 255.0, 219.0 / 255.0, 178.0 / 255.0), 1.0))
                                        .radius_all(3.0)
                                        .row()
                                        .valign(Align::Center)
                                        .gap(4.0)
                                        .show(|ui| {
                                            if is_notes_view {
                                                ui.text("\u{f1782}")
                                                    .size(11.0)
                                                    .color(colors.accent)
                                                    .show();
                                            }
                                            let title = if is_notes_view { "Notes" } else { "Explorer" };
                                            ui.text(title)
                                                .size(11.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(Color::rgb(235.0 / 255.0, 219.0 / 255.0, 178.0 / 255.0))
                                                .show();
                                        });
                                });

                            // Child 2: Action buttons & count
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .show(|ui| {
                                    let new_btn = ui.button("\u{ea7f}")
                                        .id(Id::from_u64(999997111))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(colors.highlight))
                                        .color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if new_btn.clicked {
                                        do_new_file = true;
                                    }

                                    let folder_btn = ui.button("\u{ea80}")
                                        .id(Id::from_u64(999997110))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(colors.highlight))
                                        .color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if folder_btn.clicked {
                                        do_new_folder = true;
                                    }

                                    if is_notes_view {
                                        let daily_btn = ui.button("\u{f0e17}")
                                            .id(Id::from_u64(999997119))
                                            .width(18.0)
                                            .height(18.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover(Style::bg(colors.highlight))
                                            .color(colors.text_muted)
                                            .radius_all(3.0)
                                            .padding(0.0, 0.0, 0.0, 0.0)
                                            .size(11.0)
                                            .show();
                                        if daily_btn.clicked {
                                            let cur = state.current_dir.clone();
                                            if let Ok(p) = state.notes_state.create_or_open_daily_note(&cur) {
                                                state.rebuild_tree();
                                                state.open_file(&p);
                                            }
                                        }
                                    }

                                    ui.text(&count_str)
                                        .size(10.5)
                                        .color(Color::rgb(124.0 / 255.0, 111.0 / 255.0, 100.0 / 255.0))
                                        .show();
                                });
                        });

                    if do_new_file {
                        state.start_new_file_inline(None);
                        ui.request_redraw();
                    }
                    if do_new_folder {
                        state.start_new_folder_inline(None);
                        ui.request_redraw();
                    }
                }
                crate::state::ExplorerPreset::ModernMinimalist => {
                    let mut do_new_file = false;
                    let mut do_new_folder = false;
                    let mut do_refresh = false;
                    let mut do_collapse = false;

                    let is_notes = state.sidebar_view == crate::state::SidebarView::Notes;
                    let root_name = if is_notes {
                        "NOTES".to_string()
                    } else {
                        state.current_dir.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("PROJECT")
                            .to_uppercase()
                    };

                    ui.container()
                        .fill_x()
                        .padding(8.0, 10.0, 6.0, 10.0)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                                                .show(|ui| {
                            // Child 1: Left Title
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .show(|ui| {
                                    if is_notes {
                                        ui.text("\u{f1782}")
                                            .size(12.0)
                                            .color(colors.accent)
                                            .show();
                                    }
                                    ui.text(&root_name)
                                        .size(10.5)
                                        .font_weight(FontWeight::Bold)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            // Child 2: Action buttons
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(2.0)
                                .show(|ui| {
                                    let new_btn = ui.button("\u{ea7f}")
                                        .id(Id::from_u64(999997101))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(colors.highlight))
                                        .color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if new_btn.clicked {
                                        do_new_file = true;
                                    }

                                    let folder_btn = ui.button("\u{ea80}")
                                        .id(Id::from_u64(999997100))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(colors.highlight))
                                        .color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if folder_btn.clicked {
                                        do_new_folder = true;
                                    }

                                    if is_notes {
                                        let daily_btn = ui.button("\u{f0e17}")
                                            .id(Id::from_u64(999997109))
                                            .width(18.0)
                                            .height(18.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover(Style::bg(colors.highlight))
                                            .color(colors.text_muted)
                                            .radius_all(3.0)
                                            .padding(0.0, 0.0, 0.0, 0.0)
                                            .size(11.0)
                                            .show();
                                        if daily_btn.clicked {
                                            let cur = state.current_dir.clone();
                                            if let Ok(p) = state.notes_state.create_or_open_daily_note(&cur) {
                                                state.rebuild_tree();
                                                state.open_file(&p);
                                                ui.request_redraw();
                                            }
                                        }
                                    }

                                    let ref_btn = ui.button("↺")
                                        .id(Id::from_u64(999997102))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(colors.highlight))
                                        .color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(11.5)
                                        .show();
                                    if ref_btn.clicked {
                                        do_refresh = true;
                                    }

                                    let col_btn = ui.button("⊟")
                                        .id(Id::from_u64(999997103))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(colors.highlight))
                                        .color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(11.5)
                                        .show();
                                    if col_btn.clicked {
                                        do_collapse = true;
                                    }
                                });
                        });

                    if do_new_file {
                        state.start_new_file_inline(None);
                        ui.request_redraw();
                    }
                    if do_new_folder {
                        state.start_new_folder_inline(None);
                        ui.request_redraw();
                    }
                    if do_refresh {
                        state.rebuild_tree();
                        ui.request_redraw();
                    }
                    if do_collapse {
                        state.collapse_all_folders();
                        ui.request_redraw();
                    }
                }
                crate::state::ExplorerPreset::ClassicTree => {
                    let mut do_new_file = false;
                    let mut do_new_folder = false;
                    let mut do_refresh = false;
                    let mut do_collapse = false;
                    let is_notes = state.sidebar_view == crate::state::SidebarView::Notes;
                    let root_name = if is_notes {
                        "NOTES".to_string()
                    } else {
                        state.current_dir.file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("Workspace")
                            .to_string()
                    };

                    ui.container()
                        .fill_x()
                        .padding(8.0, 10.0, 6.0, 10.0)
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            let icon = if is_notes { "\u{f1782}" } else { "📂" };
                            ui.text(icon)
                                .size(11.0)
                                .color(if is_notes { colors.accent } else { colors.text_primary })
                                .show();
                            ui.text(&root_name)
                                .size(11.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();

                            ui.container().fill_x().show(|_| {});

                            ui.text(&format!("({})", total_count))
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();

                            let new_btn = ui.button("\u{ea7f}")
                                .id(Id::from_u64(999997121))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover(Style::bg(colors.highlight))
                                .color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(12.0)
                                .show();
                            if new_btn.clicked {
                                do_new_file = true;
                            }

                            let folder_btn = ui.button("\u{ea80}")
                                .id(Id::from_u64(999997120))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover(Style::bg(colors.highlight))
                                .color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(12.0)
                                .show();
                            if folder_btn.clicked {
                                do_new_folder = true;
                            }

                            if is_notes {
                                let daily_btn = ui.button("\u{f0e17}")
                                    .id(Id::from_u64(999997129))
                                    .width(18.0)
                                    .height(18.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover(Style::bg(colors.highlight))
                                    .color(colors.text_muted)
                                    .radius_all(3.0)
                                    .padding(0.0, 0.0, 0.0, 0.0)
                                    .size(11.0)
                                    .show();
                                if daily_btn.clicked {
                                    let cur = state.current_dir.clone();
                                    if let Ok(p) = state.notes_state.create_or_open_daily_note(&cur) {
                                        state.rebuild_tree();
                                        state.open_file(&p);
                                    }
                                }
                            }

                            let ref_btn = ui.button("↺")
                                .id(Id::from_u64(999997104))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover(Style::bg(colors.highlight))
                                .color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(11.5)
                                .show();
                            if ref_btn.clicked {
                                do_refresh = true;
                            }

                            let col_btn = ui.button("⊟")
                                .id(Id::from_u64(999997105))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover(Style::bg(colors.highlight))
                                .color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(11.5)
                                .show();
                            if col_btn.clicked {
                                do_collapse = true;
                            }
                        });

                    if do_new_file {
                        state.start_new_file_inline(None);
                        ui.request_redraw();
                    }
                    if do_new_folder {
                        state.start_new_folder_inline(None);
                        ui.request_redraw();
                    }
                    if do_refresh {
                        state.rebuild_tree();
                        ui.request_redraw();
                    }
                    if do_collapse {
                        state.collapse_all_folders();
                        ui.request_redraw();
                    }
                }
            }

            ui.container()
                .fill_x()
                .height(1.0)
                .bg(colors.border)
                .show(|_| {});

            // File Listing scroll area (Tree View with branch guides)
            let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);
            let preset = state.explorer_preset;

            let (pad_v, pad_h) = match preset {
                crate::state::ExplorerPreset::NeovimSnacks => (3.0, 8.0),
                crate::state::ExplorerPreset::ModernMinimalist => (3.5, 8.0),
                crate::state::ExplorerPreset::ClassicTree => (3.0, 8.0),
            };

            let scroll_padding_h = 0.0;
            let is_tree_focused = state.focused_panel == crate::state::types::FocusedPanel::Explorer;
            let cursor_style = state.cursor_style;
            let cur_color = state.get_cursor_color(colors.accent);

            let items = &state.sidebar_items;

            // Calculate insertion point for inline file / folder creation
            let inline_insertion = if let Some(ref inline) = state.inline_tree_input {
                match &inline.mode {
                    InlineTreeInputMode::NewFile { parent_dir }
                    | InlineTreeInputMode::NewFolder { parent_dir } => {
                        let tree_root = state.get_tree_root();
                        if parent_dir == &tree_root {
                            if let Some(ref sel) = state.selected_sidebar_path {
                                if let Some(pos) = items.iter().position(|it| &it.path == sel) {
                                    let item = &items[pos];
                                    let depth = if item.is_dir && item.is_expanded {
                                        item.depth + 1
                                    } else {
                                        item.depth
                                    };
                                    Some((pos + 1, depth))
                                } else {
                                    Some((0, 0))
                                }
                            } else {
                                Some((0, 0))
                            }
                        } else {
                            if let Some(pos) = items.iter().position(|it| &it.path == parent_dir) {
                                Some((pos + 1, items[pos].depth + 1))
                            } else {
                                Some((items.len(), 0))
                            }
                        }
                    }
                    InlineTreeInputMode::Rename { .. } => None,
                }
            } else {
                None
            };

            let mut clicked_item_action: Option<(PathBuf, bool, bool)> = None;
            let mut inline_rendered = false;
            let mut do_start_new_note = false;
            let is_modal_open = state.is_modal_or_dialog_open();

            let tree_resp = ui.container()
                .fill_x()
                .fill_y()
                .scroll_y(true)
                .padding(2.0, 4.0, 2.0, scroll_padding_h)
                .gap(1.0)
                .show(|ui| {
                    if is_notes_view && items.is_empty() && state.inline_tree_input.is_none() {
                        ui.container()
                            .fill_x()
                            .padding(24.0, 16.0, 24.0, 16.0)
                            .column()
                            .valign(Align::Center)
                            .halign(Align::Center)
                            .gap(10.0)
                            .show(|ui| {
                                ui.text("\u{f1782}")
                                    .size(24.0)
                                    .color(colors.accent.with_alpha(0.5))
                                    .show();
                                ui.text("No notes yet")
                                    .size(11.5)
                                    .font_weight(FontWeight::Medium)
                                    .color(colors.text_muted)
                                    .show();
                                let create_btn = ui.button("+ New Note")
                                    .id(Id::from_u64(999997180))
                                    .height(24.0)
                                    .bg(colors.accent.with_alpha(0.15))
                                    .hover(Style::bg(colors.accent.with_alpha(0.25)))
                                    .color(colors.accent)
                                    .radius_all(4.0)
                                    .padding(0.0, 12.0, 0.0, 12.0)
                                    .size(11.0)
                                    .show();
                                if create_btn.clicked {
                                    do_start_new_note = true;
                                }
                            });
                    }

                    // Check if inline creation should be rendered at the very top (index 0)
                    if let Some((insert_idx, depth)) = inline_insertion {
                        if insert_idx == 0 && !inline_rendered {
                            inline_rendered = true;
                            if let Some(ref mut inline) = state.inline_tree_input {
                                draw_inline_create_row(ui, inline, is_notes_view, depth, &icon_theme, preset, sidebar_w, pad_h, pad_v, &colors);
                            }
                        }
                    }

                    for (idx, node) in items.iter().enumerate() {
                        let is_dir = node.is_dir;
                        let item_id = Id::from_u64(999993000 + idx as u64);

                        let is_being_renamed = match &state.inline_tree_input {
                            Some(crate::state::InlineTreeState { mode: InlineTreeInputMode::Rename { target_path }, .. }) => {
                                *target_path == node.path
                            }
                            _ => false,
                        };

                        let effective_name = if is_being_renamed {
                            let t = state.inline_tree_input.as_ref().map(|i| i.text.as_str()).unwrap_or("");
                            if t.is_empty() { &node.name } else { t }
                        } else {
                            &node.name
                        };

                        let icon_source = if state.sidebar_view == crate::state::SidebarView::Notes {
                            if is_dir {
                                let opt = state.notes_state.get_folder_option(&node.path);
                                let color = match opt {
                                    crate::state::notes::NoteFolderOption::Star => Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0),
                                    crate::state::notes::NoteFolderOption::Todo => Color::rgb(125.0 / 255.0, 196.0 / 255.0, 228.0 / 255.0),
                                    crate::state::notes::NoteFolderOption::Reminder => Color::rgb(254.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0),
                                    crate::state::notes::NoteFolderOption::Calendar => Color::rgb(142.0 / 255.0, 192.0 / 255.0, 124.0 / 255.0),
                                    crate::state::notes::NoteFolderOption::Private => Color::rgb(202.0 / 255.0, 158.0 / 255.0, 230.0 / 255.0),
                                    crate::state::notes::NoteFolderOption::Default => colors.accent,
                                };
                                let glyph = if node.is_expanded {
                                    "\u{f0770}"
                                } else {
                                    opt.icon()
                                };
                                crate::theme::FileIcon::Glyph {
                                    glyph,
                                    color,
                                }
                            } else {
                                crate::theme::FileIcon::Glyph {
                                    glyph: "\u{f0219}",
                                    color: colors.text_muted,
                                }
                            }
                        } else if is_dir {
                            icon_theme.get_icon_for_folder(effective_name, node.is_expanded)
                        } else {
                            icon_theme.get_icon_for_file(effective_name, Some(&node.path))
                        };

                        let active_tab_path = state.active_tab
                            .and_then(|idx| state.tabs.get(idx))
                            .and_then(|t| t.path.as_ref());

                        let is_selected = match &state.selected_sidebar_path {
                            Some(selected) => selected == &node.path,
                            None => {
                                if let Some(act) = active_tab_path {
                                    !node.is_dir && act == &node.path
                                } else {
                                    idx == 0
                                }
                            }
                        };

                        let show_cursor = is_selected && is_tree_focused;

                        let pad_left = 4.0;
                        let pad_right = pad_h;

                        let mut item_container = ui.container()
                            .id(item_id)
                            .fill_x()
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                                                        .padding_left(pad_left)
                            .padding_right(pad_right)
                            .padding_top(pad_v)
                            .padding_bottom(pad_v)
                            .radius_all(0.0);

                        if is_selected {
                            match preset {
                                crate::state::ExplorerPreset::ModernMinimalist => {
                                    item_container = item_container
                                        .bg(colors.accent.with_alpha(0.20));
                                }
                                _ => {
                                    item_container = item_container
                                        .bg(Color::rgba(18.0 / 255.0, 68.0 / 255.0, 105.0 / 255.0, 1.0));
                                }
                            }
                        } else {
                            item_container = item_container
                                .bg(Color::TRANSPARENT)
                                .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)));
                        }

                        let resp = item_container.show(|ui| {
                            let text_color = if is_selected {
                                colors.text_primary
                            } else if state.explorer_git_status {
                                match node.git_status {
                                    Some(crate::state::GitStatus::Modified) => Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0),
                                    Some(crate::state::GitStatus::Untracked) => Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0),
                                    Some(crate::state::GitStatus::Added) => Color::rgb(166.0 / 255.0, 218.0 / 255.0, 149.0 / 255.0),
                                    Some(crate::state::GitStatus::Deleted) => Color::rgb(248.0 / 255.0, 113.0 / 255.0, 113.0 / 255.0),
                                    Some(crate::state::GitStatus::Renamed) => Color::rgb(125.0 / 255.0, 196.0 / 255.0, 228.0 / 255.0),
                                    Some(crate::state::GitStatus::Conflicted) => Color::rgb(254.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0),
                                    None => colors.text_primary,
                                }
                            } else {
                                colors.text_primary
                            };

                            // Left node content: cursor indicator, indentation/guides, fold arrow, file/folder icon, and name
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                                                .gap(4.0)
                                .show(|ui| {
                                    // Dedicated cursor slot always on the far left side
                                    let cursor_w = 7.0;
                                    let cursor_h = 13.0;
                                    if show_cursor {
                                        match cursor_style {
                                            crate::state::types::CursorStyle::Block => {
                                                ui.container()
                                                    .width(cursor_w)
                                                    .height(cursor_h)
                                                    .radius_all(1.0)
                                                    .bg(cur_color)
                                                    .show(|_| {});
                                            }
                                            crate::state::types::CursorStyle::Line => {
                                                ui.container()
                                                    .width(cursor_w)
                                                    .height(cursor_h)
                                                    .valign(Align::Center)
                                                    .halign(Align::Center)
                                                    .show(|ui| {
                                                        ui.container()
                                                            .width(2.0)
                                                            .height(cursor_h)
                                                            .radius_all(0.5)
                                                            .bg(cur_color)
                                                            .show(|_| {});
                                                    });
                                            }
                                            crate::state::types::CursorStyle::Underline => {
                                                ui.container()
                                                    .width(cursor_w)
                                                    .height(cursor_h)
                                                    .valign(Align::Bottom)
                                                    .halign(Align::Center)
                                                    .show(|ui| {
                                                        ui.container()
                                                            .width(cursor_w)
                                                            .height(2.5)
                                                            .radius_all(0.5)
                                                            .bg(cur_color)
                                                            .show(|_| {});
                                                    });
                                            }
                                        }
                                    } else {
                                        ui.container()
                                            .width(cursor_w)
                                            .height(cursor_h)
                                            .show(|_| {});
                                    }

                                    if state.explorer_tree_guides {
                                        if preset == crate::state::ExplorerPreset::ClassicTree {
                                            let guide_prefix = node.guide_prefix.replace("├─", "┊ ").replace("└─", "└ ").replace("│ ", "┊ ");
                                            ui.text(&guide_prefix)
                                                .size(11.0)
                                                .color(Color::rgb(100.0 / 255.0, 100.0 / 255.0, 100.0 / 255.0))
                                                .show();
                                        } else {
                                            // Branch guide lines
                                            ui.text(&node.guide_prefix)
                                                .size(11.0)
                                                .color(Color::rgb(124.0 / 255.0, 111.0 / 255.0, 100.0 / 255.0))
                                                .show();
                                        }
                                    } else {
                                        // Default (Zed style): clean indentation without guidelines
                                        let indent = node.depth as f32 * 14.0;
                                        if indent > 0.0 {
                                            ui.spacing(indent);
                                        }
                                        ui.container()
                                            .width(12.0)
                                            .valign(Align::Center)
                                            .halign(Align::Center)
                                            .show(|ui| {
                                                if is_dir {
                                                    let arrow = if node.is_expanded { "\u{f078}" } else { "\u{f054}" };
                                                    ui.text(arrow)
                                                        .size(8.0)
                                                        .color(if is_selected { colors.text_primary } else { colors.text_muted })
                                                        .show();
                                                }
                                            });
                                    }

                                    icon_source.show(ui, 14.5);

                                    if is_being_renamed {
                                        let indent_w = if state.explorer_tree_guides {
                                            node.guide_prefix.chars().count() as f32 * 6.5
                                        } else {
                                            (node.depth as f32 * 14.0) + 12.0
                                        };
                                        let cursor_slot_w = 7.0 + 4.0;
                                        let used_w = pad_left + pad_right + cursor_slot_w + indent_w + 14.5 + 8.0 + 10.0;
                                        let avail_w = (sidebar_w - used_w).max(30.0);

                                        let input_seed = 999997198u64;
                                        let actual_input_id = {
                                            use std::hash::{Hash, Hasher};
                                            let mut hasher = std::collections::hash_map::DefaultHasher::new();
                                            input_seed.hash(&mut hasher);
                                            Id::from_u64(hasher.finish())
                                        };
                                        if ui.focused_id != Some(actual_input_id) {
                                            ui.focused_id = Some(actual_input_id);
                                            if let Some(ref inline) = state.inline_tree_input {
                                                let sel_anchor_id = Id::from_u64(actual_input_id.raw() ^ 0x5E1EC700);
                                                ui.cursor_state.insert(sel_anchor_id, 0);
                                                ui.cursor_state.insert(actual_input_id, inline.text.len());
                                            }
                                        }

                                        if let Some(ref mut inline) = state.inline_tree_input {
                                            let placeholder = if is_notes_view {
                                                if node.is_dir { "folder name…" } else { "note name…" }
                                            } else if node.is_dir {
                                                "folder name…"
                                            } else {
                                                "file name…"
                                            };

                                            ui.input(&mut inline.text).id(input_seed)
                                                .width(avail_w)
                                                .min_width(20.0)
                                                .color(colors.text_primary)
                                                .bg(Color::TRANSPARENT)
                                                .border(None)
                                                .placeholder(placeholder)
                                                .placeholder_color(colors.text_dim)
                                                .padding_x(2.0)
                                                .padding_y(2.0)
                                                .size(11.5)
                                                .show();
                                        }
                                    } else {
                                        let indent_w = if state.explorer_tree_guides {
                                            node.guide_prefix.chars().count() as f32 * 6.5
                                        } else {
                                            (node.depth as f32 * 14.0) + 12.0
                                        };
                                        let git_w = if state.explorer_git_status && node.git_status.is_some() {
                                            20.0
                                        } else {
                                            0.0
                                        };
                                        let cursor_slot_w = 7.0 + 4.0;
                                        let used_w = pad_left + pad_right + cursor_slot_w + indent_w + 14.5 + 8.0 + git_w + 10.0;
                                        let avail_w = (sidebar_w - used_w).max(24.0);
                                        let max_chars = (avail_w / 6.8).max(4.0) as usize;
                                        let base_name = if is_notes_view && !node.is_dir {
                                            node.name.strip_suffix(".md")
                                                .or_else(|| node.name.strip_suffix(".markdown"))
                                                .unwrap_or(&node.name)
                                        } else {
                                            &node.name
                                        };
                                        let display_name = truncate_filename(base_name, max_chars);

                                        ui.text(&display_name)
                                            .size(12.0)
                                            .color(text_color)
                                            .show();

                                        if is_notes_view && state.notes_state.pinned_paths.contains(&node.path) {
                                            ui.text("\u{f0403}")
                                                .size(10.0)
                                                .color(colors.accent)
                                                .show();
                                        }
                                    }
                                });

                            // Right Git status badge
                            if state.explorer_git_status {
                                if let Some(ref status) = node.git_status {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                                                                .padding_right(2.0)
                                        .show(|ui| {
                                            if node.is_dir {
                                                ui.container()
                                                    .width(5.0)
                                                    .height(5.0)
                                                    .radius_all(2.5)
                                                    .bg(Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0))
                                                    .show(|_| {});
                                            } else {
                                                let (badge, color) = match status {
                                                    crate::state::GitStatus::Modified => ("M", Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0)),
                                                    crate::state::GitStatus::Untracked => ("U", Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0)),
                                                    crate::state::GitStatus::Added => ("A", Color::rgb(166.0 / 255.0, 218.0 / 255.0, 149.0 / 255.0)),
                                                    crate::state::GitStatus::Deleted => ("D", Color::rgb(248.0 / 255.0, 113.0 / 255.0, 113.0 / 255.0)),
                                                    crate::state::GitStatus::Renamed => ("R", Color::rgb(125.0 / 255.0, 196.0 / 255.0, 228.0 / 255.0)),
                                                    crate::state::GitStatus::Conflicted => ("!", Color::rgb(254.0 / 255.0, 100.0 / 255.0, 50.0 / 255.0)),
                                                };
                                                ui.text(badge)
                                                    .size(9.0)
                                                    .font_weight(FontWeight::Bold)
                                                    .color(color)
                                                    .show();
                                            }
                                        });
                                }
                            }
                        });

                        if resp.clicked && !is_modal_open {
                            clicked_item_action = Some((node.path.clone(), node.is_dir, node.is_expanded));
                        }

                        if ui.right_clicked && resp.hovered && !is_modal_open {
                            right_clicked_node = Some((node.path.clone(), node.is_dir));
                        }

                        // Check if inline creation should be rendered right after this node
                        if let Some((insert_idx, depth)) = inline_insertion {
                            if insert_idx == idx + 1 && !inline_rendered {
                                inline_rendered = true;
                                if let Some(ref mut inline) = state.inline_tree_input {
                                    draw_inline_create_row(ui, inline, is_notes_view, depth, &icon_theme, preset, sidebar_w, pad_h, pad_v, &colors);
                                }
                            }
                        }
                    }

                    // If inline creation has not been rendered yet, append it at the bottom
                    if let Some((_, depth)) = inline_insertion {
                        if !inline_rendered {
                            inline_rendered = true;
                            if let Some(ref mut inline) = state.inline_tree_input {
                                draw_inline_create_row(ui, inline, is_notes_view, depth, &icon_theme, preset, sidebar_w, pad_h, pad_v, &colors);
                            }
                        }
                    }
                });

            if do_start_new_note {
                state.start_new_file_inline(None);
                ui.request_redraw();
            }

            if let Some((path, is_dir, is_expanded)) = clicked_item_action {
                if state.inline_tree_input.is_some() {
                    state.commit_inline_tree_input();
                }
                state.selected_sidebar_path = Some(path.clone());
                state.focused_panel = crate::state::types::FocusedPanel::Explorer;
                ui.focused_id = None;
                if is_dir {
                    if is_expanded {
                        state.expanded_dirs.remove(&path);
                    } else {
                        state.expanded_dirs.insert(path);
                    }
                    state.rebuild_tree();
                    state.save_persistent_state();
                } else {
                    state.open_file(&path);
                }
                ui.request_redraw();
            }

            if tree_resp.clicked && !is_modal_open {
                state.focused_panel = crate::state::types::FocusedPanel::Explorer;
                ui.focused_id = None;
                ui.needs_redraw = true;
            }

            // Right click on item -> Open context menu
            if let Some((path, is_dir)) = right_clicked_node {
                if state.inline_tree_input.is_some() {
                    state.commit_inline_tree_input();
                }
                state.selected_sidebar_path = Some(path.clone());
                state.focused_panel = crate::state::types::FocusedPanel::Explorer;
                ui.focused_id = None;
                state.tree_context_menu = Some(TreeContextMenu {
                    target_path: path,
                    is_dir,
                    menu_x: ui.mouse_x,
                    menu_y: ui.mouse_y,
                });
                ui.request_redraw();
            } else if ui.right_clicked && tree_resp.hovered && state.tree_context_menu.is_none() {
                // Right click on empty area -> Context menu for tree root
                if state.inline_tree_input.is_some() {
                    state.commit_inline_tree_input();
                }
                let root = state.get_tree_root();
                state.tree_context_menu = Some(TreeContextMenu {
                    target_path: root,
                    is_dir: true,
                    menu_x: ui.mouse_x,
                    menu_y: ui.mouse_y,
                });
                ui.request_redraw();
            }
        }
    });

    // Draw Tree Context Menu Overlay if active
    if let Some(menu) = state.tree_context_menu.clone() {
        let is_notes = state.sidebar_view == crate::state::SidebarView::Notes;
        let menu_w = 210.0;
        let menu_h = if is_notes && menu.is_dir { 440.0 } else { 330.0 };
        let menu_x = menu.menu_x.clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
        let menu_y = menu.menu_y.clamp(8.0, (ui.height - menu_h - 8.0).max(8.0));

        let mouse_in_menu = ui.mouse_in_rect(menu_x, menu_y, menu_w, menu_h);
        let mut close_menu = false;

        // Light dismiss on click outside
        if (ui.clicked || ui.right_clicked) && !mouse_in_menu {
            close_menu = true;
            ui.consume_click();
            ui.consume_right_click();
        }

        let root = state.get_tree_root();
        let target_dir = if menu.is_dir {
            menu.target_path.clone()
        } else {
            menu.target_path.parent().unwrap_or(&root).to_path_buf()
        };

        let has_clipboard = state.tree_clipboard.is_some();

        ui.overlay(|ui| {
            ui.container()
                .raw_id(Id::from_u64(999996000))
                .pos(menu_x, menu_y)
                .overlay()
                .width(menu_w)
                .bg(colors.bg_sidebar)
                .border((colors.border, 1.0))
                .radius_all(6.0)
                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, 4.0, 16.0)
                .padding(4.0, 4.0, 4.0, 4.0)
                .column()
                .gap(1.0)
                .show(|ui| {
                    let new_file_label = if is_notes { "New Note..." } else { "New File..." };
                    if draw_menu_item(ui, Id::from_u64(999996001), "\u{ea7f}", new_file_label, "", true, false, &colors) {
                        state.start_new_file_inline(Some(target_dir.clone()));
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996002), "\u{ea80}", "New Folder...", "", true, false, &colors) {
                        state.start_new_folder_inline(Some(target_dir.clone()));
                        close_menu = true;
                    }

                    if is_notes && menu.is_dir {
                        ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                        let current_opt = state.notes_state.get_folder_option(&menu.target_path);
                        for (i, opt) in crate::state::notes::NoteFolderOption::all().iter().enumerate() {
                            let is_cur = current_opt == *opt;
                            let prefix = if is_cur { "● " } else { "   " };
                            let label = format!("{}{}", prefix, opt.label());
                            let opt_id = Id::from_u64(999996100 + i as u64);
                            if draw_menu_item(ui, opt_id, opt.icon(), &label, "", true, false, &colors) {
                                state.notes_state.folder_options.insert(menu.target_path.clone(), *opt);
                                state.rebuild_tree();
                                close_menu = true;
                            }
                        }
                    }

                    if is_notes && !menu.is_dir {
                        let is_pinned = state.notes_state.pinned_paths.contains(&menu.target_path);
                        let pin_label = if is_pinned { "Unpin Note" } else { "Pin Note" };
                        if draw_menu_item(ui, Id::from_u64(999996020), "\u{f0403}", pin_label, "", true, false, &colors) {
                            if is_pinned {
                                state.notes_state.pinned_paths.remove(&menu.target_path);
                            } else {
                                state.notes_state.pinned_paths.insert(menu.target_path.clone());
                            }
                            state.rebuild_tree();
                            close_menu = true;
                        }
                    }

                    // Divider
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

                    if draw_menu_item(ui, Id::from_u64(999996003), "\u{f044}", "Rename...", "F2", true, false, &colors) {
                        state.open_rename_modal(menu.target_path.clone());
                        close_menu = true;
                    }
                    let del_icon = if is_notes { "\u{f01b4}" } else { "\u{f014}" };
                    let del_label = if is_notes { "Delete Note" } else { "Delete" };
                    if draw_menu_item(ui, Id::from_u64(999996004), del_icon, del_label, "Del", true, true, &colors) {
                        state.open_delete_confirm(menu.target_path.clone());
                        close_menu = true;
                    }

                    // Divider
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

                    if draw_menu_item(ui, Id::from_u64(999996005), "\u{f0c4}", "Cut", "Ctrl+X", true, false, &colors) {
                        state.cut_tree_path(&menu.target_path);
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996006), "\u{f0c5}", "Copy", "Ctrl+C", true, false, &colors) {
                        state.copy_tree_path(&menu.target_path);
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996007), "\u{f0ea}", "Paste", "Ctrl+V", has_clipboard, false, &colors) {
                        state.paste_into_tree_path(&target_dir);
                        close_menu = true;
                    }

                    // Divider
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});

                    if draw_menu_item(ui, Id::from_u64(999996008), "\u{f0c1}", "Copy Path", "", true, false, &colors) {
                        state.set_clipboard(menu.target_path.display().to_string());
                        state.set_status("Path copied to clipboard");
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996009), "⎘", "Copy Relative Path", "", true, false, &colors) {
                        let rel = menu.target_path.strip_prefix(&root).unwrap_or(&menu.target_path);
                        state.set_clipboard(rel.display().to_string());
                        state.set_status("Relative path copied to clipboard");
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996010), "\u{f08e}", "Reveal in File Manager", "", true, false, &colors) {
                        state.reveal_in_file_manager(&menu.target_path);
                        close_menu = true;
                    }
                });
        });

        if close_menu {
            state.tree_context_menu = None;
            ui.request_redraw();
        }
    }

}

/// Render the inline input row for creating a new file or new folder.
fn draw_inline_create_row(
    ui: &mut Ui,
    inline: &mut crate::state::InlineTreeState,
    is_notes_view: bool,
    depth: usize,
    icon_theme: &crate::theme::IconTheme,
    preset: crate::state::ExplorerPreset,
    sidebar_w: f32,
    pad_h: f32,
    pad_v: f32,
    colors: &ThemeColors,
) {
    let is_folder_mode = matches!(&inline.mode, InlineTreeInputMode::NewFolder { .. });

    // Indent based on depth (same logic as tree items)
    let indent_px = match preset {
        crate::state::ExplorerPreset::NeovimSnacks
        | crate::state::ExplorerPreset::ClassicTree => depth as f32 * 14.0,
        crate::state::ExplorerPreset::ModernMinimalist => depth as f32 * 14.0,
    };
    let pad_left = 4.0;
    let pad_right = pad_h;
    let cursor_slot_w = 7.0 + 4.0;

    // Calculate available width for the input widget strictly so it NEVER overflows the sidebar.
    let used_w = pad_left + pad_right + cursor_slot_w + indent_px + 12.0 + 14.5 + 8.0 + 14.0;
    let input_w = (sidebar_w - used_w).max(40.0);

    ui.container()
        .fill_x()
        .row()
        .valign(Align::Center)
        .padding_left(pad_left)
        .padding_right(pad_right)
        .padding_top(pad_v)
        .padding_bottom(pad_v)
        .gap(4.0)
        .radius_all(0.0)
        .bg(colors.accent.with_alpha(0.06))
        .show(|ui| {
            // Dedicated cursor slot spacer matching tree items
            ui.container()
                .width(7.0)
                .height(13.0)
                .show(|_| {});

            // Indent spacer
            if indent_px > 0.0 {
                ui.spacing(indent_px);
            }

            // Arrow spacer container (12px) to align icon with normal tree items
            ui.container()
                .width(12.0)
                .valign(Align::Center)
                .halign(Align::Center)
                .show(|ui| {
                    if is_folder_mode {
                        ui.text("\u{f054}")
                            .size(8.0)
                            .color(colors.text_muted)
                            .show();
                    }
                });

            // Dynamic Icon matching theme icon pack or Notes vault
            let icon_source = if is_notes_view {
                if is_folder_mode {
                    crate::theme::FileIcon::Glyph { glyph: "\u{f0c82}", color: colors.accent }
                } else {
                    crate::theme::FileIcon::Glyph { glyph: "\u{f0219}", color: colors.text_muted }
                }
            } else if is_folder_mode {
                let folder_name = if inline.text.is_empty() { "folder" } else { &inline.text };
                icon_theme.get_icon_for_folder(folder_name, false)
            } else {
                let file_name = if inline.text.is_empty() { "file" } else { &inline.text };
                icon_theme.get_icon_for_file(file_name, None)
            };
            icon_source.show(ui, 14.5);

            // Use the actual ui.input widget with explicit width and min_width to prevent overflow
            let input_seed = 999997198u64;
            let actual_input_id = {
                use std::hash::{Hash, Hasher};
                let mut hasher = std::collections::hash_map::DefaultHasher::new();
                input_seed.hash(&mut hasher);
                Id::from_u64(hasher.finish())
            };
            if ui.focused_id != Some(actual_input_id) {
                ui.focused_id = Some(actual_input_id);
            }

            let placeholder = if is_notes_view {
                if is_folder_mode { "folder name…" } else { "note name…" }
            } else if is_folder_mode {
                "folder name…"
            } else {
                "file name…"
            };

            ui.input(&mut inline.text).id(input_seed)
                .width(input_w)
                .min_width(20.0)
                .color(colors.text_primary)
                .bg(Color::TRANSPARENT)
                .border(None)
                .placeholder(placeholder)
                .placeholder_color(colors.text_dim)
                .padding_x(2.0)
                .padding_y(2.0)
                .size(11.5)
                .show();
        });
}

/// Render a single context-menu item. Returns true if it was clicked.
pub(super) fn draw_menu_item(
    ui: &mut Ui,
    id: Id,
    icon: &str,
    label: &str,
    shortcut: &str,
    enabled: bool,
    is_destructive: bool,
    colors: &crate::theme::ThemeColors,
) -> bool {
    let text_color = if !enabled {
        colors.text_dim
    } else if is_destructive {
        Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0)
    } else {
        colors.text_primary
    };

    let resp = ui.container()
        .id(id)
        .fill_x()
        .height(26.0)
        .row()
        .valign(Align::Center)
        .padding_left(10.0)
        .padding_right(8.0)
        .gap(8.0)
        .radius_all(4.0)
        .bg(Color::TRANSPARENT)
        .hover(Style::bg(if enabled { colors.highlight } else { Color::TRANSPARENT }))
        .show(|ui| {
            // Icon column (fixed width so labels align)
            ui.container()
                .width(14.0)
                .halign(Align::Center)
                .show(|ui| {
                    ui.text(icon)
                        .size(11.0)
                        .color(if !enabled { colors.text_dim } else { text_color })
                        .show();
                });

            // Label
            ui.text(label)
                .size(11.5)
                .color(text_color)
                .show();

            // Spacer
            ui.container().fill_x().show(|_| {});

            // Shortcut hint
            if !shortcut.is_empty() {
                ui.text(shortcut)
                    .size(10.0)
                    .color(colors.text_dim)
                    .show();
            }
        });

    enabled && resp.clicked
}

fn draw_no_folder_view(ui: &mut Ui, state: &mut ZenthreeState, sidebar_w: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let inner_w = (sidebar_w - 24.0).max(1.0);

    // Section header
    ui.container()
        .fill_x()
        .padding(8.0, 10.0, 6.0, 10.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.text("NO FOLDER OPENED")
                .size(10.5)
                .font_weight(FontWeight::Bold)
                .color(colors.text_primary.with_alpha(1.0))
                .show();
        });

    ui.container()
        .fill_x()
        .height(1.0)
        .bg(colors.border)
        .show(|_| {});

    // Body
    ui.container()
        .fill_x()
        .padding(20.0, 12.0, 20.0, 12.0)
        .column()
        .gap(12.0)
        .valign(Align::Top)
        .show(|ui| {
            ui.text("You have not yet opened a folder or project.")
                .size(11.0)
                .color(colors.text_primary.with_alpha(1.0))
                .max_width(inner_w)
                .wrap(true)
                .show();

            let open_btn = ui.button("Open Folder")
                .id(Id::from_u64(999997190))
                .width(inner_w)
                .height(28.0)
                .bg(colors.accent)
                .hover(Style::bg(colors.accent.with_alpha(0.85)))
                .color(Color::WHITE)
                .radius_all(4.0)
                .size(11.0)
                .show();

            if open_btn.clicked {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.request_open_workspace(path);
                    ui.request_redraw();
                }
            }

            let new_file_btn = ui.button("New Untitled File")
                .id(Id::from_u64(999997191))
                .width(inner_w)
                .height(26.0)
                .bg(Color::TRANSPARENT)
                .hover(Style::bg(colors.highlight))
                .border((colors.border, 1.0))
                .color(colors.text_primary)
                .radius_all(4.0)
                .size(11.0)
                .show();

            if new_file_btn.clicked {
                state.new_untitled_file();
                ui.request_redraw();
            }

            ui.container()
                .width(inner_w)
                .padding_top(8.0)
                .column()
                .gap(6.0)
                .show(|ui| {
                    let shortcut_color = if colors.bg_base.r > 0.5 {
                        colors.text_primary.with_alpha(0.6)
                    } else {
                        colors.text_muted
                    };
                    ui.text("Shortcuts:")
                        .size(9.5)
                        .font_weight(FontWeight::Bold)
                        .color(shortcut_color)
                        .max_width(inner_w)
                        .show();
                    ui.text("Ctrl+Shift+O : Open Folder")
                        .size(9.0)
                        .color(shortcut_color)
                        .max_width(inner_w)
                        .show();
                    ui.text("Ctrl+O : Open File")
                        .size(9.0)
                        .color(shortcut_color)
                        .max_width(inner_w)
                        .show();
                    ui.text("Ctrl+N : New File")
                        .size(9.0)
                        .color(shortcut_color)
                        .max_width(inner_w)
                        .show();
                });
        });
}

pub fn truncate_filename(name: &str, max_chars: usize) -> String {
    let char_count = name.chars().count();
    if char_count <= max_chars {
        name.to_string()
    } else if max_chars <= 4 {
        let mut s: String = name.chars().take(max_chars.saturating_sub(1)).collect();
        s.push('…');
        s
    } else {
        if let Some(dot_pos) = name.rfind('.') {
            let ext = &name[dot_pos..];
            let ext_len = ext.chars().count();
            if ext_len >= 2 && ext_len <= 6 && ext_len < max_chars.saturating_sub(2) {
                let keep_stem = max_chars - ext_len - 1;
                let stem: String = name.chars().take(keep_stem).collect();
                return format!("{}…{}", stem, ext);
            }
        }
        let mut s: String = name.chars().take(max_chars.saturating_sub(1)).collect();
        s.push('…');
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_filename_short() {
        assert_eq!(truncate_filename("main.rs", 10), "main.rs");
        assert_eq!(truncate_filename("main.rs", 7), "main.rs");
    }

    #[test]
    fn test_truncate_filename_long_with_extension() {
        let truncated = truncate_filename("very_long_component_name.tsx", 14);
        assert_eq!(truncated.chars().count(), 14);
        assert!(truncated.ends_with(".tsx"));
        assert!(truncated.contains('…'));
    }

    #[test]
    fn test_truncate_filename_no_extension() {
        let truncated = truncate_filename("very_long_folder_name", 10);
        assert_eq!(truncated.chars().count(), 10);
        assert_eq!(truncated, "very_long…");
    }

    #[test]
    fn test_sidebar_cursor_state_and_styles() {
        let mut state = crate::state::ZenthreeState::default();
        state.focused_panel = crate::state::types::FocusedPanel::Explorer;
        assert_eq!(state.focused_panel, crate::state::types::FocusedPanel::Explorer);

        // Test cursor styles: Block, Line, Underline
        state.cursor_style = crate::state::types::CursorStyle::Block;
        assert_eq!(state.cursor_style, crate::state::types::CursorStyle::Block);

        state.cursor_style = crate::state::types::CursorStyle::Line;
        assert_eq!(state.cursor_style, crate::state::types::CursorStyle::Line);

        state.cursor_style = crate::state::types::CursorStyle::Underline;
        assert_eq!(state.cursor_style, crate::state::types::CursorStyle::Underline);

        // Verify cursor color reactive resolution
        let accent = zenthra_core::Color::rgb(0.2, 0.6, 0.9);
        let cur_color = state.get_cursor_color(accent);
        assert!(cur_color.a > 0.0);
    }
}
