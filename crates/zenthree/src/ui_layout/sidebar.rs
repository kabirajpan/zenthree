use crate::state::{ZenthreeState, InlineTreeInputMode, TreeContextMenu};
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context};
use crate::theme::ThemeColors;
use std::path::PathBuf;

pub fn draw_sidebar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let sidebar_w = state.sidebar_width;

    // 1. Keyboard event handling for inline file/folder input or tree shortcuts
    if state.inline_tree_input.is_some() {
        let mut commit = false;
        let mut cancel = false;
        let mut backspace = false;
        let mut chars: Vec<char> = Vec::new();

        ui.input_events.retain(|event| {
            match event {
                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                    match key {
                        winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                            commit = true;
                            return false;
                        }
                        winit::keyboard::KeyCode::Escape => {
                            cancel = true;
                            return false;
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            backspace = true;
                            return false;
                        }
                        _ => {}
                    }
                }
                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                    if !c.is_control() {
                        chars.push(*c);
                        return false;
                    }
                }
                _ => {}
            }
            true
        });

        if let Some(ref mut inline) = state.inline_tree_input {
            if backspace {
                inline.text.pop();
                state.last_cursor_activity = std::time::Instant::now();
                ui.request_redraw();
            }
            for c in chars {
                inline.text.push(c);
                state.last_cursor_activity = std::time::Instant::now();
                ui.request_redraw();
            }
        }

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
                state.start_rename_inline(selected_path.clone());
                ui.request_redraw();
            }
        }
    }

    let mut right_clicked_node: Option<(PathBuf, bool)> = None;

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
            } else if state.sidebar_view == crate::state::SidebarView::Notes {
                crate::ui_layout::draw_notes_panel(ui, state, sidebar_w);
            } else if !state.has_workspace {
                draw_no_folder_view(ui, state, sidebar_w);
            } else {
                let total_count = state.sidebar_items.len();
                let count_str = format!("{}/{}", total_count, total_count);

                match state.explorer_preset {
                crate::state::ExplorerPreset::NeovimSnacks => {
                    let mut do_new_file = false;
                    let mut do_new_folder = false;
                    let _do_refresh = false;
                    let _do_collapse = false;

                    ui.container()
                        .full_width()
                        .padding(8.0, 10.0, 6.0, 10.0)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .no_wrap()
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
                                        .weight(FontWeight::Bold)
                                        .color(Color::rgb(251.0 / 255.0, 73.0 / 255.0, 52.0 / 255.0))
                                        .show();

                                    // Box with title and border: ╭────── Explorer ──────╮
                                    ui.container()
                                        .padding(2.0, 10.0, 2.0, 10.0)
                                        .border(Color::rgb(235.0 / 255.0, 219.0 / 255.0, 178.0 / 255.0), 1.0)
                                        .radius_all(3.0)
                                        .row()
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("Explorer")
                                                .size(11.0)
                                                .weight(FontWeight::Bold)
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
                                        .hover_bg(colors.highlight)
                                        .text_color(colors.text_muted)
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
                                        .hover_bg(colors.highlight)
                                        .text_color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if folder_btn.clicked {
                                        do_new_folder = true;
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

                    let root_name = state.current_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("PROJECT")
                        .to_uppercase();

                    ui.container()
                        .full_width()
                        .padding(8.0, 10.0, 6.0, 10.0)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .no_wrap()
                        .show(|ui| {
                            // Child 1: Left Title
                            ui.text(&root_name)
                                .size(10.5)
                                .weight(FontWeight::Bold)
                                .color(colors.text_muted)
                                .show();

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
                                        .hover_bg(colors.highlight)
                                        .text_color(colors.text_muted)
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
                                        .hover_bg(colors.highlight)
                                        .text_color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if folder_btn.clicked {
                                        do_new_folder = true;
                                    }

                                    let ref_btn = ui.button("↺")
                                        .id(Id::from_u64(999997102))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(colors.highlight)
                                        .text_color(colors.text_muted)
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
                                        .hover_bg(colors.highlight)
                                        .text_color(colors.text_muted)
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
                    let root_name = state.current_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Workspace")
                        .to_string();

                    ui.container()
                        .full_width()
                        .padding(8.0, 10.0, 6.0, 10.0)
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text("📂")
                                .size(11.0)
                                .show();
                            ui.text(&root_name)
                                .size(11.0)
                                .weight(FontWeight::Bold)
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
                                .hover_bg(colors.highlight)
                                .text_color(colors.text_muted)
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
                                .hover_bg(colors.highlight)
                                .text_color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(12.0)
                                .show();
                            if folder_btn.clicked {
                                do_new_folder = true;
                            }

                            let ref_btn = ui.button("↺")
                                .id(Id::from_u64(999997104))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover_bg(colors.highlight)
                                .text_color(colors.text_muted)
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
                                .hover_bg(colors.highlight)
                                .text_color(colors.text_muted)
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
                .full_width()
                .height(1.0)
                .bg(colors.border)
                .show(|_| {});

            // File Listing scroll area (Tree View with branch guides)
            let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);
            let preset = state.explorer_preset;

            let (pad_v, pad_h) = match preset {
                crate::state::ExplorerPreset::NeovimSnacks => (2.0, 8.0),
                crate::state::ExplorerPreset::ModernMinimalist => (3.0, 8.0),
                crate::state::ExplorerPreset::ClassicTree => (2.5, 8.0),
            };

            let scroll_padding_h = 0.0;

            let items = &state.sidebar_items;

            // Calculate insertion point for inline file / folder creation
            let inline_insertion = if let Some(ref inline) = state.inline_tree_input {
                match &inline.mode {
                    InlineTreeInputMode::NewFile { parent_dir }
                    | InlineTreeInputMode::NewFolder { parent_dir } => {
                        if parent_dir == &state.current_dir {
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

            let mut clicked_item_idx: Option<usize> = None;
            let mut inline_rendered = false;

            let tree_resp = ui.container()
                .full_width()
                .fill_y()
                .scroll_y(true)
                .padding(2.0, scroll_padding_h, 2.0, scroll_padding_h)
                .gap(1.0)
                .show(|ui| {
                    // Check if inline creation should be rendered at the very top (index 0)
                    if let Some((insert_idx, depth)) = inline_insertion {
                        if insert_idx == 0 && !inline_rendered {
                            inline_rendered = true;
                            draw_inline_create_row(ui, state, depth, &icon_theme, preset, sidebar_w, pad_h, pad_v);
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

                        let icon_source = if is_dir {
                            icon_theme.get_icon_for_folder(effective_name, node.is_expanded)
                        } else {
                            icon_theme.get_icon_for_file(effective_name, Some(&node.path))
                        };

                        let active_tab_path = state.active_tab
                            .and_then(|idx| state.tabs.get(idx))
                            .and_then(|t| t.path.as_ref());

                        let is_selected = match &state.selected_sidebar_path {
                            Some(selected) => selected == &node.path,
                            None => !node.is_dir && active_tab_path == Some(&node.path),
                        };

                        let mut item_container = ui.container()
                            .id(item_id)
                            .full_width()
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .no_wrap()
                            .padding_left(pad_h)
                            .padding_right(pad_h)
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
                            match preset {
                                crate::state::ExplorerPreset::ModernMinimalist => {
                                    item_container = item_container
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(colors.highlight);
                                }
                                _ => {
                                    item_container = item_container
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(60.0 / 255.0, 56.0 / 255.0, 54.0 / 255.0, 0.4));
                                }
                            }
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

                            // Left node content: indentation/guides, fold arrow, file/folder icon, and name
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .no_wrap()
                                .gap(4.0)
                                .show(|ui| {
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
                                                        .size(7.5)
                                                        .color(if is_selected { colors.text_primary } else { colors.text_muted })
                                                        .show();
                                                }
                                            });
                                    }

                                    icon_source.show(ui, 13.0);

                                    if is_being_renamed {
                                        let input_text = state.inline_tree_input.as_ref().map(|i| i.text.as_str()).unwrap_or("");
                                        let cursor_str = if (state.last_cursor_activity.elapsed().as_millis() / 530) % 2 == 0 {
                                            "|"
                                        } else {
                                            " "
                                        };
                                        ui.container()
                                            .height(20.0)
                                            .fill_x()
                                            .bg(colors.bg_base)
                                            .border(colors.accent, 1.0)
                                            .radius_all(2.0)
                                            .padding(0.0, 4.0, 0.0, 4.0)
                                            .row()
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                if input_text.is_empty() {
                                                    ui.text("name...")
                                                        .size(11.0)
                                                        .color(colors.text_dim)
                                                        .show();
                                                } else {
                                                    ui.text(input_text)
                                                        .size(11.5)
                                                        .color(colors.text_primary)
                                                        .show();
                                                }
                                                ui.text(cursor_str)
                                                    .size(11.5)
                                                    .weight(FontWeight::Bold)
                                                    .color(colors.accent)
                                                    .show();
                                            });
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
                                        let used_w = pad_h * 2.0 + indent_w + 13.0 + 8.0 + git_w + 10.0;
                                        let avail_w = (sidebar_w - used_w).max(24.0);
                                        let max_chars = (avail_w / 6.8).max(4.0) as usize;
                                        let display_name = truncate_filename(&node.name, max_chars);

                                        ui.text(&display_name)
                                            .size(11.5)
                                            .color(text_color)
                                            .show();
                                    }
                                });

                            // Right Git status badge
                            if state.explorer_git_status {
                                if let Some(ref status) = node.git_status {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .no_wrap()
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
                                                    .weight(FontWeight::Bold)
                                                    .color(color)
                                                    .show();
                                            }
                                        });
                                }
                            }
                        });

                        if resp.clicked {
                            clicked_item_idx = Some(idx);
                        }

                        if ui.right_clicked && resp.hovered {
                            right_clicked_node = Some((node.path.clone(), node.is_dir));
                        }

                        // Check if inline creation should be rendered right after this node
                        if let Some((insert_idx, depth)) = inline_insertion {
                            if insert_idx == idx + 1 && !inline_rendered {
                                inline_rendered = true;
                                draw_inline_create_row(ui, state, depth, &icon_theme, preset, sidebar_w, pad_h, pad_v);
                            }
                        }
                    }

                    // If inline creation has not been rendered yet, append it at the bottom
                    if let Some((_, depth)) = inline_insertion {
                        if !inline_rendered {
                            inline_rendered = true;
                            draw_inline_create_row(ui, state, depth, &icon_theme, preset, sidebar_w, pad_h, pad_v);
                        }
                    }
                });

            if let Some(idx) = clicked_item_idx {
                let action = items.get(idx).map(|n| (n.path.clone(), n.is_dir, n.is_expanded));
                if let Some((path, is_dir, is_expanded)) = action {
                    if state.inline_tree_input.is_some() {
                        state.commit_inline_tree_input();
                    }
                    state.selected_sidebar_path = Some(path.clone());
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
            }

            // Right click on item -> Open context menu
            if let Some((path, is_dir)) = right_clicked_node {
                if state.inline_tree_input.is_some() {
                    state.commit_inline_tree_input();
                }
                state.selected_sidebar_path = Some(path.clone());
                state.tree_context_menu = Some(TreeContextMenu {
                    target_path: path,
                    is_dir,
                    menu_x: ui.mouse_x,
                    menu_y: ui.mouse_y,
                });
                ui.request_redraw();
            } else if ui.right_clicked && tree_resp.hovered && state.tree_context_menu.is_none() {
                // Right click on empty area -> Context menu for workspace root
                if state.inline_tree_input.is_some() {
                    state.commit_inline_tree_input();
                }
                state.tree_context_menu = Some(TreeContextMenu {
                    target_path: state.current_dir.clone(),
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
        let menu_w = 210.0;
        let menu_h = 320.0;
        let menu_x = menu.menu_x.clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
        let menu_y = menu.menu_y.clamp(8.0, (ui.height - menu_h - 8.0).max(8.0));

        let mouse_in_menu = ui.mouse_in_rect(menu_x, menu_y, menu_w, menu_h);
        let mut close_menu = false;

        // Light dismiss on click outside
        if (ui.clicked || ui.right_clicked) && !mouse_in_menu {
            close_menu = true;
        }

        let target_dir = if menu.is_dir {
            menu.target_path.clone()
        } else {
            menu.target_path.parent().unwrap_or(&state.current_dir).to_path_buf()
        };

        let has_clipboard = state.tree_clipboard.is_some();

        ui.overlay(|ui| {
            ui.container()
                .raw_id(Id::from_u64(999996000))
                .absolute(menu_x, menu_y)
                .overlay()
                .width(menu_w)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .radius_all(6.0)
                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, 4.0, 16.0)
                .padding(4.0, 4.0, 4.0, 4.0)
                .column()
                .gap(1.0)
                .show(|ui| {
                    if draw_menu_item(ui, Id::from_u64(999996001), "\u{ea7f}", "New File...", "", true, false, &colors) {
                        state.start_new_file_inline(Some(target_dir.clone()));
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996002), "\u{ea80}", "New Folder...", "", true, false, &colors) {
                        state.start_new_folder_inline(Some(target_dir.clone()));
                        close_menu = true;
                    }

                    // Divider
                    ui.container().full_width().height(1.0).bg(colors.border).show(|_| {});

                    if draw_menu_item(ui, Id::from_u64(999996003), "\u{f044}", "Rename...", "F2", true, false, &colors) {
                        state.start_rename_inline(menu.target_path.clone());
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996004), "\u{f014}", "Delete", "Del", true, true, &colors) {
                        state.delete_tree_path(&menu.target_path);
                        close_menu = true;
                    }

                    // Divider
                    ui.container().full_width().height(1.0).bg(colors.border).show(|_| {});

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
                    ui.container().full_width().height(1.0).bg(colors.border).show(|_| {});

                    if draw_menu_item(ui, Id::from_u64(999996008), "\u{f0c1}", "Copy Path", "", true, false, &colors) {
                        state.set_clipboard(menu.target_path.display().to_string());
                        state.set_status("Path copied to clipboard");
                        close_menu = true;
                    }
                    if draw_menu_item(ui, Id::from_u64(999996009), "⎘", "Copy Relative Path", "", true, false, &colors) {
                        let rel = menu.target_path.strip_prefix(&state.current_dir).unwrap_or(&menu.target_path);
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

    // Render vertical splitter handle and capture mouse drags
    let splitter_x = ui.cursor_x;
    let splitter_y = ui.base_y;
    let status_bar_h = if state.status_bar_visible { 24.0 } else { 0.0 };
    let splitter_h = ui.height - 36.0 - status_bar_h;

    // Detect if mouse is hovering over 6-pixel splitter bounds
    let handle_width = 6.0;
    let is_hovered = ui.mouse_in_rect(splitter_x - handle_width / 2.0, splitter_y, handle_width, splitter_h);

    if is_hovered || state.active_resize_sidebar {
        ui.cursor_icon = CursorIcon::ColResize;
    }

    if is_hovered && ui.mouse_down && ui.active_drag.is_none() {
        state.active_resize_sidebar = true;
    }

    if state.active_resize_sidebar {
        if !ui.mouse_down {
            state.active_resize_sidebar = false;
            state.save_persistent_state();
        } else {
            // Dragging: update sidebar width
            let target_w = ui.mouse_x;
            state.sidebar_width = target_w.clamp(140.0, (ui.width * 0.45).max(200.0));
            ui.request_redraw();
        }
    }

    // Visual feedback for edge handler
    let edge_color = if state.active_resize_sidebar {
        colors.accent
    } else if is_hovered {
        colors.accent.with_alpha(0.8)
    } else {
        colors.border
    };

    let edge_thickness = if state.active_resize_sidebar || is_hovered { 2.0 } else { 1.0 };
    let edge_draw_x = if edge_thickness == 1.0 { (splitter_x - 1.0).round() } else { (splitter_x - 1.0).round() };

    ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [edge_draw_x, splitter_y],
            size: [edge_thickness, splitter_h],
            color: edge_color.to_array(),
            radius: [0.0; 4],
            border_width: 0.0,
            border_color: [0.0; 4],
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));

    // Center pill grip handle when hovered or active
    if is_hovered || state.active_resize_sidebar {
        let grip_w = 4.0;
        let grip_h = 36.0;
        let grip_x = splitter_x - grip_w / 2.0;
        let grip_y = splitter_y + (splitter_h - grip_h) / 2.0;
        ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
            instance: zenthra_render::RectInstance {
                pos: [grip_x, grip_y],
                size: [grip_w, grip_h],
                color: colors.accent.to_array(),
                radius: [2.0; 4],
                border_width: 0.0,
                border_color: [0.0; 4],
                clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
                ..Default::default()
            },
        }));
    }
}

/// Render the inline input row for creating a new file or new folder.
fn draw_inline_create_row(
    ui: &mut Ui,
    state: &ZenthreeState,
    depth: usize,
    icon_theme: &crate::theme::IconTheme,
    preset: crate::state::ExplorerPreset,
    sidebar_w: f32,
    pad_h: f32,
    pad_v: f32,
) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    let is_folder_mode = matches!(
        state.inline_tree_input.as_ref().map(|i| &i.mode),
        Some(InlineTreeInputMode::NewFolder { .. })
    );

    let input_text = state.inline_tree_input.as_ref().map(|i| i.text.as_str()).unwrap_or("");

    // Blinking cursor
    let cursor_str = if (state.last_cursor_activity.elapsed().as_millis() / 530) % 2 == 0 {
        "|"
    } else {
        " "
    };

    // Indent based on depth (same logic as tree items)
    let indent_px = match preset {
        crate::state::ExplorerPreset::NeovimSnacks
        | crate::state::ExplorerPreset::ClassicTree => depth as f32 * 14.0,
        crate::state::ExplorerPreset::ModernMinimalist => depth as f32 * 14.0,
    };

    // Approximate available width for text input: account for indent, arrow container (12px), icon (13px), padding & gaps
    let icon_and_padding = pad_h * 2.0 + indent_px + 12.0 + 13.0 + 4.0 * 2.0;
    let avail_w = (sidebar_w - icon_and_padding - 20.0).max(60.0);

    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .padding_left(pad_h)
        .padding_right(pad_h)
        .padding_top(pad_v)
        .padding_bottom(pad_v)
        .gap(4.0)
        .radius_all(0.0)
        .bg(colors.accent.with_alpha(0.08))
        .show(|ui| {
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
                            .size(7.5)
                            .color(colors.text_muted)
                            .show();
                    }
                });

            // Dynamic Icon matching theme icon pack
            let icon_source = if is_folder_mode {
                let folder_name = if input_text.is_empty() { "folder" } else { input_text };
                icon_theme.get_icon_for_folder(folder_name, false)
            } else {
                let file_name = if input_text.is_empty() { "file" } else { input_text };
                icon_theme.get_icon_for_file(file_name, None)
            };
            icon_source.show(ui, 13.0);

            // Text input box
            ui.container()
                .height(20.0)
                .width(avail_w)
                .bg(colors.bg_base)
                .border(colors.accent, 1.0)
                .radius_all(2.0)
                .padding(0.0, 4.0, 0.0, 4.0)
                .row()
                .valign(Align::Center)
                .show(|ui| {
                    if input_text.is_empty() {
                        let placeholder = if is_folder_mode { "folder name…" } else { "file name…" };
                        ui.text(placeholder)
                            .size(11.0)
                            .color(colors.text_dim)
                            .show();
                    } else {
                        ui.text(input_text)
                            .size(11.5)
                            .color(colors.text_primary)
                            .show();
                    }
                    ui.text(cursor_str)
                        .size(11.5)
                        .weight(FontWeight::Bold)
                        .color(colors.accent)
                        .show();
                });

            // Hint text
            ui.text("↵")
                .size(10.0)
                .color(colors.text_dim)
                .show();
        });
}

/// Render a single context-menu item. Returns true if it was clicked.
fn draw_menu_item(
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
        .full_width()
        .height(26.0)
        .row()
        .valign(Align::Center)
        .padding_left(10.0)
        .padding_right(8.0)
        .gap(8.0)
        .radius_all(4.0)
        .bg(Color::TRANSPARENT)
        .hover_bg(if enabled { colors.highlight } else { Color::TRANSPARENT })
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
    let inner_w = (sidebar_w - 24.0).max(120.0);

    // Section header
    ui.container()
        .full_width()
        .padding(8.0, 10.0, 6.0, 10.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.text("NO FOLDER OPENED")
                .size(10.5)
                .weight(FontWeight::Bold)
                .color(colors.text_muted)
                .show();
        });

    ui.container()
        .full_width()
        .height(1.0)
        .bg(colors.border)
        .show(|_| {});

    // Body
    ui.container()
        .full_width()
        .padding(20.0, 12.0, 20.0, 12.0)
        .column()
        .gap(12.0)
        .valign(Align::Top)
        .show(|ui| {
            ui.text("You have not yet opened a folder or project.")
                .size(11.0)
                .color(colors.text_muted)
                .max_width(inner_w)
                .wrap(true)
                .show();

            let open_btn = ui.button("Open Folder")
                .id(Id::from_u64(999997190))
                .width(inner_w)
                .height(28.0)
                .bg(colors.accent)
                .hover_bg(colors.accent.with_alpha(0.85))
                .text_color(Color::WHITE)
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
                .hover_bg(colors.highlight)
                .border(colors.border, 1.0)
                .text_color(colors.text_primary)
                .radius_all(4.0)
                .size(11.0)
                .show();

            if new_file_btn.clicked {
                state.new_untitled_file();
                ui.request_redraw();
            }

            ui.container()
                .padding_top(8.0)
                .column()
                .gap(4.0)
                .show(|ui| {
                    ui.text("Shortcuts:")
                        .size(9.5)
                        .weight(FontWeight::Bold)
                        .color(colors.text_dim)
                        .show();
                    ui.text("Ctrl+Shift+O  Open Folder")
                        .size(9.0)
                        .color(colors.text_dim)
                        .show();
                    ui.text("Ctrl+O        Open File")
                        .size(9.0)
                        .color(colors.text_dim)
                        .show();
                    ui.text("Ctrl+N        New File")
                        .size(9.0)
                        .color(colors.text_dim)
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
}


