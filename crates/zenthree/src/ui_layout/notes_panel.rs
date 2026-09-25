#![allow(dead_code)]
use crate::state::notes::{InlineCreateMode, NoteTreeNode, NotesVaultScope};
use crate::state::{path_matches, ZenthreeState};
use crate::theme::ThemeColors;
use std::path::PathBuf;
use zenthra::{use_context, Align, Color, FontWeight, Id, Ui};

pub fn draw_notes_panel(ui: &mut Ui, state: &mut ZenthreeState, sidebar_w: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    // Initial scan if notes have not been loaded yet
    if state.notes_state.last_scanned_at.is_none() {
        let current_dir = state.current_dir.clone();
        state.notes_state.scan_notes(&current_dir);
    }

    let mut note_to_open: Option<PathBuf> = None;
    let mut item_to_delete: Option<PathBuf> = None;
    let mut note_to_toggle_pin: Option<PathBuf> = None;
    let mut folder_to_toggle: Option<PathBuf> = None;
    let mut action_create_daily = false;
    let mut action_collapse_all = false;
    let mut action_refresh = false;
    let mut action_submit_inline = false;
    let mut action_cancel_inline = false;
    let mut switch_scope: Option<NotesVaultScope> = None;
    let mut trigger_new_file_in: Option<PathBuf> = None;
    let mut trigger_new_folder_in: Option<PathBuf> = None;

    let panel_w = sidebar_w;
    let project_name = state
        .current_dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("project");

    let vault_root = state.notes_state.get_vault_dir(&state.current_dir);

    // Outer panel column layout (Pure Obsidian Left Sidebar)
    ui.container()
        .width(panel_w)
        .fill_y()
        .clip(true)
        .column()
        .show(|ui| {
            // 1. Top Obsidian Icon Action Bar
            ui.container()
                .full_width()
                .height(32.0)
                .padding(4.0, 8.0, 4.0, 8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left: Vault label
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            ui.text("\u{f1782}")
                                .size(12.0)
                                .color(colors.accent)
                                .show();
                            ui.text("NOTES")
                                .size(10.5)
                                .weight(FontWeight::Bold)
                                .color(colors.text_muted)
                                .show();
                        });

                    // Right: Action buttons (New Note, New Folder, Collapse All, Search, Refresh)
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(2.0)
                        .show(|ui| {
                            // 📄 New Note at root
                            let new_note_btn = ui
                                .container()
                                .padding(3.0, 5.0, 3.0, 5.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight.with_alpha(0.20))
                                .clickable(true)
                                .show(|ui| {
                                    ui.text("\u{f044}") // New note
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if new_note_btn.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if new_note_btn.clicked {
                                trigger_new_file_in = Some(vault_root.clone());
                            }

                            // 📁 New Folder at root
                            let new_folder_btn = ui
                                .container()
                                .padding(3.0, 5.0, 3.0, 5.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight.with_alpha(0.20))
                                .clickable(true)
                                .show(|ui| {
                                    ui.text("\u{f07b}") // New folder
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if new_folder_btn.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if new_folder_btn.clicked {
                                trigger_new_folder_in = Some(vault_root.clone());
                            }

                            // 🔼 Collapse All
                            let collapse_btn = ui
                                .container()
                                .padding(3.0, 5.0, 3.0, 5.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight.with_alpha(0.20))
                                .clickable(true)
                                .show(|ui| {
                                    ui.text("\u{f066}")
                                        .size(10.5)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if collapse_btn.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if collapse_btn.clicked {
                                action_collapse_all = true;
                            }

                            // 🔍 Search Toggle
                            let search_toggle_btn = ui
                                .container()
                                .padding(3.0, 5.0, 3.0, 5.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight.with_alpha(0.20))
                                .clickable(true)
                                .show(|ui| {
                                    ui.text(zenthra::icons::NF_FA_SEARCH)
                                        .size(10.5)
                                        .color(if state.notes_state.search_active {
                                            colors.accent
                                        } else {
                                            colors.text_muted
                                        })
                                        .show();
                                });
                            if search_toggle_btn.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if search_toggle_btn.clicked {
                                state.notes_state.search_active = !state.notes_state.search_active;
                                ui.needs_redraw = true;
                            }

                            // 🔄 Refresh
                            let refresh_btn = ui
                                .container()
                                .padding(3.0, 5.0, 3.0, 5.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight.with_alpha(0.20))
                                .clickable(true)
                                .show(|ui| {
                                    ui.text("\u{f0450}")
                                        .size(10.5)
                                        .color(colors.text_dim)
                                        .show();
                                });
                            if refresh_btn.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if refresh_btn.clicked {
                                action_refresh = true;
                            }
                        });
                });

            // 2. Search Box (Collapsible)
            if state.notes_state.search_active || !state.notes_state.search_query.is_empty() {
                ui.container()
                    .full_width()
                    .padding(2.0, 8.0, 4.0, 8.0)
                    .show(|ui| {
                        ui.container()
                            .full_width()
                            .height(24.0)
                            .bg(colors.bg_base)
                            .border(colors.border.with_alpha(0.4), 1.0)
                            .radius_all(4.0)
                            .padding_left(6.0)
                            .padding_right(6.0)
                            .row()
                            .valign(Align::Center)
                            .gap(5.0)
                            .show(|ui| {
                                ui.text(zenthra::icons::NF_FA_SEARCH)
                                    .size(10.0)
                                    .color(colors.text_dim)
                                    .show();

                                let query_empty = state.notes_state.search_query.is_empty();

                                ui.input(&mut state.notes_state.search_query, Id::from_u64(999999020))
                                    .fill_x()
                                    .no_bg()
                                    .no_border()
                                    .color(colors.text_primary)
                                    .size(10.5)
                                    .show();

                                if !query_empty {
                                    let clear_btn = ui
                                        .container()
                                        .padding(2.0, 3.0, 2.0, 3.0)
                                        .radius_all(2.0)
                                        .hover_bg(colors.highlight.with_alpha(0.25))
                                        .clickable(true)
                                        .show(|ui| {
                                            ui.text("\u{f00d}")
                                                .size(8.5)
                                                .color(colors.text_muted)
                                                .show();
                                        });
                                    if clear_btn.hovered {
                                        ui.cursor_icon = zenthra::CursorIcon::Pointer;
                                    }
                                    if clear_btn.clicked {
                                        state.notes_state.search_query.clear();
                                        let cur = state.current_dir.clone();
                                        state.notes_state.scan_notes(&cur);
                                        ui.needs_redraw = true;
                                    }
                                }
                            });
                    });
            }

            // Separator line
            ui.container()
                .full_width()
                .height(1.0)
                .bg(colors.border.with_alpha(0.3))
                .show(|_| {});

            // 3. Root inline create input (if creating at vault root)
            if let Some(ref mode) = state.notes_state.inline_create {
                let parent = match mode {
                    InlineCreateMode::File { parent_dir } => parent_dir,
                    InlineCreateMode::Folder { parent_dir } => parent_dir,
                };
                if parent == &vault_root {
                    draw_inline_creator(
                        ui,
                        mode,
                        &mut state.notes_state.inline_input,
                        &colors,
                        0,
                        &mut action_submit_inline,
                        &mut action_cancel_inline,
                    );
                }
            }

            // 4. Obsidian-Style File Tree (Expands to fill all vertical space)
            let tree_nodes = state.notes_state.tree_nodes.clone();
            let active_path = state
                .active_tab
                .and_then(|idx| state.tabs.get(idx))
                .and_then(|t| t.path.clone());

            ui.container()
                .full_width()
                .fill_y()
                .scroll_y(true)
                .clip(true)
                .padding(2.0, 4.0, 2.0, 4.0)
                .gap(1.0)
                .show(|ui| {
                    if tree_nodes.is_empty() {
                        ui.container()
                            .full_width()
                            .padding(28.0, 10.0, 28.0, 10.0)
                            .column()
                            .valign(Align::Center)
                            .halign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                ui.text("No notes in this vault")
                                    .size(10.5)
                                    .color(colors.text_dim)
                                    .show();
                            });
                    } else {
                        for (idx, node) in tree_nodes.iter().enumerate() {
                            let is_active = active_path
                                .as_ref()
                                .map(|ap| path_matches(ap, &node.path))
                                .unwrap_or(false);

                            if node.is_dir {
                                draw_folder_row(
                                    ui,
                                    node,
                                    &colors,
                                    idx,
                                    &mut folder_to_toggle,
                                    &mut trigger_new_file_in,
                                    &mut trigger_new_folder_in,
                                    &mut item_to_delete,
                                );
                            } else {
                                draw_file_row(
                                    ui,
                                    node,
                                    is_active,
                                    &colors,
                                    idx,
                                    &mut note_to_open,
                                    &mut note_to_toggle_pin,
                                    &mut item_to_delete,
                                );
                            }

                            // Show inline input if active inside this folder
                            if let Some(ref mode) = state.notes_state.inline_create {
                                let parent = match mode {
                                    InlineCreateMode::File { parent_dir } => parent_dir,
                                    InlineCreateMode::Folder { parent_dir } => parent_dir,
                                };
                                if node.is_dir && parent == &node.path && node.is_expanded {
                                    draw_inline_creator(
                                        ui,
                                        mode,
                                        &mut state.notes_state.inline_input,
                                        &colors,
                                        node.depth + 1,
                                        &mut action_submit_inline,
                                        &mut action_cancel_inline,
                                    );
                                }
                            }
                        }
                    }
                });

            // 5. Obsidian-Style Bottom Bar: Vault Switcher & Daily Note
            ui.container()
                .full_width()
                .height(30.0)
                .bg(colors.bg_base)
                .border(colors.border.with_alpha(0.3), 1.0)
                .padding(3.0, 6.0, 3.0, 6.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left: Vault Scope Switcher [f024b <project_name>] | [f059f Global]
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(2.0)
                        .show(|ui| {
                            let is_proj = state.notes_state.scope == NotesVaultScope::Project;
                            let proj_label = format!("\u{f024b} {}", project_name);
                            let btn_proj = ui
                                .button(&proj_label)
                                .id(Id::from_u64(999999010))
                                .height(20.0)
                                .bg(if is_proj {
                                    colors.accent.with_alpha(0.15)
                                } else {
                                    Color::TRANSPARENT
                                })
                                .text_color(if is_proj {
                                    colors.accent
                                } else {
                                    colors.text_muted
                                })
                                .radius_all(3.0)
                                .padding_left(5.0)
                                .padding_right(5.0)
                                .size(9.5)
                                .show();
                            if btn_proj.clicked && !is_proj {
                                switch_scope = Some(NotesVaultScope::Project);
                            }

                            let is_glob = state.notes_state.scope == NotesVaultScope::Global;
                            let btn_glob = ui
                                .button("\u{f059f} Global")
                                .id(Id::from_u64(999999011))
                                .height(20.0)
                                .bg(if is_glob {
                                    colors.accent.with_alpha(0.15)
                                } else {
                                    Color::TRANSPARENT
                                })
                                .text_color(if is_glob {
                                    colors.accent
                                } else {
                                    colors.text_muted
                                })
                                .radius_all(3.0)
                                .padding_left(5.0)
                                .padding_right(5.0)
                                .size(9.5)
                                .show();
                            if btn_glob.clicked && !is_glob {
                                switch_scope = Some(NotesVaultScope::Global);
                            }
                        });

                    // Right: 📅 Daily Note button
                    let daily_btn = ui
                        .container()
                        .padding(2.0, 5.0, 2.0, 5.0)
                        .radius_all(3.0)
                        .hover_bg(colors.highlight.with_alpha(0.20))
                        .clickable(true)
                        .show(|ui| {
                            ui.text("\u{f0e17}")
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();
                        });
                    if daily_btn.hovered {
                        ui.cursor_icon = zenthra::CursorIcon::Pointer;
                    }
                    if daily_btn.clicked {
                        action_create_daily = true;
                    }
                });
        });

    // 2. Process deferred actions safely outside UI builder closures
    if let Some(scope) = switch_scope {
        state.notes_state.scope = scope;
        let current_dir = state.current_dir.clone();
        state.notes_state.scan_notes(&current_dir);
        ui.request_redraw();
    }

    if action_create_daily {
        let current_dir = state.current_dir.clone();
        if let Ok(path) = state.notes_state.create_or_open_daily_note(&current_dir) {
            state.open_file(&path);
            ui.request_redraw();
        }
    }

    if action_collapse_all {
        let current_dir = state.current_dir.clone();
        state.notes_state.collapse_all(&current_dir);
        ui.request_redraw();
    }

    if action_refresh {
        let current_dir = state.current_dir.clone();
        state.notes_state.scan_notes(&current_dir);
        ui.request_redraw();
    }

    if let Some(folder_path) = folder_to_toggle {
        let current_dir = state.current_dir.clone();
        state.notes_state.toggle_folder(&folder_path, &current_dir);
        ui.request_redraw();
    }

    if let Some(target_dir) = trigger_new_file_in {
        state.notes_state.inline_create = Some(InlineCreateMode::File {
            parent_dir: target_dir,
        });
        state.notes_state.inline_input.clear();
        ui.request_redraw();
    }

    if let Some(target_dir) = trigger_new_folder_in {
        state.notes_state.inline_create = Some(InlineCreateMode::Folder {
            parent_dir: target_dir,
        });
        state.notes_state.inline_input.clear();
        ui.request_redraw();
    }

    if action_cancel_inline {
        state.notes_state.inline_create = None;
        state.notes_state.inline_input.clear();
        ui.request_redraw();
    }

    if action_submit_inline {
        let input = state.notes_state.inline_input.clone();
        if let Some(mode) = state.notes_state.inline_create.clone() {
            let current_dir = state.current_dir.clone();
            match mode {
                InlineCreateMode::File { parent_dir } => {
                    if let Ok(path) = state.notes_state.create_note_file(&parent_dir, &input, &current_dir) {
                        state.notes_state.inline_create = None;
                        state.notes_state.inline_input.clear();
                        // Open new file immediately on the buffer tab!
                        state.open_file(&path);
                        ui.request_redraw();
                    }
                }
                InlineCreateMode::Folder { parent_dir } => {
                    if let Ok(_path) = state.notes_state.create_note_folder(&parent_dir, &input, &current_dir) {
                        state.notes_state.inline_create = None;
                        state.notes_state.inline_input.clear();
                        ui.request_redraw();
                    }
                }
            }
        }
    }

    if let Some(path) = note_to_open {
        state.open_file(&path);
        ui.request_redraw();
    }

    if let Some(path) = note_to_toggle_pin {
        let current_dir = state.current_dir.clone();
        state.notes_state.toggle_pin(&path, &current_dir);
        ui.request_redraw();
    }

    if let Some(path) = item_to_delete {
        let current_dir = state.current_dir.clone();
        let _ = state.notes_state.delete_item(&path, &current_dir);
        ui.request_redraw();
    }
}

/// Draw a folder row in the Obsidian tree
fn draw_folder_row(
    ui: &mut Ui,
    node: &NoteTreeNode,
    colors: &ThemeColors,
    idx: usize,
    folder_to_toggle: &mut Option<PathBuf>,
    trigger_new_file_in: &mut Option<PathBuf>,
    trigger_new_folder_in: &mut Option<PathBuf>,
    item_to_delete: &mut Option<PathBuf>,
) {
    let indent = (node.depth as f32) * 12.0 + 4.0;

    let row = ui
        .container()
        .full_width()
        .height(23.0)
        .padding_left(indent)
        .padding_right(6.0)
        .radius_all(3.0)
        .hover_bg(colors.highlight.with_alpha(0.18))
        .clickable(true)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            // Left: Chevron + Folder Name
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(5.0)
                .show(|ui| {
                    let chevron = if node.is_expanded {
                        "\u{f078}" // Chevron down (expanded)
                    } else {
                        "\u{f054}" // Chevron right (collapsed)
                    };
                    ui.text(chevron)
                        .size(8.5)
                        .color(colors.text_dim)
                        .show();

                    // Folder Name
                    ui.text(&node.name)
                        .size(11.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_muted)
                        .show();
                });

            // Right: Quick Actions (+ file, + folder, delete \u{f01b4})
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(2.0)
                .show(|ui| {
                    // Add File inside folder
                    let add_f_btn = ui
                        .button("+")
                        .id(Id::from_u64(999999400 + (idx * 3) as u64))
                        .height(15.0)
                        .bg(Color::TRANSPARENT)
                        .text_color(colors.text_dim)
                        .size(9.0)
                        .show();
                    if add_f_btn.clicked {
                        *trigger_new_file_in = Some(node.path.clone());
                    }

                    // Add Subfolder inside folder
                    let add_sub_btn = ui
                        .button("\u{f07b}")
                        .id(Id::from_u64(999999402 + (idx * 3) as u64))
                        .height(15.0)
                        .bg(Color::TRANSPARENT)
                        .text_color(colors.text_dim)
                        .size(8.5)
                        .show();
                    if add_sub_btn.clicked {
                        *trigger_new_folder_in = Some(node.path.clone());
                    }

                    // Delete folder: icon \u{f01b4}
                    let del_btn = ui
                        .button("\u{f01b4}")
                        .id(Id::from_u64(999999401 + (idx * 3) as u64))
                        .height(15.0)
                        .bg(Color::TRANSPARENT)
                        .text_color(colors.text_dim)
                        .size(8.5)
                        .show();
                    if del_btn.clicked {
                        *item_to_delete = Some(node.path.clone());
                    }
                });
        });

    if row.hovered {
        ui.cursor_icon = zenthra::CursorIcon::Pointer;
    }
    if row.clicked {
        *folder_to_toggle = Some(node.path.clone());
    }
}

/// Draw a note / file row in the Obsidian tree
fn draw_file_row(
    ui: &mut Ui,
    node: &NoteTreeNode,
    is_active: bool,
    colors: &ThemeColors,
    idx: usize,
    note_to_open: &mut Option<PathBuf>,
    note_to_toggle_pin: &mut Option<PathBuf>,
    item_to_delete: &mut Option<PathBuf>,
) {
    let indent = (node.depth as f32) * 12.0 + 16.0;

    let row = ui
        .container()
        .full_width()
        .height(23.0)
        .padding_left(indent)
        .padding_right(6.0)
        .radius_all(3.0)
        .bg(if is_active {
            colors.accent.with_alpha(0.14)
        } else {
            Color::TRANSPARENT
        })
        .hover_bg(if is_active {
            colors.accent.with_alpha(0.20)
        } else {
            colors.highlight.with_alpha(0.18)
        })
        .clickable(true)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            // Left: Title (clean title without .md)
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(5.0)
                .show(|ui| {
                    if node.is_pinned {
                        // Pin icon: \u{f0403}
                        ui.text("\u{f0403}")
                            .size(9.5)
                            .color(colors.accent)
                            .show();
                    } else if node.is_daily {
                        // Calendar icon for daily notes
                        ui.text("\u{f0e17}")
                            .size(9.5)
                            .color(colors.accent.with_alpha(0.8))
                            .show();
                    }

                    // Display name (Obsidian-style)
                    ui.text(&node.name)
                        .size(10.8)
                        .weight(if is_active {
                            FontWeight::Bold
                        } else {
                            FontWeight::Regular
                        })
                        .color(if is_active {
                            colors.text_primary
                        } else {
                            colors.text_muted
                        })
                        .show();
                });

            // Right: Badge (PNG / JPG) + Pin (\u{f0403}) + Delete (\u{f01b4})
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(3.0)
                .show(|ui| {
                    if let Some(ref ext) = node.ext_badge {
                        ui.text(ext)
                            .size(8.5)
                            .weight(FontWeight::Medium)
                            .color(colors.text_dim)
                            .show();
                    }

                    // Pin toggle button: icon \u{f0403}
                    let pin_btn = ui
                        .button("\u{f0403}")
                        .id(Id::from_u64(999999500 + (idx * 2) as u64))
                        .height(15.0)
                        .bg(Color::TRANSPARENT)
                        .text_color(if node.is_pinned {
                            colors.accent
                        } else {
                            colors.text_dim
                        })
                        .size(8.5)
                        .show();
                    if pin_btn.clicked {
                        *note_to_toggle_pin = Some(node.path.clone());
                    }

                    // Delete button: icon \u{f01b4}
                    let del_btn = ui
                        .button("\u{f01b4}")
                        .id(Id::from_u64(999999501 + (idx * 2) as u64))
                        .height(15.0)
                        .bg(Color::TRANSPARENT)
                        .text_color(colors.text_dim)
                        .size(8.5)
                        .show();
                    if del_btn.clicked {
                        *item_to_delete = Some(node.path.clone());
                    }
                });
        });

    if row.hovered {
        ui.cursor_icon = zenthra::CursorIcon::Pointer;
    }
    if row.clicked {
        *note_to_open = Some(node.path.clone());
    }
}

/// Draw an inline input row for creating a new file or folder inside the tree
fn draw_inline_creator(
    ui: &mut Ui,
    mode: &InlineCreateMode,
    input_text: &mut String,
    colors: &ThemeColors,
    depth: usize,
    action_submit: &mut bool,
    action_cancel: &mut bool,
) {
    let indent = (depth as f32) * 12.0 + 12.0;
    let is_folder = matches!(mode, InlineCreateMode::Folder { .. });

    ui.container()
        .full_width()
        .height(24.0)
        .padding_left(indent)
        .padding_right(6.0)
        .radius_all(3.0)
        .bg(colors.bg_base)
        .border(colors.accent.with_alpha(0.6), 1.0)
        .row()
        .valign(Align::Center)
        .gap(4.0)
        .show(|ui| {
            ui.text(if is_folder { "\u{f07b}" } else { "\u{f044}" })
                .size(10.0)
                .color(colors.accent)
                .show();

            ui.input(input_text, Id::from_u64(999999080))
                .fill_x()
                .no_bg()
                .no_border()
                .color(colors.text_primary)
                .size(10.5)
                .show();

            // Submit check
            let ok_btn = ui
                .button("✓")
                .id(Id::from_u64(999999081))
                .height(18.0)
                .bg(colors.accent)
                .text_color(colors.text_primary)
                .radius_all(2.0)
                .padding_left(5.0)
                .padding_right(5.0)
                .size(9.0)
                .show();
            if ok_btn.clicked {
                *action_submit = true;
            }

            // Cancel cross
            let cancel_btn = ui
                .button("✕")
                .id(Id::from_u64(999999082))
                .height(18.0)
                .bg(colors.highlight.with_alpha(0.3))
                .text_color(colors.text_muted)
                .radius_all(2.0)
                .padding_left(5.0)
                .padding_right(5.0)
                .size(9.0)
                .show();
            if cancel_btn.clicked {
                *action_cancel = true;
            }
        });
}
