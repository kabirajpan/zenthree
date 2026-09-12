use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};

pub fn draw_sidebar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();
    let sidebar_w = state.sidebar_width;

    ui.container()
        .width(sidebar_w)
        .fill_y()
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .column()
        .show(|ui| {
            let total_count = state.sidebar_items.len();
            let count_str = format!("{}/{}", total_count, total_count);

            match state.explorer_preset {
                crate::state::ExplorerPreset::NeovimSnacks => {
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

                            // Child 2: Count "42/42"
                            ui.text(&count_str)
                                .size(10.5)
                                .color(Color::rgb(124.0 / 255.0, 111.0 / 255.0, 100.0 / 255.0))
                                .show();
                        });
                }
                crate::state::ExplorerPreset::ModernMinimalist => {
                    let mut do_new_file = false;
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
                                    let new_btn = ui.button("+")
                                        .id(Id::from_u64(999997101))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.1))
                                        .text_color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(12.0)
                                        .show();
                                    if new_btn.clicked {
                                        do_new_file = true;
                                    }

                                    let ref_btn = ui.button("↺")
                                        .id(Id::from_u64(999997102))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.1))
                                        .text_color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(11.0)
                                        .show();
                                    if ref_btn.clicked {
                                        do_refresh = true;
                                    }

                                    let col_btn = ui.button("⊟")
                                        .id(Id::from_u64(999997103))
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.1))
                                        .text_color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(11.0)
                                        .show();
                                    if col_btn.clicked {
                                        do_collapse = true;
                                    }
                                });
                        });

                    if do_new_file {
                        state.new_file_in_workspace();
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

                            let ref_btn = ui.button("↺")
                                .id(Id::from_u64(999997104))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.1))
                                .text_color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(11.0)
                                .show();
                            if ref_btn.clicked {
                                do_refresh = true;
                            }

                            let col_btn = ui.button("⊟")
                                .id(Id::from_u64(999997105))
                                .width(18.0)
                                .height(18.0)
                                .bg(Color::TRANSPARENT)
                                .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.1))
                                .text_color(colors.text_muted)
                                .radius_all(3.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .size(11.0)
                                .show();
                            if col_btn.clicked {
                                do_collapse = true;
                            }
                        });

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

            // File Listing scroll area (Tree View with branch guides)
            let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);
            let preset = state.explorer_preset;

            let scroll_padding_h = if preset == crate::state::ExplorerPreset::ModernMinimalist { 6.0 } else { 4.0 };

            ui.container()
                .full_width()
                .fill_y()
                .scroll_y(true)
                .padding(2.0, scroll_padding_h, 2.0, scroll_padding_h)
                .gap(1.0)
                .show(|ui| {
                    let items = state.sidebar_items.clone();
                    for (idx, node) in items.iter().enumerate() {
                        let is_dir = node.is_dir;
                        let item_id = Id::from_u64(999993000 + idx as u64);

                        let icon_source = if is_dir {
                            icon_theme.get_icon_for_folder(&node.name, node.is_expanded)
                        } else {
                            icon_theme.get_icon_for_file(&node.name, Some(&node.path))
                        };

                        let active_tab_path = state.active_tab
                            .and_then(|idx| state.tabs.get(idx))
                            .and_then(|t| t.path.as_ref());

                        let is_selected = state.selected_sidebar_path.as_ref() == Some(&node.path)
                            || (!node.is_dir && active_tab_path == Some(&node.path));

                        let (pad_v, pad_h, radius) = match preset {
                            crate::state::ExplorerPreset::NeovimSnacks => (2.0, 6.0, 0.0),
                            crate::state::ExplorerPreset::ModernMinimalist => (3.5, 6.0, 4.0),
                            crate::state::ExplorerPreset::ClassicTree => (2.5, 6.0, 0.0),
                        };

                        let mut item_container = ui.container()
                            .id(item_id)
                            .full_width()
                            .row()
                            .valign(Align::Center)
                            .padding_left(pad_h)
                            .padding_right(pad_h)
                            .padding_top(pad_v)
                            .padding_bottom(pad_v)
                            .gap(4.0)
                            .radius_all(radius);

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
                            } else if preset == crate::state::ExplorerPreset::ClassicTree && state.explorer_git_status {
                                match node.git_status {
                                    Some(crate::state::GitStatus::Modified) => Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0),
                                    Some(crate::state::GitStatus::Untracked) => Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0),
                                    None => colors.text_primary,
                                }
                            } else {
                                colors.text_primary
                            };

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

                            ui.text(&node.name)
                                .size(11.5)
                                .color(text_color)
                                .show();

                            if state.explorer_git_status {
                                if let Some(ref status) = node.git_status {
                                    ui.container().fill_x().show(|_| {});
                                    if preset == crate::state::ExplorerPreset::ModernMinimalist {
                                        let (badge, color) = match status {
                                            crate::state::GitStatus::Modified => ("M", Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0)),
                                            crate::state::GitStatus::Untracked => ("U", Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0)),
                                        };
                                        ui.text(badge)
                                            .size(9.0)
                                            .weight(FontWeight::Bold)
                                            .color(color)
                                            .show();
                                    } else {
                                        match status {
                                            crate::state::GitStatus::Modified => {
                                                ui.text("○")
                                                    .size(10.0)
                                                    .color(Color::rgb(238.0 / 255.0, 180.0 / 255.0, 46.0 / 255.0)) // #eeb42e
                                                    .show();
                                            }
                                            crate::state::GitStatus::Untracked => {
                                                ui.text("+")
                                                    .size(10.0)
                                                    .color(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0)) // #b8bb26
                                                    .show();
                                            }
                                        }
                                    }
                                }
                            }
                        });

                        if resp.clicked {
                            state.selected_sidebar_path = Some(node.path.clone());
                            if is_dir {
                                if node.is_expanded {
                                    state.expanded_dirs.remove(&node.path);
                                } else {
                                    state.expanded_dirs.insert(node.path.clone());
                                }
                                state.rebuild_tree();
                                state.save_persistent_state();
                            } else {
                                state.open_file(&node.path);
                            }
                            ui.request_redraw();
                        }
                    }
                });
        });

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
    let edge_draw_x = splitter_x - edge_thickness / 2.0;

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
