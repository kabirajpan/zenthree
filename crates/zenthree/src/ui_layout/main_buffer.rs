use std::path::PathBuf;
use crate::state::{ZenthreeState, path_matches, DiffLineState};
use crate::lsp::protocol::DiagnosticSeverity;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, Style, use_context};
use zenthra_widgets::ui::{DrawCommand, TextDraw, RectDraw, ScrollDrag};
use zenthra_render::RectInstance;
use zenthra_text::prelude::{CosmicFontProvider, TextOptions, ShapedBuffer};
use editor::buffer::Point;
use editor::selection::Selection;

pub fn draw_main_buffer(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    if state.tabs.is_empty()
        && state.panel_context_menu.is_none()
        && !ui.modal_active
        && !state.is_modal_or_dialog_open()
        && ui.right_clicked
        && ui.mouse_in_rect(ui.cursor_x, ui.cursor_y, width, height)
    {
        state.panel_context_menu = Some(crate::state::PanelContextMenu {
            target: crate::state::PanelContextMenuTarget::Editor,
            target_path: state.current_dir.clone(),
            target_is_dir: true,
            terminal_index: None,
            menu_x: ui.mouse_x,
            menu_y: ui.mouse_y,
        });
        ui.consume_click();
        ui.consume_right_click();
        state.focused_panel = crate::state::FocusedPanel::Editor;
        ui.request_redraw();
    }

    // Drain any fresh completions delivered by LSP
    if let Some(items) = state.lsp_manager.latest_completions.take() {
        if !items.is_empty() {
            state.completion_state.merge_lsp_items(&items);
            state.completion_items = items;
            state.completion_selected_idx = 0;
            state.completion_active = state.completion_state.active;
        }
    }

    ui.container()
        .width(width)
        .height(height)
        .bg(colors.bg_editor)
        .column()
        .show(|ui| {
            if state.tabs.is_empty() {
                // Welcome / Empty State
                ui.container()
                    .width(width)
                    .height(height)
                    .halign(Align::Center)
                    .valign(Align::Center)
                    .gap(16.0)
                    .show(|ui| {
                        ui.text("Zenthree Code Editor")
                            .size(24.0)
                            .font_weight(FontWeight::Bold)
                            .color(colors.accent)
                            .show();

                        ui.text("A fast, lightweight code editor built with Zenthra UI")
                            .size(13.0)
                            .color(colors.text_muted)
                            .show();

                        ui.spacing(16.0);

                        ui.container()
                            .gap(8.0)
                            .show(|ui| {
                                let draw_shortcut = |ui: &mut Ui, label: &str, keys: &str| {
                                    ui.container()
                                        .row()
                                        .gap(24.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text(label)
                                                .size(12.5)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.container()
                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                .bg(colors.border)
                                                .radius_all(4.0)
                                                .show(|ui| {
                                                    ui.text(keys)
                                                        .size(11.0)
                                                        .font_weight(FontWeight::Bold)
                                                        .color(colors.text_primary)
                                                        .show();
                                                });
                                        });
                                };

                                draw_shortcut(ui, "New File", "Ctrl + N");
                                draw_shortcut(ui, "Open File", "Ctrl + O");
                                draw_shortcut(ui, "Toggle Sidebar", "Ctrl + B");
                            });
                    });
            } else {
                let is_modal_open = state.is_modal_or_dialog_open() || ui.modal_active || state.cmdline_dialog.open;
                let mut active_tab_idx = state.active_tab.unwrap_or(0);

                let tab_bar_h = 26.0;

                // ─── Tab Bar ────────────────────────────────────────────────
                let tab_bar_bg = if state.glassmorphism { Color::TRANSPARENT } else { colors.bg_inactive_tab };
                ui.container()
                    .width(width)
                    .height(tab_bar_h)
                    .bg(tab_bar_bg)
                    .row()
                    .valign(Align::Center)
                    .show(|ui| {
                        let mut tab_to_close = None;
                        let mut tab_to_activate = None;
                        let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);

                        for idx in 0..state.tabs.len() {
                            let tab_is_dirty = state.tabs[idx].is_dirty;
                            let tab_name = state.tabs[idx].name.clone();
                            let is_active = idx == active_tab_idx;

                            let tab_id = Id::from_u64(999994000 + idx as u64);
                            let tab_close_id = Id::from_u64(999995000 + idx as u64);
                            let (bg, border_col) = if is_active {
                                (colors.bg_active_tab, if state.glassmorphism { colors.accent.with_alpha(0.60) } else { colors.border })
                            } else {
                                (if state.glassmorphism { Color::TRANSPARENT } else { colors.bg_inactive_tab }, if state.glassmorphism { colors.border.with_alpha(0.20) } else { colors.border })
                            };
                            let text_color = if is_active { colors.text_primary } else { colors.text_muted };

                            let icon_source = if let Some(ref path) = state.tabs[idx].path {
                                let fname = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                                icon_theme.get_icon_for_file(fname, Some(path))
                            } else {
                                let name = &state.tabs[idx].name;
                                icon_theme.get_icon_for_file(name, None)
                            };

                            let mut close_clicked = false;

                            let resp = ui.container()
                                .id(tab_id)
                                .height(tab_bar_h)
                                .bg(bg)
                                .border((border_col, 1.0))
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .padding_left(9.0)
                                .padding_right(6.0)
                                .hover(Style::bg(colors.highlight))
                                .show(|ui| {
                                    if state.tabs[idx].path.is_none() {
                                        ui.text("\u{f04e9}")
                                            .size(11.5)
                                            .color(colors.accent)
                                            .show();
                                    } else {
                                        icon_source.show(ui, 12.0);
                                    }

                                    ui.text(&tab_name)
                                        .size(11.0)
                                        .color(text_color)
                                        .show();

                                    let (lines_added, lines_removed) = {
                                        let tab = &state.tabs[idx];
                                        let rev_opt = tab.path.as_ref().and_then(|p| state.pending_reviews.iter().find(|r| path_matches(&r.path, p)));
                                        let mut la = rev_opt.map(|r| r.total_added_lines()).unwrap_or(0).max(tab.lines_added);
                                        let lr = rev_opt.map(|r| r.total_removed_lines()).unwrap_or(0).max(tab.lines_removed);
                                        if la == 0 && lr == 0 {
                                            if let Some(r) = rev_opt {
                                                if r.is_new_file {
                                                    la = tab.editor.buffer.line_count().max(1);
                                                }
                                            }
                                        }
                                        (la, lr)
                                    };
                                    if lines_added > 0 {
                                        ui.text(&format!("+{}", lines_added))
                                            .size(9.5)
                                            .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                            .show();
                                    }
                                    if lines_removed > 0 {
                                        ui.text(&format!("-{}", lines_removed))
                                            .size(9.5)
                                            .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                            .show();
                                    }

                                    if tab_is_dirty {
                                        ui.container()
                                            .width(5.0)
                                            .height(5.0)
                                            .radius_all(2.5)
                                            .bg(colors.accent)
                                            .show(|_| {});
                                    }

                                    // Close button
                                    let close_resp = ui.button("\u{f00d}")
                                        .id(tab_close_id)
                                        .width(14.0)
                                        .height(14.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(Color::rgb(30.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0)))
                                        .color(colors.text_primary)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(9.0)
                                        .show();

                                    if close_resp.clicked {
                                        close_clicked = true;
                                    }
                                });

                            if !is_modal_open {
                                if close_clicked {
                                    tab_to_close = Some(idx);
                                } else if resp.clicked {
                                    tab_to_activate = Some(idx);
                                }
                            }
                        }

                        if let Some(idx) = tab_to_close {
                            state.close_tab(idx);
                            active_tab_idx = state.active_tab.unwrap_or(0);
                            ui.request_redraw();
                        } else if let Some(idx) = tab_to_activate {
                            state.set_active_tab(Some(idx));
                            state.focused_panel = crate::state::types::FocusedPanel::Editor;
                            ui.focused_id = Some(Id::from_u64(999996000 + idx as u64));
                            active_tab_idx = idx;
                            state.sync_active_review_for_tab(idx);
                            if let Some(ref path) = state.tabs[idx].path.clone() {
                                state.selected_sidebar_path = Some(path.clone());
                                state.check_language_support_prompt(path);
                            }
                            ui.request_redraw();
                        }
                    });

                // 1px divider line below tab bar across full editor width
                ui.container()
                    .width(width)
                    .height(1.0)
                    .bg(if state.glassmorphism { colors.border.with_alpha(0.25) } else { colors.border })
                    .show(|_| {});

                let path_bar_h = 22.0;
                let mut segment_clicked_this_frame = false;

                // ─── Interactive Breadcrumb / File Path Bar (below buffer tabs) ──
                if active_tab_idx < state.tabs.len() {
                    let tab = &state.tabs[active_tab_idx];
                    let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);

                    struct BreadcrumbSegment {
                        name: String,
                        full_path: PathBuf,
                        #[allow(dead_code)]
                        is_dir: bool,
                    }

                    let mut segments: Vec<BreadcrumbSegment> = Vec::new();

                    if let Some(ref file_path) = tab.path {
                        let rel_opt = file_path.strip_prefix(&state.current_dir).ok().map(|p| p.to_path_buf())
                            .or_else(|| {
                                if let (Ok(can_file), Ok(can_dir)) = (file_path.canonicalize(), state.current_dir.canonicalize()) {
                                    can_file.strip_prefix(can_dir).ok().map(|p| p.to_path_buf())
                                } else {
                                    None
                                }
                            })
                            .or_else(|| {
                                if file_path.is_relative() {
                                    let full = state.current_dir.join(file_path);
                                    if let (Ok(can_file), Ok(can_dir)) = (full.canonicalize(), state.current_dir.canonicalize()) {
                                        return can_file.strip_prefix(can_dir).ok().map(|p| p.to_path_buf());
                                    }
                                }
                                None
                            });

                        let project_name = state.current_dir
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("");

                        if let Some(rel) = rel_opt {
                            let clean_rel = if !project_name.is_empty() {
                                rel.strip_prefix(project_name).unwrap_or(&rel)
                            } else {
                                &rel
                            };

                            let mut cur_path = state.current_dir.clone();
                            let components: Vec<_> = clean_rel.components().collect();
                            let comp_count = components.len();

                            for (i, comp) in components.into_iter().enumerate() {
                                let comp_str = comp.as_os_str().to_string_lossy().to_string();
                                cur_path.push(&comp_str);
                                let is_last = i == comp_count - 1;
                                segments.push(BreadcrumbSegment {
                                    name: comp_str,
                                    full_path: cur_path.clone(),
                                    is_dir: !is_last,
                                });
                            }
                        } else {
                            segments.push(BreadcrumbSegment {
                                name: tab.name.clone(),
                                full_path: file_path.clone(),
                                is_dir: false,
                            });
                        }
                    }

                    if segments.is_empty() {
                        segments.push(BreadcrumbSegment {
                            name: tab.name.clone(),
                            full_path: tab.path.clone().unwrap_or_default(),
                            is_dir: false,
                        });
                    }

                    let is_scrolled = tab.scroll_y > 0.5;
                    let path_bar_bottom = ui.cursor_y + path_bar_h;

                    let path_bg = if state.glassmorphism {
                        Color::TRANSPARENT
                    } else {
                        colors.bg_editor
                    };

                    let mut path_container = ui.container()
                        .width(width)
                        .height(path_bar_h)
                        .bg(path_bg)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .padding_left(12.0)
                        .padding_right(12.0);
                        
                    if is_scrolled && !state.glassmorphism {
                        let shadow_col = if state.theme == crate::theme::ThemeMode::Dark {
                            Color::rgba(0.0, 0.0, 0.0, 0.5)
                        } else {
                            Color::rgba(0.0, 0.0, 0.0, 0.15)
                        };
                        path_container = path_container.shadow(shadow_col, 0.0, 1.5, 4.0);
                    }

                    let mut segment_clicked: Option<(usize, f32)> = None;
                    let mut toggle_markdown_preview = false;

                    path_container.show(|ui| {
                        // Left: Breadcrumb hierarchy
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                let total_segs = segments.len();
                                for (idx, seg) in segments.iter().enumerate() {
                                    let is_last = idx == total_segs - 1;
                                    let seg_id = Id::from_u64(999998000 + (active_tab_idx as u64 * 100) + idx as u64);

                                    let is_open = state.active_breadcrumb_popup.as_ref().map(|p| {
                                        p.selected_name == seg.name
                                    }).unwrap_or(false);

                                    let seg_x = ui.cursor_x;

                                    let seg_resp = ui.container()
                                        .id(seg_id)
                                        .row()
                                        .valign(Align::Center)
                                        .gap(4.0)
                                        .radius_all(0.0)
                                        .padding(2.0, 4.0, 2.0, 4.0)
                                        .hover(Style::bg(colors.highlight))
                                        .bg(if is_open { colors.highlight } else { Color::TRANSPARENT })
                                        .show(|ui| {
                                            if is_last {
                                                let fname = tab.path.as_ref()
                                                    .and_then(|p| p.file_name())
                                                    .and_then(|n| n.to_str())
                                                    .unwrap_or(&seg.name);
                                                let file_icon_source = icon_theme.get_icon_for_file(fname, tab.path.as_deref());

                                                file_icon_source.show(ui, 11.5);

                                                ui.text(&seg.name)
                                                    .size(11.5)
                                                    .color(colors.text_primary)
                                                    .show();
                                            } else {
                                                ui.text(&seg.name)
                                                    .size(11.5)
                                                    .color(colors.text_muted)
                                                    .show();
                                            }
                                        });

                                    if !is_modal_open && seg_resp.clicked {
                                        segment_clicked = Some((idx, seg_x));
                                    }

                                    if !is_last {
                                        ui.text("\u{f054}")
                                            .size(8.0)
                                            .color(colors.text_muted)
                                            .show();
                                    }
                                }
                            });

                        // Right: Markdown Edit / Preview Toggle
                        if tab.is_markdown() {
                            let preview_btn_id = Id::from_u64(999999000 + active_tab_idx as u64);
                            let (icon, label) = if tab.is_markdown_preview {
                                ("\u{f044}", "Edit Source")
                            } else {
                                ("\u{f06e}", "Preview")
                            };
                            let toggle_resp = ui.container()
                                .id(preview_btn_id)
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .padding(2.0, 8.0, 2.0, 8.0)
                                .radius_all(4.0)
                                .bg(if tab.is_markdown_preview { colors.accent.with_alpha(0.18) } else { colors.bg_sidebar.with_alpha(0.7) })
                                .border(((if tab.is_markdown_preview { colors.accent.with_alpha(0.4) } else { colors.border.with_alpha(0.3) }), 0.5))
                                .hover(Style::bg(colors.highlight))
                                .show(|ui| {
                                    ui.text(icon).size(9.0).color(if tab.is_markdown_preview { colors.accent } else { colors.text_muted }).show();
                                    ui.text(label).size(10.5).font_weight(FontWeight::Medium).color(if tab.is_markdown_preview { colors.accent } else { colors.text_muted }).show();
                                });

                            if !is_modal_open && toggle_resp.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if !is_modal_open && toggle_resp.clicked {
                                toggle_markdown_preview = true;
                            }
                        }
                    });

                    if toggle_markdown_preview {
                        state.tabs[active_tab_idx].is_markdown_preview = !state.tabs[active_tab_idx].is_markdown_preview;
                        ui.request_redraw();
                    }

                    // Handle segment click to open/toggle popup
                    if let Some((idx, seg_x)) = segment_clicked {
                        segment_clicked_this_frame = true;
                        let seg = &segments[idx];
                        let target_dir = if seg.is_dir {
                            seg.full_path.clone()
                        } else {
                            seg.full_path.parent().unwrap_or(&state.current_dir).to_path_buf()
                        };

                        let is_already_open = state.active_breadcrumb_popup.as_ref()
                            .map(|p| p.dir_path == target_dir && p.selected_name == seg.name)
                            .unwrap_or(false);

                        if is_already_open {
                            state.active_breadcrumb_popup = None;
                        } else {
                            state.active_breadcrumb_popup = Some(crate::state::BreadcrumbPopup {
                                dir_path: target_dir,
                                selected_name: seg.name.clone(),
                                popup_x: seg_x,
                                popup_y: path_bar_bottom + 1.0,
                                expanded_dirs: std::collections::HashSet::new(),
                            });
                        }
                        ui.request_redraw();
                    }
                }

                let mut cursor_screen_pos: Option<[f32; 2]> = None;

                // ─── Editor Area ────────────────────────────────────────────
                if active_tab_idx < state.tabs.len() {
                    let editor_h = (height - tab_bar_h - path_bar_h - 1.0).max(50.0);
                    let editor_id = Id::from_u64(999996000 + active_tab_idx as u64);

                    if state.tabs[active_tab_idx].is_markdown_preview {
                        crate::ui_layout::draw_markdown_buffer_preview(
                            ui,
                            state,
                            active_tab_idx,
                            width,
                            editor_h,
                            &colors,
                        );
                    } else {

                    // Update modifiers
                    for event in &ui.input_events {
                        match event {
                            zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                                match key {
                                    winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                                        state.ctrl_pressed = true;
                                    }
                                    winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                                        state.shift_pressed = true;
                                    }
                                    winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
                                        state.alt_pressed = true;
                                    }
                                    _ => {}
                                }
                            }
                            zenthra_platform::event::PlatformEvent::KeyUp { key } => {
                                match key {
                                    winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                                        state.ctrl_pressed = false;
                                    }
                                    winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                                        state.shift_pressed = false;
                                    }
                                    winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
                                        state.alt_pressed = false;
                                    }
                                    _ => {}
                                }
                            }
                            _ => {}
                        }
                    }

                    let mut save_tab = false;
                    let mut save_tab_as = false;
                    let mut format_tab = false;
                    let mut copy_action = false;
                    let mut cut_action = false;
                    let mut paste_action = false;
                    let mut undo_action = false;
                    let mut redo_action = false;
                    let mut accept_review_action = false;
                    let mut reject_review_action = false;
                    let mut open_cmdline_action = false;
                    let has_active_review = state.active_review().is_some();
                    let is_modal_open = state.is_modal_or_dialog_open() || ui.modal_active || state.cmdline_dialog.open;
                    let is_focused = !is_modal_open
                        && state.focused_panel == crate::state::types::FocusedPanel::Editor
                        && (ui.focused_id == Some(editor_id) || ui.focused_id.is_none());
                    if is_focused && ui.focused_id.is_none() {
                        ui.focused_id = Some(editor_id);
                    }
                    if is_focused {
                        // Schedule the next redraw precisely when the cursor blink state changes,
                        // rather than blindly polling every 250ms every frame.
                        // The old approach fired a new frame every frame which re-scheduled
                        // another 250ms wake, creating a continuous 4fps loop at 39% CPU idle.
                        let elapsed_ms = state.last_cursor_activity.elapsed().as_millis();
                        let ms_until_next_blink = if elapsed_ms < 500 {
                            // Cursor is solid-on during initial 500ms grace period
                            500 - elapsed_ms
                        } else {
                            // Blink period is 530ms; calculate ms until next flip
                            530 - ((elapsed_ms - 500) % 530)
                        };
                        ui.request_redraw_after(std::time::Duration::from_millis(
                            ms_until_next_blink as u64
                        ));
                    }

                    // Handle scrollbar drag and scroll wheel before input events are taken/drained
                    let editor_x = ui.cursor_x;
                    let editor_y = ui.cursor_y;
                    let editor_w = width;

                    let is_resizing = state.is_resizing();
                    let terminal_visible = state.terminal_visible;
                    let ai_panel_visible = state.ai_panel_visible;
                    let is_occluded_by_ai = ai_panel_visible && state.ai_panel_expanded;

                    let is_near_panel_edge = (ai_panel_visible && ui.mouse_x >= editor_x + editor_w - 6.0)
                        || (terminal_visible && ui.mouse_y >= editor_y + editor_h - 6.0)
                        || is_resizing;

                    if is_resizing {
                        if let Some(drag) = ui.active_drag {
                            let scrollbar_id = Id::from_u64(999997000 + active_tab_idx as u64);
                            if drag.id == scrollbar_id {
                                ui.active_drag = None;
                            }
                        }
                    }

                    {
                        let tab_mut = &mut state.tabs[active_tab_idx];
                        let line_height_pixels = (state.font_size * state.zoom_level * 1.54).round().max(14.0);
                        let total_lines = tab_mut.editor.buffer.line_count();
                        let content_h = total_lines as f32 * line_height_pixels;
                        let max_scroll_y = (content_h - editor_h).max(0.0);

                        if max_scroll_y > 0.0 {
                            let track_h = editor_h;
                            let thumb_h = ((editor_h / content_h) * track_h).max(20.0);
                            let scrollbar_w = 6.0;
                            let scrollbar_x = editor_x + editor_w - scrollbar_w - 4.0;
                            let scrollbar_id = Id::from_u64(999997000 + active_tab_idx as u64);

                            let mut is_dragging = false;
                            if let Some(drag) = ui.active_drag {
                                if drag.id == scrollbar_id && !is_resizing {
                                    is_dragging = true;
                                    if ui.mouse_down {
                                        let ratio = if track_h > thumb_h { max_scroll_y / (track_h - thumb_h) } else { 0.0 };
                                        let delta_y = ui.mouse_y - drag.start_mouse;
                                        let new_s = (drag.start_scroll + delta_y * ratio).clamp(0.0, max_scroll_y);
                                        tab_mut.scroll_y = new_s;
                                        tab_mut.target_scroll_y = new_s;
                                        ui.needs_redraw = true;
                                    }
                                }
                            }

                            if !is_dragging && ui.clicked && !is_near_panel_edge && !is_resizing && !is_occluded_by_ai && !is_modal_open && !ui.is_occluded(scrollbar_id, ui.mouse_x, ui.mouse_y) {
                                let max_sb_hit_x = editor_x + editor_w - 5.0;
                                let max_sb_hit_y = editor_y + track_h - (if terminal_visible { 6.0 } else { 0.0 });
                                let mouse_in_track = ui.mouse_x >= scrollbar_x - 4.0 && ui.mouse_x <= max_sb_hit_x
                                    && ui.mouse_y >= editor_y && ui.mouse_y <= max_sb_hit_y;
                                if mouse_in_track {
                                    let thumb_y = (tab_mut.scroll_y / max_scroll_y) * (track_h - thumb_h);
                                    let thumb_y_on_screen = editor_y + thumb_y;
                                    let mouse_in_thumb = ui.mouse_y >= thumb_y_on_screen && ui.mouse_y <= thumb_y_on_screen + thumb_h;

                                    if mouse_in_thumb {
                                        ui.active_drag = Some(ScrollDrag {
                                            id: scrollbar_id,
                                            start_mouse: ui.mouse_y,
                                            start_scroll: tab_mut.scroll_y,
                                        });
                                    } else {
                                        let click_y_ratio = if track_h > thumb_h {
                                            (ui.mouse_y - editor_y - thumb_h / 2.0) / (track_h - thumb_h)
                                        } else {
                                            0.0
                                        };
                                        let new_scroll_y = (click_y_ratio * max_scroll_y).clamp(0.0, max_scroll_y);
                                        tab_mut.scroll_y = new_scroll_y;
                                        tab_mut.target_scroll_y = new_scroll_y;
                                        ui.active_drag = Some(ScrollDrag {
                                            id: scrollbar_id,
                                            start_mouse: ui.mouse_y,
                                            start_scroll: new_scroll_y,
                                        });
                                        ui.needs_redraw = true;
                                    }
                                }
                            }
                        }

                        let is_hovered = !is_occluded_by_ai && !is_modal_open && !ui.is_occluded(editor_id, ui.mouse_x, ui.mouse_y) && ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
                        if is_hovered {
                            let mut scroll_delta_y = 0.0;
                            ui.input_events.retain(|event| {
                                if let zenthra_platform::event::PlatformEvent::MouseWheel { delta_y, .. } = event {
                                    scroll_delta_y += *delta_y;
                                    false
                                } else {
                                    true
                                }
                            });
                            if scroll_delta_y != 0.0 {
                                let step = if scroll_delta_y.abs() <= 5.0 { scroll_delta_y * 38.0 } else { scroll_delta_y };
                                tab_mut.target_scroll_y = (tab_mut.target_scroll_y - step).clamp(0.0, max_scroll_y);
                                ui.needs_redraw = true;
                            }
                        }

                        // Smooth kinetic dampening (lerp towards target_scroll_y)
                        let scroll_diff = tab_mut.target_scroll_y - tab_mut.scroll_y;
                        if scroll_diff.abs() > 0.5 {
                            tab_mut.scroll_y += scroll_diff * 0.35;
                            ui.needs_redraw = true;
                        } else {
                            tab_mut.scroll_y = tab_mut.target_scroll_y;
                        }
                    }

                    let mut auto_scroll_to_cursor = false;

                    {
                        let tab = &mut state.tabs[active_tab_idx];

                        // Process input events if editor is focused and no modal is open
                        if is_focused && !is_modal_open {
                            let events = std::mem::take(&mut ui.input_events);
                            let mut changed = false;

                            // We keep MouseWheel events preserved for hovered side panels/terminal
                            let (wheel_events, other_events): (Vec<_>, Vec<_>) = events.into_iter().partition(|e| {
                                matches!(e, zenthra_platform::event::PlatformEvent::MouseWheel { .. })
                            });

                            for event in &other_events {
                                state.last_cursor_activity = std::time::Instant::now();
                                match event {
                                    zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                                        if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                                            if state.vim_mode && state.editor_vim_normal {
                                                if *c == ':' {
                                                    if state.sidebar_visible && state.sidebar_view == crate::state::SidebarView::Explorer {
                                                        open_cmdline_action = true;
                                                    }
                                                    changed = true;
                                                    continue;
                                                } else if *c == 'i' || *c == 'a' {
                                                    state.editor_vim_normal = false;
                                                    changed = true;
                                                    continue;
                                                } else if *c == 'h' {
                                                    tab.editor.move_left();
                                                    changed = true;
                                                    continue;
                                                } else if *c == 'l' {
                                                    tab.editor.move_right();
                                                    changed = true;
                                                    continue;
                                                } else if *c == 'j' {
                                                    tab.editor.move_down();
                                                    changed = true;
                                                    continue;
                                                } else if *c == 'k' {
                                                    tab.editor.move_up();
                                                    changed = true;
                                                    continue;
                                                }
                                            }
                                            tab.editor.insert(&c.to_string(), tab.path.as_deref(), Some(&state.indent_calc));
                                            changed = true;
                                            let cur = tab.editor.cursor();
                                            state.completion_state.populate_instant(&tab.editor.buffer, tab.path.as_deref(), cur);
                                            state.completion_active = state.completion_state.active;
                                            if let Some(ref p) = tab.path {
                                                let content = tab.editor.buffer.to_string();
                                                state.lsp_manager.on_document_change(p, &content);
                                                state.lsp_manager.request_completion(p, cur.row as u32, cur.column as u32);
                                            }
                                            ui.request_redraw();
                                        }
                                    }
                                    zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                                        match key {
                                            winit::keyboard::KeyCode::Escape => {
                                                if state.completion_state.active || state.completion_active {
                                                    state.completion_state.dismiss();
                                                    state.completion_active = false;
                                                    state.completion_items.clear();
                                                    changed = true;
                                                }
                                                if state.snippet_session.is_some() {
                                                    state.snippet_session = None;
                                                    let cur = tab.editor.cursor();
                                                    tab.editor.set_cursor(cur);
                                                    changed = true;
                                                }
                                                if state.vim_mode {
                                                    state.editor_vim_normal = true;
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Space if state.ctrl_pressed => {
                                                let cur = tab.editor.cursor();
                                                state.completion_state.populate_instant(&tab.editor.buffer, tab.path.as_deref(), cur);
                                                state.completion_state.active = !state.completion_state.items.is_empty();
                                                state.completion_active = state.completion_state.active;
                                                if let Some(ref p) = tab.path {
                                                    state.lsp_manager.request_completion(p, cur.row as u32, cur.column as u32);
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Backspace => {
                                                if state.ctrl_pressed && has_active_review {
                                                    reject_review_action = true;
                                                    changed = true;
                                                } else if state.ctrl_pressed {
                                                    tab.editor.delete_word_left();
                                                    changed = true;
                                                    let cur = tab.editor.cursor();
                                                    state.completion_state.populate_instant(&tab.editor.buffer, tab.path.as_deref(), cur);
                                                    state.completion_active = state.completion_state.active;
                                                    if state.completion_active {
                                                        if let Some(ref p) = tab.path {
                                                            let content = tab.editor.buffer.to_string();
                                                            state.lsp_manager.on_document_change(p, &content);
                                                            state.lsp_manager.request_completion(p, cur.row as u32, cur.column as u32);
                                                        }
                                                    }
                                                    ui.request_redraw();
                                                } else {
                                                    tab.editor.backspace();
                                                    changed = true;
                                                    let cur = tab.editor.cursor();
                                                    state.completion_state.populate_instant(&tab.editor.buffer, tab.path.as_deref(), cur);
                                                    state.completion_active = state.completion_state.active;
                                                    if state.completion_active {
                                                        if let Some(ref p) = tab.path {
                                                            let content = tab.editor.buffer.to_string();
                                                            state.lsp_manager.on_document_change(p, &content);
                                                            state.lsp_manager.request_completion(p, cur.row as u32, cur.column as u32);
                                                        }
                                                    }
                                                    ui.request_redraw();
                                                }
                                            }
                                            winit::keyboard::KeyCode::Delete => {
                                                if state.ctrl_pressed {
                                                    tab.editor.delete_word_right();
                                                    changed = true;
                                                } else {
                                                    tab.editor.delete();
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Enter => {
                                                if state.ctrl_pressed && has_active_review {
                                                    accept_review_action = true;
                                                    changed = true;
                                                } else if state.completion_state.active && !state.completion_state.items.is_empty() {
                                                    let prefix = state.completion_state.prefix.clone();
                                                    let prev_expr = state.completion_state.prev_expr.clone();
                                                    if let Some(item) = state.completion_state.items.get(state.completion_state.selected_idx).cloned() {
                                                        state.snippet_session = crate::completion::commit_completion(
                                                            &mut tab.editor,
                                                            &item,
                                                            &prefix,
                                                            prev_expr.as_deref(),
                                                            item.is_postfix,
                                                            tab.path.as_deref(),
                                                            Some(&state.indent_calc),
                                                        );
                                                    }
                                                    state.completion_state.dismiss();
                                                    state.completion_active = false;
                                                    state.completion_items.clear();
                                                    changed = true;
                                                    ui.request_redraw();
                                                } else if let Some(session) = state.snippet_session.take() {
                                                    if tab.editor.cursor().row < session.target_point.row {
                                                        tab.editor.set_cursor(session.target_point);
                                                    } else {
                                                        tab.editor.insert("\n", tab.path.as_deref(), Some(&state.indent_calc));
                                                    }
                                                    changed = true;
                                                    ui.request_redraw();
                                                } else {
                                                    tab.editor.insert("\n", tab.path.as_deref(), Some(&state.indent_calc));
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Tab => {
                                                if state.completion_state.active && !state.completion_state.items.is_empty() {
                                                    let prefix = state.completion_state.prefix.clone();
                                                    let prev_expr = state.completion_state.prev_expr.clone();
                                                    if let Some(item) = state.completion_state.items.get(state.completion_state.selected_idx).cloned() {
                                                        state.snippet_session = crate::completion::commit_completion(
                                                            &mut tab.editor,
                                                            &item,
                                                            &prefix,
                                                            prev_expr.as_deref(),
                                                            item.is_postfix,
                                                            tab.path.as_deref(),
                                                            Some(&state.indent_calc),
                                                        );
                                                    }
                                                    state.completion_state.dismiss();
                                                    state.completion_active = false;
                                                    state.completion_items.clear();
                                                    changed = true;
                                                    ui.request_redraw();
                                                } else if let Some(session) = state.snippet_session.take() {
                                                    tab.editor.set_cursor(session.target_point);
                                                    changed = true;
                                                    ui.request_redraw();
                                                } else {
                                                    tab.editor.insert("    ", tab.path.as_deref(), Some(&state.indent_calc));
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::ArrowLeft => {
                                                if state.ctrl_pressed && state.shift_pressed {
                                                    tab.editor.extend_selection_word_left();
                                                } else if state.ctrl_pressed {
                                                    tab.editor.move_word_left();
                                                } else if state.shift_pressed {
                                                    tab.editor.extend_selection_left();
                                                } else {
                                                    tab.editor.move_left();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::ArrowRight => {
                                                if state.ctrl_pressed && state.shift_pressed {
                                                    tab.editor.extend_selection_word_right();
                                                } else if state.ctrl_pressed {
                                                    tab.editor.move_word_right();
                                                } else if state.shift_pressed {
                                                    tab.editor.extend_selection_right();
                                                } else {
                                                    tab.editor.move_right();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::ArrowUp => {
                                                if state.completion_state.active && !state.completion_state.items.is_empty() {
                                                    state.completion_state.selected_idx = state.completion_state.selected_idx.saturating_sub(1);
                                                    changed = true;
                                                } else {
                                                    if state.ctrl_pressed && state.shift_pressed {
                                                        tab.editor.extend_selection_up_by(5);
                                                    } else if state.ctrl_pressed {
                                                        tab.editor.move_up_by(5);
                                                    } else if state.shift_pressed {
                                                        tab.editor.extend_selection_up();
                                                    } else {
                                                        tab.editor.move_up();
                                                    }
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::ArrowDown => {
                                                if state.completion_state.active && !state.completion_state.items.is_empty() {
                                                    state.completion_state.selected_idx = (state.completion_state.selected_idx + 1).min(state.completion_state.items.len().saturating_sub(1));
                                                    changed = true;
                                                } else {
                                                    if state.ctrl_pressed && state.shift_pressed {
                                                        tab.editor.extend_selection_down_by(5);
                                                    } else if state.ctrl_pressed {
                                                        tab.editor.move_down_by(5);
                                                    } else if state.shift_pressed {
                                                        tab.editor.extend_selection_down();
                                                    } else {
                                                        tab.editor.move_down();
                                                    }
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::PageUp => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_up_by(10);
                                                } else {
                                                    tab.editor.move_up_by(10);
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::PageDown => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_down_by(10);
                                                } else {
                                                    tab.editor.move_down_by(10);
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Home => {
                                                if state.ctrl_pressed && state.shift_pressed {
                                                    tab.editor.extend_selection_to_top();
                                                } else if state.ctrl_pressed {
                                                    tab.editor.move_to_top();
                                                } else if state.shift_pressed {
                                                    tab.editor.extend_selection_to_line_start();
                                                } else {
                                                    tab.editor.move_to_line_start();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::End => {
                                                if state.ctrl_pressed && state.shift_pressed {
                                                    tab.editor.extend_selection_to_bottom();
                                                } else if state.ctrl_pressed {
                                                    tab.editor.move_to_bottom();
                                                } else if state.shift_pressed {
                                                    tab.editor.extend_selection_to_line_end();
                                                } else {
                                                    tab.editor.move_to_line_end();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::KeyD if state.ctrl_pressed && state.shift_pressed => {
                                                tab.editor.duplicate_line_down();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::KeyD if state.ctrl_pressed => {
                                                tab.editor.select_word_at_cursor();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::KeyK if state.ctrl_pressed && state.shift_pressed => {
                                                tab.editor.delete_line();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::KeyA
                                                if state.ctrl_pressed && !state.shift_pressed =>
                                            {
                                                tab.editor.select_all();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::KeyS if state.ctrl_pressed && state.shift_pressed => {
                                                save_tab_as = true;
                                            }
                                            winit::keyboard::KeyCode::KeyS if state.ctrl_pressed => {
                                                save_tab = true;
                                            }
                                            winit::keyboard::KeyCode::KeyF if state.ctrl_pressed && state.shift_pressed => {
                                                format_tab = true;
                                            }
                                            winit::keyboard::KeyCode::KeyC if state.ctrl_pressed => {
                                                copy_action = true;
                                            }
                                            winit::keyboard::KeyCode::KeyX if state.ctrl_pressed => {
                                                cut_action = true;
                                            }
                                            winit::keyboard::KeyCode::KeyV if state.ctrl_pressed => {
                                                paste_action = true;
                                            }
                                            winit::keyboard::KeyCode::KeyZ if state.ctrl_pressed && state.shift_pressed => {
                                                redo_action = true;
                                            }
                                            winit::keyboard::KeyCode::KeyZ if state.ctrl_pressed => {
                                                undo_action = true;
                                            }
                                            winit::keyboard::KeyCode::KeyY if state.ctrl_pressed => {
                                                redo_action = true;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if changed {
                                let tab_idx = state.active_tab.unwrap_or(0);
                                state.on_buffer_changed(tab_idx);
                                auto_scroll_to_cursor = true;
                                ui.needs_redraw = true;
                            }
                            ui.input_events = wheel_events;
                        }
                    }

                    if copy_action {
                        state.copy_selection();
                        ui.request_redraw();
                    }
                    if cut_action {
                        state.cut_selection();
                        auto_scroll_to_cursor = true;
                        ui.request_redraw();
                    }
                    if paste_action {
                        state.paste_clipboard();
                        auto_scroll_to_cursor = true;
                        ui.request_redraw();
                    }
                    if undo_action {
                        state.undo_active_tab();
                        auto_scroll_to_cursor = true;
                        ui.request_redraw();
                    }
                    if redo_action {
                        state.redo_active_tab();
                        auto_scroll_to_cursor = true;
                        ui.request_redraw();
                    }
                    if save_tab {
                        state.save_active_tab();
                        ui.request_redraw();
                    }
                    if save_tab_as {
                        state.save_active_tab_as();
                        ui.request_redraw();
                    }
                    if format_tab {
                        state.format_active_tab();
                        ui.request_redraw();
                    }
                    if accept_review_action {
                        state.accept_current_review();
                        ui.request_redraw();
                    } else if reject_review_action {
                        state.reject_current_review();
                        ui.request_redraw();
                    }
                    if open_cmdline_action {
                        state.open_cmdline(Some(""), None);
                        ui.request_redraw();
                    }

                    // Layout positions
                    let editor_x = ui.cursor_x;
                    let editor_y = ui.cursor_y;
                    let editor_w = width;

                    // Dimensions of typography
                    let eff_font_size = (state.font_size * state.zoom_level).max(8.0);
                    let line_height_pixels = (eff_font_size * 1.54).round().max(14.0);
                    let total_lines = state.tabs[active_tab_idx].editor.buffer.line_count();

                    let active_diff_line_states: Option<Vec<DiffLineState>> = state.tabs.get(active_tab_idx).and_then(|tab| {
                        let tab_path = tab.path.as_ref();
                        state.pending_reviews.iter().find(|r| {
                            tab_path.map(|tp| path_matches(tp, &r.path)).unwrap_or(false)
                                || tab.name == r.path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                        })
                    }).or_else(|| state.active_review())
                      .map(|r| {
                          let mut states: Vec<DiffLineState> = r.diff_lines.iter().map(|(_, s)| *s).collect();
                          if r.is_new_file || (r.total_removed_lines() == 0 && !states.iter().any(|s| *s == DiffLineState::Added)) {
                              let count = state.tabs.get(active_tab_idx).map(|t| t.editor.buffer.line_count()).unwrap_or(states.len());
                              states = vec![DiffLineState::Added; count];
                          }
                          states
                      });

                    let content_h = total_lines as f32 * line_height_pixels;

                    let active_blame_annotation = if state.git_blame_inline {
                        let cursor_row = state.tabs.get(active_tab_idx).map(|t| t.editor.cursor().row).unwrap_or(0);
                        if let Some(tab_path) = state.tabs.get(active_tab_idx).and_then(|t| t.path.as_deref()) {
                            state.git_blame_manager.get_line_blame(tab_path, &state.current_dir, cursor_row)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    let mut hovered_blame_trigger: Option<(crate::state::GitBlameLine, f32, f32, f32)> = None;
                    let mut clicked_blame: Option<crate::state::GitBlameLine> = None;

                    let tab_mut = &mut state.tabs[active_tab_idx];
                    let cursor_p = tab_mut.editor.cursor();

                    // Auto-scroll viewport so cursor row is visible ONLY when navigating/typing
                    if auto_scroll_to_cursor {
                        let cursor_top = cursor_p.row as f32 * line_height_pixels;
                        let cursor_bottom = cursor_top + line_height_pixels;

                        if cursor_top < tab_mut.scroll_y {
                            tab_mut.scroll_y = cursor_top;
                            tab_mut.target_scroll_y = cursor_top;
                        } else if cursor_bottom > tab_mut.scroll_y + editor_h - 10.0 {
                            let max_scroll = (content_h - editor_h).max(0.0);
                            let s = (cursor_bottom - editor_h + 10.0).clamp(0.0, max_scroll);
                            tab_mut.scroll_y = s;
                            tab_mut.target_scroll_y = s;
                        }
                    }

                    let scroll_y = tab_mut.scroll_y;

                    // Pre-cache the visible line range before entering the render loop closure
                    let start_y_visible = scroll_y;
                    let end_y_visible = scroll_y + editor_h;
                    let visible_start = ((start_y_visible.max(0.0) / line_height_pixels) as usize).min(total_lines);
                    let visible_end = ((end_y_visible.max(0.0) / line_height_pixels) as usize + 2).min(total_lines);

                    tab_mut.editor.buffer.ensure_range_cached(visible_start..(visible_end + 2).min(total_lines + 1));

                    // Sync highlighter theme — only when theme actually changes.
                    // Previously this called set_theme() unconditionally every frame, which
                    // cleared highlight_cache on every render, forcing a full tree-sitter
                    // reparse + rope-to-String allocation on every single frame.
                    let want_dark = state.theme != crate::theme::ThemeMode::Light;
                    if state.highlighter.theme_is_dark() != Some(want_dark) {
                        let syntax_theme = if want_dark {
                            syntax::SyntaxTheme::dark()
                        } else {
                            syntax::SyntaxTheme::light()
                        };
                        state.highlighter.set_theme(syntax_theme, want_dark);
                    }

                    let dummy_path = tab_mut.path.clone().unwrap_or_else(|| {
                        std::path::PathBuf::from(format!("{}.rs", tab_mut.name))
                    });

                    // Run the syntax highlighter for the visible viewport
                    let highlights = state.highlighter.highlight_viewport(
                        tab_mut.editor.buffer.rope(),
                        tab_mut.version,
                        Some(&dummy_path),
                        visible_start,
                        (visible_end + 2).min(total_lines),
                    );

                    let active_tab_path = tab_mut.path.clone();
                    let mut line_diagnostics: std::collections::HashMap<u32, DiagnosticSeverity> = std::collections::HashMap::new();
                    if let Some(ref tp) = active_tab_path {
                        for (p, diags) in &state.lsp_manager.diagnostics {
                            let matches = p == tp || crate::state::path_matches(p, tp);
                            if matches {
                                for d in diags {
                                    if let Some(sev) = d.severity {
                                        let line = d.range.start.line;
                                        line_diagnostics.entry(line)
                                            .and_modify(|existing| {
                                                if *existing != DiagnosticSeverity::Error && sev == DiagnosticSeverity::Error {
                                                    *existing = DiagnosticSeverity::Error;
                                                } else if *existing != DiagnosticSeverity::Error && *existing != DiagnosticSeverity::Warning && sev == DiagnosticSeverity::Warning {
                                                    *existing = DiagnosticSeverity::Warning;
                                                }
                                            })
                                            .or_insert(sev);
                                    }
                                }
                            }
                        }
                    }

                    let mut active_cursor_screen_pos: Option<[f32; 2]> = None;
                    let cursor_style = state.cursor_style;
                    let cur_color = state.get_cursor_color(colors.accent);

                    // Container wrapper - not scrollable, but clips
                    ui.container()
                        .id(editor_id)
                        .width(editor_w)
                        .height(editor_h)
                        .clip(true)
                        .show(|ui| {
                            let tab = &mut state.tabs[active_tab_idx];
                            // Evict only entries far outside the current viewport instead of
                            // clearing everything. For a 400-line file scrolled around, the old
                            // code would nuke all ~50 cached shapes whenever the cache hit 1000
                            // entries. Now we keep only lines near the visible range.
                            if tab.shaping_cache.len() > 1500 {
                                let keep_start = visible_start.saturating_sub(50);
                                let keep_end = visible_end + 50;
                                tab.shaping_cache.retain(|line, _| *line >= keep_start && *line < keep_end);
                            }

                            let draw_start = ui.draws.len();

                            let clip = [editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h];

                            // 1. Draw gutter background
                            let gutter_w = if state.line_numbers_visible { 44.0 } else { 8.0 };
                            if state.line_numbers_visible {
                                let gutter_color = if state.glassmorphism {
                                    Color::TRANSPARENT
                                } else {
                                    colors.bg_gutter
                                };
                                if gutter_color.a > 0.001 {
                                    ui.draws.push(DrawCommand::Rect(RectDraw {
                                        instance: RectInstance {
                                            pos: [editor_x, editor_y],
                                            size: [gutter_w, editor_h],
                                            color: gutter_color.to_array(),
                                            radius: [0.0; 4],
                                            clip_rect: clip,
                                            ..Default::default()
                                        }
                                    }));
                                }
                            }

                            if let Some(fs) = ui.font_system.as_ref() {
                                let adapter = state.font_adapter.get_or_insert_with(|| {
                                    let mut a = CosmicFontProvider::new_with_system(fs.clone());
                                    a.set_layout_size(10000.0, 10000.0);
                                    a
                                });

                                let sub_line_offset = (scroll_y as f64 % line_height_pixels as f64) as f32;
                                for i in visible_start..visible_end {
                                    let line_y = editor_y + ((i.saturating_sub(visible_start)) as f32 * line_height_pixels) - sub_line_offset;
                                    let cursor_p = tab.editor.cursor();
                                    let diff_line_state = active_diff_line_states.as_ref().and_then(|s| s.get(i).copied());

                                    // Line highlighting for diff lines (- in red, + in green)
                                    match diff_line_state {
                                        Some(DiffLineState::Added) => {
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [editor_x + gutter_w, line_y],
                                                    size: [editor_w - gutter_w, line_height_pixels],
                                                    color: Color::rgba(0.18, 0.63, 0.26, 0.18).to_array(),
                                                    radius: [0.0; 4],
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                }
                                            }));
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [editor_x, line_y],
                                                    size: [3.5, line_height_pixels],
                                                    color: Color::rgb(0.24, 0.78, 0.35).to_array(),
                                                    radius: [0.0; 4],
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                }
                                            }));
                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                text: "+".to_string(),
                                                pos: [editor_x + gutter_w - 9.0, line_y + 3.0],
                                                options: TextOptions::new().size(12.0).font_family("monospace").color(Color::rgb(0.35, 0.85, 0.45)),
                                                clip,
                                            }));
                                        }
                                        Some(DiffLineState::Removed) => {
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [editor_x + gutter_w, line_y],
                                                    size: [editor_w - gutter_w, line_height_pixels],
                                                    color: Color::rgba(0.97, 0.32, 0.29, 0.22).to_array(),
                                                    radius: [0.0; 4],
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                }
                                            }));
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [editor_x, line_y],
                                                    size: [3.5, line_height_pixels],
                                                    color: Color::rgb(0.97, 0.32, 0.29).to_array(),
                                                    radius: [0.0; 4],
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                }
                                            }));
                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                text: "-".to_string(),
                                                pos: [editor_x + gutter_w - 9.0, line_y + 3.0],
                                                options: TextOptions::new().size(12.0).font_family("monospace").color(Color::rgb(0.97, 0.45, 0.45)),
                                                clip,
                                            }));
                                        }
                                        _ => {
                                            if cursor_p.row == i && is_focused {
                                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                                    instance: RectInstance {
                                                        pos: [editor_x + gutter_w, line_y],
                                                        size: [editor_w - gutter_w, line_height_pixels],
                                                        color: Color::rgba(1.0, 1.0, 1.0, 0.04).to_array(),
                                                        radius: [0.0; 4],
                                                        clip_rect: clip,
                                                        ..Default::default()
                                                    }
                                                }));
                                            }
                                        }
                                    }

                                    // Draw line number
                                    if state.line_numbers_visible {
                                        let num_str = format!("{}", i + 1);
                                        let num_x = editor_x + gutter_w - 16.0 - (num_str.len() as f32 * 6.5);
                                        let num_color = if diff_line_state == Some(DiffLineState::Added) {
                                            Color::rgb(0.40, 0.85, 0.50)
                                        } else if diff_line_state == Some(DiffLineState::Removed) {
                                            Color::rgb(0.97, 0.45, 0.45)
                                        } else if cursor_p.row == i && is_focused {
                                            colors.text_primary
                                        } else {
                                            colors.text_dim
                                        };

                                        ui.draws.push(DrawCommand::Text(TextDraw {
                                            text: num_str,
                                            pos: [num_x, line_y + 3.0],
                                            options: TextOptions::new().size((eff_font_size - 1.0).max(10.0)).font_family("monospace").color(num_color),
                                            clip,
                                        }));
                                    }

                                    // Draw diagnostic indicator in gutter
                                    if let Some(sev) = line_diagnostics.get(&(i as u32)) {
                                        let dot_col = match sev {
                                            DiagnosticSeverity::Error => Color::rgb(0.95, 0.28, 0.28),
                                            DiagnosticSeverity::Warning => Color::rgb(0.92, 0.68, 0.22),
                                            _ => Color::rgb(0.35, 0.65, 0.95),
                                        };
                                        let dot_x = if state.line_numbers_visible { editor_x + 5.0 } else { editor_x + 2.0 };
                                        ui.draws.push(DrawCommand::Rect(RectDraw {
                                            instance: RectInstance {
                                                pos: [dot_x, line_y + 7.0],
                                                size: [6.0, 6.0],
                                                color: dot_col.to_array(),
                                                radius: [3.0; 4],
                                                clip_rect: clip,
                                                ..Default::default()
                                            }
                                        }));
                                    }

                                    // Shape code line (cached - zero buffer allocation if line matches)
                                    let opts = TextOptions::new().size(eff_font_size).font_family("monospace").color(colors.text_primary);
                                    let is_cache_valid = if let Some((cached_text, _)) = tab.shaping_cache.get(&i) {
                                        tab.editor.buffer.line_matches(i, cached_text)
                                    } else {
                                        false
                                    };
                                    if !is_cache_valid {
                                        let line_text = tab.editor.buffer.line(i).unwrap_or_default();
                                        let clean = line_text.trim_end_matches(&['\r', '\n'][..]).to_string();
                                        let shaped = adapter.shape(&clean, &opts);
                                        tab.shaping_cache.insert(i, (clean, std::sync::Arc::new(shaped)));
                                    }
                                    let (clean_text_storage, shaped) = {
                                        let entry = tab.shaping_cache.get(&i).unwrap();
                                        (entry.0.clone(), entry.1.clone())
                                    };
                                    let clean_text = clean_text_storage.as_str();

                                    // Draw selections
                                    for selection in &tab.editor.selections {
                                        if !selection.is_empty() {
                                            let (start_p, end_p) = selection.range();
                                            if i >= start_p.row && i <= end_p.row {
                                                let start_col = if i == start_p.row { start_p.column } else { 0 };
                                                let end_col = if i == end_p.row { end_p.column } else { clean_text.chars().count() };

                                                let start_byte = clean_text.chars().take(start_col).map(|c| c.len_utf8()).sum();
                                                let end_byte = clean_text.chars().take(end_col).map(|c| c.len_utf8()).sum();

                                                let start_x = find_x_for_byte(&shaped, clean_text, start_byte);
                                                let end_x = find_x_for_byte(&shaped, clean_text, end_byte);

                                                let sel_w = (end_x - start_x).max(8.0);
                                                let sel_color = colors.accent.with_alpha(0.28);

                                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                                    instance: RectInstance {
                                                        pos: [editor_x + gutter_w + start_x, line_y],
                                                        size: [sel_w, line_height_pixels],
                                                        color: sel_color.to_array(),
                                                        radius: [0.0; 4],
                                                        clip_rect: clip,
                                                        ..Default::default()
                                                    }
                                                }));
                                            }
                                        }
                                    }

                                    // Draw code line text itself
                                    if diff_line_state == Some(DiffLineState::Removed) {
                                        let removed_opts = TextOptions::new().size(eff_font_size).font_family("monospace").color(Color::rgb(0.97, 0.50, 0.50));
                                        ui.draws.push(DrawCommand::Text(TextDraw {
                                            text: clean_text.to_string(),
                                            pos: [editor_x + gutter_w, line_y + 2.0],
                                            options: removed_opts,
                                            clip,
                                        }));
                                    } else {
                                        let line_spans = highlights.get(&i);
                                        if let Some(spans) = line_spans {
                                            let styled_spans = get_styled_spans(clean_text, spans, colors.text_primary, &shaped);
                                            for span in styled_spans {
                                                let span_opts = TextOptions::new()
                                                    .size(eff_font_size)
                                                    .font_family("monospace")
                                                    .color(span.color);
                                                ui.draws.push(DrawCommand::Text(TextDraw {
                                                    text: span.text,
                                                    pos: [editor_x + gutter_w + span.x_offset, line_y + 2.0],
                                                    options: span_opts,
                                                    clip,
                                                }));
                                            }
                                        } else {
                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                text: clean_text.to_string(),
                                                pos: [editor_x + gutter_w, line_y + 2.0],
                                                options: opts,
                                                clip,
                                            }));
                                        }
                                    }

                                    // Draw blinking cursor
                                    if cursor_p.row == i && is_focused {
                                        let cursor_byte = clean_text.chars().take(cursor_p.column).map(|c| c.len_utf8()).sum();
                                        let cursor_x = find_x_for_byte(&shaped, clean_text, cursor_byte);
                                        active_cursor_screen_pos = Some([editor_x + gutter_w + cursor_x, line_y + line_height_pixels]);

                                        let elapsed_ms = state.last_cursor_activity.elapsed().as_millis();
                                        let show_cursor = if elapsed_ms < 500 {
                                            true
                                        } else {
                                            ((elapsed_ms - 500) / 530) % 2 == 0
                                        };
                                        if show_cursor {
                                            let cursor_h = (eff_font_size + 3.0).min(line_height_pixels).max(14.0);
                                            let cursor_y = line_y + ((line_height_pixels - cursor_h) / 2.0).round();

                                            match cursor_style {
                                                crate::state::types::CursorStyle::Block => {
                                                    let char_under_cursor = clean_text.chars().nth(cursor_p.column);
                                                    let char_w = if let Some(c) = char_under_cursor {
                                                        let b_start: usize = clean_text.chars().take(cursor_p.column).map(|ch| ch.len_utf8()).sum();
                                                        let b_end = b_start + c.len_utf8();
                                                        let x_end = find_x_for_byte(&shaped, clean_text, b_end);
                                                        (x_end - cursor_x).max(8.0)
                                                    } else {
                                                        (eff_font_size * 0.6).round().max(8.0)
                                                    };

                                                    // 1. Draw block cursor vertically centered in the line
                                                    ui.draws.push(DrawCommand::Rect(RectDraw {
                                                        instance: RectInstance {
                                                            pos: [editor_x + gutter_w + cursor_x, cursor_y],
                                                            size: [char_w, cursor_h],
                                                            color: cur_color.to_array(),
                                                            radius: [0.0; 4],
                                                            clip_rect: clip,
                                                            ..Default::default()
                                                        }
                                                    }));

                                                    // 2. Render character under cursor in inverted high-contrast color!
                                                    if let Some(c) = char_under_cursor {
                                                        if c != ' ' && c != '\t' && c != '\n' && c != '\r' {
                                                            let inv_color = if cur_color.r * 0.299 + cur_color.g * 0.587 + cur_color.b * 0.114 > 0.5 {
                                                                colors.bg_base
                                                            } else {
                                                                Color::rgb(1.0, 1.0, 1.0)
                                                            };
                                                            let opts = TextOptions::new()
                                                                .size(eff_font_size)
                                                                .font_family("monospace")
                                                                .color(inv_color);
                                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                                text: c.to_string(),
                                                                pos: [editor_x + gutter_w + cursor_x, line_y + 2.0],
                                                                options: opts,
                                                                clip,
                                                            }));
                                                        }
                                                    }
                                                }
                                                crate::state::types::CursorStyle::Line => {
                                                    ui.draws.push(DrawCommand::Rect(RectDraw {
                                                        instance: RectInstance {
                                                            pos: [editor_x + gutter_w + cursor_x, cursor_y],
                                                            size: [2.0, cursor_h],
                                                            color: cur_color.to_array(),
                                                            radius: [0.0; 4],
                                                            clip_rect: clip,
                                                            ..Default::default()
                                                        }
                                                    }));
                                                }
                                                crate::state::types::CursorStyle::Underline => {
                                                    let char_under_cursor = clean_text.chars().nth(cursor_p.column);
                                                    let char_w = if let Some(c) = char_under_cursor {
                                                        let b_start: usize = clean_text.chars().take(cursor_p.column).map(|ch| ch.len_utf8()).sum();
                                                        let b_end = b_start + c.len_utf8();
                                                        let x_end = find_x_for_byte(&shaped, clean_text, b_end);
                                                        (x_end - cursor_x).max(8.0)
                                                    } else {
                                                        (eff_font_size * 0.6).round().max(8.0)
                                                    };
                                                    let under_y = cursor_y + cursor_h - 2.0;
                                                    ui.draws.push(DrawCommand::Rect(RectDraw {
                                                        instance: RectInstance {
                                                            pos: [editor_x + gutter_w + cursor_x, under_y],
                                                            size: [char_w, 2.0],
                                                            color: cur_color.to_array(),
                                                            radius: [0.0; 4],
                                                            clip_rect: clip,
                                                            ..Default::default()
                                                        }
                                                    }));
                                                }
                                            }

                                            // Draw inline ghost text (LazyVim style virtual text preview)
                                            if state.completion_state.active && !state.completion_state.items.is_empty() {
                                                if let Some(sel_item) = state.completion_state.items.get(state.completion_state.selected_idx) {
                                                    let full_text = sel_item.insert_text.as_deref().unwrap_or(&sel_item.label);
                                                    let clean_full = full_text.strip_suffix('~').unwrap_or(full_text);
                                                    let prefix = &state.completion_state.prefix;
                                                    let ghost_text = if clean_full.starts_with(prefix) {
                                                        &clean_full[prefix.len()..]
                                                    } else {
                                                        clean_full
                                                    };
                                                    let first_line_ghost = ghost_text.lines().next().unwrap_or("");
                                                    if !first_line_ghost.is_empty() {
                                                        let ghost_opts = TextOptions::new()
                                                            .size(eff_font_size)
                                                            .font_family("monospace")
                                                            .color(colors.text_dim.with_alpha(0.50));
                                                        ui.draws.push(DrawCommand::Text(TextDraw {
                                                            text: first_line_ghost.to_string(),
                                                            pos: [editor_x + gutter_w + cursor_x + 1.0, line_y + 2.0],
                                                            options: ghost_opts,
                                                            clip,
                                                        }));
                                                    }
                                                }
                                            }
                                        }
                                    }

                                    // Draw inline git blame annotation on the active cursor line (Zed style)
                                    if cursor_p.row == i {
                                        if let Some(ref blame) = active_blame_annotation {
                                            let line_end_byte = clean_text.len();
                                            let line_end_x = find_x_for_byte(&shaped, clean_text, line_end_byte);
                                            let blame_x = editor_x + gutter_w + line_end_x + 32.0;

                                            let blame_str = format!("\u{ea68} {}, {}", blame.author, blame.time_ago);
                                            let pill_w = (blame_str.chars().count() as f32 * 7.2).max(45.0);

                                            let is_blame_hov = ui.mouse_x >= blame_x - 6.0
                                                && ui.mouse_x <= blame_x + pill_w + 6.0
                                                && ui.mouse_y >= line_y
                                                && ui.mouse_y <= line_y + line_height_pixels;

                                            if is_blame_hov {
                                                ui.cursor_icon = CursorIcon::Pointer;
                                                hovered_blame_trigger = Some((blame.clone(), blame_x, line_y, pill_w));

                                                // Draw subtle interactive pill highlight background
                                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                                    instance: RectInstance {
                                                        pos: [blame_x - 6.0, line_y + 1.0],
                                                        size: [pill_w + 12.0, line_height_pixels - 2.0],
                                                        color: colors.highlight.with_alpha(0.35).to_array(),
                                                        radius: [4.0; 4],
                                                        clip_rect: clip,
                                                        ..Default::default()
                                                    },
                                                }));

                                                if ui.clicked {
                                                    clicked_blame = Some(blame.clone());
                                                }
                                            }

                                            let text_color = if is_blame_hov {
                                                colors.text_primary.with_alpha(0.90)
                                            } else {
                                                colors.text_muted.with_alpha(0.42)
                                            };

                                            let blame_opts = TextOptions::new()
                                                .size((eff_font_size - 1.0).max(11.0))
                                                .color(text_color);

                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                text: blame_str,
                                                pos: [blame_x, line_y + 2.0],
                                                options: blame_opts,
                                                clip,
                                            }));
                                        }
                                    }
                                }
                            }

                            // Draw custom scrollbar thumb if content is taller than viewport
                            let max_scroll_y = (content_h - editor_h).max(0.0);
                            if max_scroll_y > 0.0 {
                                let track_h = editor_h;
                                let thumb_h = ((editor_h / content_h) * track_h).max(20.0);
                                let thumb_y = (scroll_y / max_scroll_y) * (track_h - thumb_h);

                                let scrollbar_w = 6.0;
                                let scrollbar_x = editor_x + editor_w - scrollbar_w - 4.0;

                                let scrollbar_id = Id::from_u64(999997000 + active_tab_idx as u64);
                                let is_dragging_thumb = ui.active_drag.map(|d| d.id == scrollbar_id).unwrap_or(false);
                                let is_hovering_thumb = ui.mouse_x >= scrollbar_x - 6.0 && ui.mouse_x <= scrollbar_x + scrollbar_w + 6.0
                                    && ui.mouse_y >= editor_y + thumb_y && ui.mouse_y <= editor_y + thumb_y + thumb_h;

                                let thumb_color = if is_dragging_thumb {
                                    Color::rgba(1.0, 1.0, 1.0, 0.5)
                                } else if is_hovering_thumb {
                                    Color::rgba(1.0, 1.0, 1.0, 0.38)
                                } else {
                                    Color::rgba(1.0, 1.0, 1.0, 0.22)
                                };

                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                    instance: RectInstance {
                                        pos: [scrollbar_x, editor_y + thumb_y],
                                        size: [scrollbar_w, thumb_h],
                                        color: thumb_color.to_array(),
                                        radius: [3.0; 4],
                                        clip_rect: clip,
                                        ..Default::default()
                                    }
                                }));
                            }

                            // Draw drop-shadow overlay at top of editor when scrolled
                            if scroll_y > 0.5 {
                                let shadow_col = if state.theme == crate::theme::ThemeMode::Dark {
                                    [0.0, 0.0, 0.0, 0.55]
                                } else {
                                    [0.0, 0.0, 0.0, 0.15]
                                };
                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                    instance: RectInstance {
                                        pos: [editor_x, editor_y],
                                        size: [editor_w, 1.0],
                                        color: [0.0, 0.0, 0.0, 0.0],
                                        shadow_color: shadow_col,
                                        shadow_offset: [0.0, 1.0],
                                        shadow_blur: 3.0,
                                        clip_rect: clip,
                                        ..Default::default()
                                    },
                                }));
                            }

                            // Advance the layout in immediate-mode container context
                            ui.advance(editor_w, editor_h, draw_start);
                        });

                    let gutter_w = if state.line_numbers_visible { 44.0 } else { 8.0 };
                    cursor_screen_pos = active_cursor_screen_pos.or_else(|| {
                        let cur = state.tabs.get(active_tab_idx)?.editor.cursor();
                        let sub_line_offset = (scroll_y as f64 % line_height_pixels as f64) as f32;
                        let line_y = editor_y + ((cur.row.saturating_sub(visible_start)) as f32 * line_height_pixels) - sub_line_offset;
                        if line_y >= editor_y && line_y <= editor_y + editor_h {
                            Some([editor_x + gutter_w + (cur.column as f32 * 7.8), line_y + line_height_pixels])
                        } else {
                            None
                        }
                    });

                    // Set focus and mouse selection logic on click
                    let is_hovered = !is_modal_open && ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
                    let right_clicked_editor = ui.right_clicked
                        && is_hovered
                        && !is_near_panel_edge
                        && !(state.ai_panel_visible && state.ai_panel_expanded)
                        && !state.is_resizing();
                    if right_clicked_editor {
                        let root = state.current_dir.clone();
                        let target_path = state.tabs[active_tab_idx]
                            .path
                            .clone()
                            .unwrap_or_else(|| root.clone());
                        state.panel_context_menu = Some(crate::state::PanelContextMenu {
                            target: crate::state::PanelContextMenuTarget::Editor,
                            target_is_dir: target_path.is_dir(),
                            target_path,
                            terminal_index: None,
                            menu_x: ui.mouse_x,
                            menu_y: ui.mouse_y,
                        });
                        ui.consume_click();
                        ui.consume_right_click();
                        ui.focused_id = Some(editor_id);
                        state.focused_panel = crate::state::FocusedPanel::Editor;
                        ui.request_redraw();
                    }

                    // Handle clicking inline git blame pill
                    if let Some(ref blame) = clicked_blame {
                        ui.consume_click();
                        if !blame.is_uncommitted {
                            state.sidebar_visible = true;
                            state.sidebar_view = crate::state::types::SidebarView::Git;
                            state.set_clipboard(blame.commit_hash.clone());
                            let short_h = if blame.commit_hash.len() >= 7 { &blame.commit_hash[..7] } else { &blame.commit_hash };
                            state.set_status(format!("Commit {} • {} (copied hash)", short_h, blame.summary));
                        } else {
                            state.sidebar_visible = true;
                            state.sidebar_view = crate::state::types::SidebarView::Git;
                            state.set_status("Uncommitted changes in working tree".to_string());
                        }
                        ui.request_redraw();
                    }

                    // Update active blame hover card state
                    if let Some((blame, bx, by, pw)) = hovered_blame_trigger {
                        let card_w = 340.0;
                        let card_h = 105.0;
                        let card_x = bx.clamp(editor_x + 8.0, (editor_x + editor_w - card_w - 8.0).max(editor_x + 8.0));
                        let card_y = if by - card_h - 6.0 >= editor_y {
                            by - card_h - 6.0
                        } else {
                            by + line_height_pixels + 6.0
                        };

                        state.active_blame_hover_card = Some(crate::state::types::BlameHoverCardState {
                            author: blame.author,
                            time_ago: blame.time_ago,
                            summary: blame.summary,
                            commit_hash: blame.commit_hash,
                            card_x,
                            card_y,
                            card_w,
                            card_h,
                            trigger_rect: [bx - 6.0, by, pw + 12.0, line_height_pixels],
                        });
                    } else if let Some(ref card) = state.active_blame_hover_card {
                        let in_trigger = ui.mouse_in_rect(card.trigger_rect[0], card.trigger_rect[1], card.trigger_rect[2], card.trigger_rect[3]);
                        let in_card = ui.mouse_in_rect(card.card_x - 4.0, card.card_y - 4.0, card.card_w + 8.0, card.card_h + 8.0);
                        if !in_trigger && !in_card {
                            state.active_blame_hover_card = None;
                        }
                    }

                    if ui.clicked && is_hovered && !right_clicked_editor && clicked_blame.is_none() {
                        // Avoid positioning the cursor if clicking on the scrollbar or panel edges
                        let scrollbar_w = 6.0;
                        let scrollbar_x = editor_x + editor_w - scrollbar_w - 4.0;
                        let max_scroll_y = (content_h - editor_h).max(0.0);
                        let mouse_in_scrollbar = max_scroll_y > 0.0
                            && ui.mouse_x >= scrollbar_x - 6.0
                            && ui.mouse_x <= scrollbar_x + scrollbar_w + 6.0;
                        let is_near_panel_edge = (state.ai_panel_visible && ui.mouse_x >= editor_x + editor_w - 8.0)
                            || (state.terminal_visible && ui.mouse_y >= editor_y + editor_h - 8.0)
                            || state.is_resizing();

                        let is_occluded_by_ai = state.ai_panel_visible && state.ai_panel_expanded;
                        if !mouse_in_scrollbar && !is_near_panel_edge && !is_occluded_by_ai {
                            let tab = &mut state.tabs[active_tab_idx];
                            ui.focused_id = Some(editor_id);
                            state.focused_panel = crate::state::types::FocusedPanel::Editor;
                            ui.needs_redraw = true;

                            let rel_y = (ui.mouse_y - (editor_y + ui.offset_y)) as f64 + tab.scroll_y as f64;
                            let line_idx = ((rel_y / line_height_pixels as f64).max(0.0) as usize).min(total_lines.saturating_sub(1));

                            let gutter_w = if state.line_numbers_visible { 44.0 } else { 8.0 };
                            let rel_x = ui.mouse_x - (editor_x + ui.offset_x) - gutter_w;
                            let rel_x = rel_x.max(0.0);

                            let mut col_idx = 0;
                            if let Some(fs) = ui.font_system.as_ref() {
                                let opts = TextOptions::new().size(state.font_size).font_family("monospace");
                                let is_cache_valid = if let Some((cached_text, _)) = tab.shaping_cache.get(&line_idx) {
                                    tab.editor.buffer.line_matches(line_idx, cached_text)
                                } else {
                                    false
                                };
                                if !is_cache_valid {
                                    let adapter = state.font_adapter.get_or_insert_with(|| {
                                        let mut a = CosmicFontProvider::new_with_system(fs.clone());
                                        a.set_layout_size(10000.0, 10000.0);
                                        a
                                    });
                                    let line_text = tab.editor.buffer.line(line_idx).unwrap_or_default();
                                    let clean = line_text.trim_end_matches(&['\r', '\n'][..]).to_string();
                                    let shaped = adapter.shape(&clean, &opts);
                                    tab.shaping_cache.insert(line_idx, (clean, std::sync::Arc::new(shaped)));
                                }
                                let (clean_text_storage, shaped) = {
                                    let entry = tab.shaping_cache.get(&line_idx).unwrap();
                                    (entry.0.clone(), entry.1.clone())
                                };
                                let clean_text = clean_text_storage.as_str();
                                let byte_idx = find_byte_for_x(&shaped, clean_text, rel_x);
                                col_idx = clean_text[..byte_idx.min(clean_text.len())].chars().count();
                            }

                            if state.shift_pressed {
                                let saved = tab.editor.selection().start;
                                tab.editor.set_selection(Selection::new(saved, Point::new(line_idx, col_idx)));
                            } else {
                                tab.editor.set_cursor(Point::new(line_idx, col_idx));
                            }
                            state.snippet_session = None;
                            state.last_cursor_activity = std::time::Instant::now();
                        }
                    }
                    }
                }

            // ─── Render Breadcrumb Dropdown Popup Overlay ─────────────────
                if let Some(popup) = state.active_breadcrumb_popup.clone() {
                    let mut file_to_open = None;
                    let mut toggle_dir_path = None;
                    let mut dismiss_popup = false;

                    let mut items: Vec<crate::state::FileTreeNode> = Vec::new();
                    collect_popup_tree(&popup.dir_path, 0, &popup.expanded_dirs, &mut items);

                    let popup_w = 260.0;
                    let row_h = 24.0;
                    let max_h = 320.0;
                    let popup_h = (items.len() as f32 * (row_h + 2.0) + 12.0).clamp(36.0, max_h);

                    let mut px = popup.popup_x;
                    if px + popup_w > ui.width - 8.0 {
                        px = (ui.width - popup_w - 8.0).max(8.0);
                    }
                    px = px.max(8.0);
                    let py = popup.popup_y;

                    let is_mouse_inside = ui.mouse_x >= px && ui.mouse_x <= px + popup_w
                        && ui.mouse_y >= py && ui.mouse_y <= py + popup_h;

                    // Dismiss if clicking outside popup and not on breadcrumb segment
                    if ui.clicked && !is_mouse_inside && !segment_clicked_this_frame {
                        dismiss_popup = true;
                    }

                    let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);
                    let popup_bg = colors.bg_sidebar;

                    let popup_id = Id::from_u64(999998888);

                    ui.overlay(|ui| {
                        ui.container()
                            .raw_id(popup_id)
                            .pos(px, py)
                            .overlay()
                            .width(popup_w)
                            .height(popup_h)
                            .scroll_y(true)
                            .bg(popup_bg)
                            .border((colors.border, 1.0))
                            .radius_all(0.0)
                            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.4), 0.0, 4.0, 8.0)
                            .padding(4.0, 4.0, 4.0, 4.0)
                            .column()
                            .gap(1.0)
                            .show(|ui| {
                                if items.is_empty() {
                                    ui.container()
                                        .fill_x()
                                        .height(row_h)
                                        .padding_left(8.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("(Empty folder)")
                                                .size(11.0)
                                                .color(colors.text_dim)
                                                .show();
                                        });
                                } else {
                                    for (idx, node) in items.iter().enumerate() {
                                        let is_dir = node.is_dir;
                                        let depth = node.depth;
                                        let indent = depth as f32 * 14.0;
                                        let item_id = Id::from_u64(999998900 + idx as u64);

                                        let icon_source = if is_dir {
                                            icon_theme.get_icon_for_folder(&node.name, node.is_expanded)
                                        } else {
                                            icon_theme.get_icon_for_file(&node.name, Some(&node.path))
                                        };

                                        let is_selected = node.name == popup.selected_name;

                                        let mut item_container = ui.container()
                                            .id(item_id)
                                            .fill_x()
                                            .row()
                                            .valign(Align::Center)
                                            .padding_left(8.0 + indent)
                                            .padding_top(3.0)
                                            .padding_bottom(3.0)
                                            .gap(6.0)
                                            .radius_all(0.0);

                                        if is_selected {
                                            item_container = item_container
                                                .bg(Color::rgba(18.0 / 255.0, 68.0 / 255.0, 105.0 / 255.0, 1.0));
                                        } else {
                                            item_container = item_container
                                                .bg(Color::TRANSPARENT)
                                                .hover(Style::bg(colors.highlight));
                                        }

                                        let resp = item_container.show(|ui| {
                                            let text_color = if is_selected {
                                                colors.text_primary
                                            } else {
                                                colors.text_primary
                                            };

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

                                            icon_source.show(ui, 13.0);

                                            ui.text(&node.name)
                                                .size(11.5)
                                                .color(text_color)
                                                .show();
                                        });

                                        if resp.clicked {
                                            if is_dir {
                                                toggle_dir_path = Some(node.path.clone());
                                            } else {
                                                file_to_open = Some(node.path.clone());
                                            }
                                        }
                                    }
                                }
                            });
                    });

                    if let Some(fpath) = file_to_open {
                        state.open_file(&fpath);
                        state.active_breadcrumb_popup = None;
                        ui.request_redraw();
                    } else if let Some(dpath) = toggle_dir_path {
                        if let Some(ref mut popup_state) = state.active_breadcrumb_popup {
                            if popup_state.expanded_dirs.contains(&dpath) {
                                popup_state.expanded_dirs.remove(&dpath);
                            } else {
                                popup_state.expanded_dirs.insert(dpath);
                            }
                        }
                        ui.request_redraw();
                    } else if dismiss_popup {
                        state.active_breadcrumb_popup = None;
                        ui.request_redraw();
                    }
                }

                // ─── Render IntelliSense Autocomplete Floating Popup Overlay ──────────
                if state.completion_state.active && !state.completion_state.items.is_empty() {
                    if let Some([cur_x, cur_y]) = cursor_screen_pos {
                        let ev = crate::completion::ui::draw_completion_popup(
                            ui,
                            &state.completion_state,
                            &colors,
                            (cur_x, cur_y),
                        );
                        match ev {
                            crate::completion::ui::CompletionUiEvent::Commit(idx) => {
                                let prefix = state.completion_state.prefix.clone();
                                let prev_expr = state.completion_state.prev_expr.clone();
                                if let Some(item) = state.completion_state.items.get(idx).cloned() {
                                    let tab_path = state.tabs[active_tab_idx].path.clone();
                                    state.snippet_session = crate::completion::commit_completion(
                                        &mut state.tabs[active_tab_idx].editor,
                                        &item,
                                        &prefix,
                                        prev_expr.as_deref(),
                                        item.is_postfix,
                                        tab_path.as_deref(),
                                        Some(&state.indent_calc),
                                    );
                                    state.on_buffer_changed(active_tab_idx);
                                    state.completion_state.dismiss();
                                    state.completion_active = false;
                                    state.completion_items.clear();
                                    ui.request_redraw();
                                }
                            }
                            crate::completion::ui::CompletionUiEvent::Dismiss => {
                                state.completion_state.dismiss();
                                state.completion_active = false;
                                state.completion_items.clear();
                                ui.request_redraw();
                            }
                            crate::completion::ui::CompletionUiEvent::None => {}
                        }
                    }
                }
            }
        });

    draw_editor_context_menu(ui, state, &colors);
    draw_blame_hover_card(ui, state, &colors);
}

#[derive(Clone, Copy)]
enum EditorContextAction {
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Format,
    ViewGit,
    File(crate::state::FileContextAction),
}

fn draw_editor_context_menu(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: &crate::theme::ThemeColors,
) {
    let Some(menu) = state.panel_context_menu.clone() else {
        return;
    };
    if menu.target != crate::state::PanelContextMenuTarget::Editor {
        return;
    }

    let menu_w = 210.0;
    let has_active_tab = state.active_tab.and_then(|idx| state.tabs.get(idx)).is_some();
    let menu_h = if has_active_tab {
        17.0 * 26.0 + 5.0 + 22.0 + 8.0
    } else {
        10.0 * 26.0 + 3.0 + 13.0 + 8.0
    };
    let menu_x = menu.menu_x.clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
    let menu_y = menu.menu_y.clamp(8.0, (ui.height - menu_h - 8.0).max(8.0));
    let mouse_in_menu = ui.mouse_in_rect(menu_x, menu_y, menu_w, menu_h);
    let escape_pressed = ui.input_events.iter().any(|event| {
        matches!(
            event,
            zenthra_platform::event::PlatformEvent::KeyDown {
                key: winit::keyboard::KeyCode::Escape
            }
        )
    });
    if escape_pressed {
        state.panel_context_menu = None;
        ui.input_events.retain(|event| {
            !matches!(
                event,
                zenthra_platform::event::PlatformEvent::KeyDown {
                    key: winit::keyboard::KeyCode::Escape
                }
            )
        });
        ui.request_redraw();
        return;
    }
    if (ui.clicked || ui.right_clicked) && !mouse_in_menu {
        state.panel_context_menu = None;
        ui.consume_click();
        ui.consume_right_click();
        ui.request_redraw();
        return;
    }

    let has_selection = state
        .active_tab
        .and_then(|idx| state.tabs.get(idx))
        .map(|tab| tab.editor.selection().start != tab.editor.selection().end)
        .unwrap_or(false);
    let target_path = menu.target_path.clone();
    let target_is_dir = menu.target_is_dir;
    let root = state.current_dir.clone();
    let has_file_target = target_path != root;
    let has_file_clipboard = state.tree_clipboard.is_some();
    let mut action = None;

    ui.overlay(|ui| {
        ui.container()
            .raw_id(Id::from_u64(999997200))
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
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997208), "\u{ea7f}", "New File...", "", true, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::NewFile));
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997209), "\u{ea80}", "New Folder...", "", state.has_workspace, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::NewFolder));
                }
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                if has_active_tab {
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997201), "\u{f0e2}", "Undo", "Ctrl+Z", true, false, colors) {
                        action = Some(EditorContextAction::Undo);
                    }
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997202), "\u{f01e}", "Redo", "Ctrl+Y", true, false, colors) {
                        action = Some(EditorContextAction::Redo);
                    }
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997203), "\u{f0c4}", "Cut", "Ctrl+X", has_selection, false, colors) {
                        action = Some(EditorContextAction::Cut);
                    }
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997204), "\u{f0c5}", "Copy", "Ctrl+C", has_selection, false, colors) {
                        action = Some(EditorContextAction::Copy);
                    }
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997205), "\u{f0ea}", "Paste", "Ctrl+V", true, false, colors) {
                        action = Some(EditorContextAction::Paste);
                    }
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997206), "\u{f0c5}", "Select All", "Ctrl+A", true, false, colors) {
                        action = Some(EditorContextAction::SelectAll);
                    }
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997207), "\u{f0ad}", "Format Document", "Ctrl+Shift+F", true, false, colors) {
                        action = Some(EditorContextAction::Format);
                    }
                    if super::sidebar::draw_menu_item(ui, Id::from_u64(999997230), "\u{ea68}", "View in Git Panel", "", true, false, colors) {
                        action = Some(EditorContextAction::ViewGit);
                    }
                    ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997210), "\u{f044}", "Rename File...", "F2", has_file_target, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::Rename));
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997211), "\u{f014}", "Delete File", "Del", has_file_target, true, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::Delete));
                }
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997212), "\u{f0c4}", "Cut File", "", has_file_target, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::Cut));
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997213), "\u{f0c5}", "Copy File", "", has_file_target, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::Copy));
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997214), "\u{f0ea}", "Paste File", "", has_file_clipboard, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::Paste));
                }
                ui.container().fill_x().height(1.0).bg(colors.border).show(|_| {});
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997215), "\u{f0c1}", "Copy Path", "", true, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::CopyPath));
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997216), "⎘", "Copy Relative Path", "", true, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::CopyRelativePath));
                }
                if super::sidebar::draw_menu_item(ui, Id::from_u64(999997217), "\u{f08e}", "Reveal in File Manager", "", true, false, colors) {
                    action = Some(EditorContextAction::File(crate::state::FileContextAction::RevealInFileManager));
                }
            });
    });

    if let Some(action) = action {
        state.panel_context_menu = None;
        match action {
            EditorContextAction::File(file_action) => {
                state.run_file_context_action(file_action, &target_path, target_is_dir);
                if matches!(file_action, crate::state::FileContextAction::NewFile | crate::state::FileContextAction::NewFolder) {
                    state.focused_panel = crate::state::FocusedPanel::Explorer;
                    ui.focused_id = None;
                }
            }
            action => if let Some(idx) = state.active_tab {
                ui.focused_id = Some(Id::from_u64(999996000 + idx as u64));
                state.focused_panel = crate::state::FocusedPanel::Editor;
                match action {
                EditorContextAction::Undo => state.undo_active_tab(),
                EditorContextAction::Redo => state.redo_active_tab(),
                EditorContextAction::Cut => state.cut_selection(),
                EditorContextAction::Copy => state.copy_selection(),
                EditorContextAction::Paste => state.paste_clipboard(),
                EditorContextAction::SelectAll => {
                    if let Some(tab) = state.tabs.get_mut(idx) {
                        tab.editor.select_all();
                    }
                }
                EditorContextAction::Format => state.format_active_tab(),
                EditorContextAction::ViewGit => {
                    state.sidebar_visible = true;
                    state.sidebar_view = crate::state::types::SidebarView::Git;
                }
                EditorContextAction::File(_) => unreachable!(),
                }
            }
        }
        ui.request_redraw();
    }
}

fn draw_blame_hover_card(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: &crate::theme::ThemeColors,
) {
    let Some(ref card) = state.active_blame_hover_card else {
        return;
    };

    let card_x = card.card_x;
    let card_y = card.card_y;
    let card_w = card.card_w;
    let card_h = card.card_h;
    let author = card.author.clone();
    let time_ago = card.time_ago.clone();
    let summary = card.summary.clone();
    let commit_hash = card.commit_hash.clone();
    let short_hash = if commit_hash.len() >= 7 {
        commit_hash[..7].to_string()
    } else {
        commit_hash.clone()
    };

    let mut open_git_panel = false;
    let mut copy_hash = false;

    let card_id = Id::from_u64(999997450);

    ui.overlay(|ui| {
        ui.container()
            .raw_id(card_id)
            .pos(card_x, card_y)
            .overlay()
            .width(card_w)
            .height(card_h)
            .bg(colors.bg_sidebar)
            .border((colors.border.with_alpha(0.60), 1.0))
            .radius_all(8.0)
            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.45), 0.0, 6.0, 18.0)
            .padding(10.0, 12.0, 10.0, 12.0)
            .column()
            .gap(6.0)
            .show(|ui| {
                // 1. Header: Author, relative time, and short hash pill
                ui.container()
                    .fill_x()
                    .row()
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .show(|ui| {
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text("\u{ea68}")
                                    .size(13.0)
                                    .color(colors.accent)
                                    .show();
                                ui.text(&author)
                                    .size(12.5)
                                    .font_weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                                ui.text("•")
                                    .size(11.0)
                                    .color(colors.text_muted.with_alpha(0.6))
                                    .show();
                                ui.text(&time_ago)
                                    .size(11.5)
                                    .color(colors.text_muted)
                                    .show();
                            });

                        // Hash Pill (interactive)
                        let hash_btn_id = Id::from_u64(999997451);
                        let hash_hov = ui.interaction_state.get(&hash_btn_id).copied().unwrap_or(0.0) > 0.5;
                        let hash_btn = ui.container()
                            .raw_id(hash_btn_id)
                            .interactive(true)
                            .padding(2.0, 6.0, 2.0, 6.0)
                            .radius_all(4.0)
                            .bg(if hash_hov { colors.highlight } else { colors.border.with_alpha(0.35) })
                            .row()
                            .valign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                ui.text(&short_hash)
                                    .size(11.0)
                                    .font_weight(FontWeight::SemiBold)
                                    .color(colors.accent)
                                    .show();
                                ui.text("\u{f0c5}")
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            });
                        if hash_btn.clicked {
                            copy_hash = true;
                        }
                    });

                // 2. Commit message / summary
                if !summary.is_empty() {
                    ui.container()
                        .fill_x()
                        .padding_top(2.0)
                        .padding_bottom(2.0)
                        .show(|ui| {
                            ui.text(&summary)
                                .size(12.0)
                                .color(colors.text_primary)
                                .wrap(true)
                                .show();
                        });
                }

                // Divider line
                ui.container()
                    .fill_x()
                    .height(1.0)
                    .bg(colors.border.with_alpha(0.30))
                    .show(|_| {});

                // 3. Actions Row
                ui.container()
                    .fill_x()
                    .row()
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .show(|ui| {
                        // Open in Git Panel button
                        let git_btn_id = Id::from_u64(999997452);
                        let git_hov = ui.interaction_state.get(&git_btn_id).copied().unwrap_or(0.0) > 0.5;
                        let git_btn = ui.container()
                            .raw_id(git_btn_id)
                            .interactive(true)
                            .padding(3.0, 8.0, 3.0, 8.0)
                            .radius_all(4.0)
                            .bg(if git_hov { colors.accent.with_alpha(0.20) } else { colors.border.with_alpha(0.25) })
                            .row()
                            .valign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                ui.text("\u{ea68}")
                                    .size(11.5)
                                    .color(colors.accent)
                                    .show();
                                ui.text("Open in Git Panel")
                                    .size(11.0)
                                    .font_weight(FontWeight::Medium)
                                    .color(colors.text_primary)
                                    .show();
                            });
                        if git_btn.clicked {
                            open_git_panel = true;
                        }

                        // Copy Full Hash button
                        let copy_btn_id = Id::from_u64(999997453);
                        let copy_hov = ui.interaction_state.get(&copy_btn_id).copied().unwrap_or(0.0) > 0.5;
                        let copy_btn = ui.container()
                            .raw_id(copy_btn_id)
                            .interactive(true)
                            .padding(3.0, 8.0, 3.0, 8.0)
                            .radius_all(4.0)
                            .bg(if copy_hov { colors.highlight } else { Color::TRANSPARENT })
                            .row()
                            .valign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                ui.text("\u{f0c5}")
                                    .size(11.0)
                                    .color(colors.text_muted)
                                    .show();
                                ui.text("Copy Full Hash")
                                    .size(11.0)
                                    .color(colors.text_muted)
                                    .show();
                            });
                        if copy_btn.clicked {
                            copy_hash = true;
                        }
                    });
            });
    });

    if open_git_panel {
        state.sidebar_visible = true;
        state.sidebar_view = crate::state::types::SidebarView::Git;
        state.active_blame_hover_card = None;
        ui.request_redraw();
    }
    if copy_hash {
        state.set_clipboard(commit_hash.clone());
        state.set_status(format!("Copied hash {}", short_hash));
        state.active_blame_hover_card = None;
        ui.request_redraw();
    }
}


fn collect_popup_tree(
    dir: &std::path::Path,
    depth: usize,
    expanded_dirs: &std::collections::HashSet<PathBuf>,
    items: &mut Vec<crate::state::FileTreeNode>,
) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
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

            if name.starts_with('.') && name != ".gitignore" {
                continue;
            }

            let is_dir = path.is_dir();
            let is_expanded = is_dir && expanded_dirs.contains(&path);

            items.push(crate::state::FileTreeNode {
                path: path.clone(),
                name,
                depth,
                is_dir,
                is_expanded,
                is_last: false,
                guide_prefix: String::new(),
                git_status: None,
            });

            if is_expanded {
                collect_popup_tree(&path, depth + 1, expanded_dirs, items);
            }
        }
    }
}

fn find_x_for_byte(shaped: &ShapedBuffer, clean_text: &str, byte_idx: usize) -> f32 {
    let mut char_w = 7.81;
    for g in shaped.glyphs() {
        if g.width > 4.0 && g.width < 20.0 {
            char_w = g.width;
            break;
        }
    }

    if shaped.glyphs().is_empty() {
        let col = clean_text[..byte_idx.min(clean_text.len())].chars().count();
        return col as f32 * char_w;
    }

    let mut best_x = 0.0;
    let mut found = false;
    for glyph in shaped.glyphs() {
        if glyph.cluster >= byte_idx {
            best_x = glyph.x;
            found = true;
            break;
        }
    }

    if !found {
        if let Some(last) = shaped.glyphs().last() {
            let last_cluster = last.cluster;
            if byte_idx > last_cluster && byte_idx <= clean_text.len() {
                let extra_chars = clean_text[last_cluster..byte_idx].chars().count().saturating_sub(1);
                best_x = last.x + last.width + extra_chars as f32 * char_w;
            } else {
                best_x = last.x + last.width;
            }
        } else {
            let col = clean_text[..byte_idx.min(clean_text.len())].chars().count();
            best_x = col as f32 * char_w;
        }
    }

    best_x
}

fn find_byte_for_x(shaped: &ShapedBuffer, clean_text: &str, rel_x: f32) -> usize {
    if rel_x <= 0.0 || clean_text.is_empty() {
        return 0;
    }

    let mut char_w = 7.81;
    for g in shaped.glyphs() {
        if g.width > 4.0 && g.width < 20.0 {
            char_w = g.width;
            break;
        }
    }

    let glyphs = shaped.glyphs();
    if glyphs.is_empty() {
        let col = ((rel_x + char_w * 0.5) / char_w).floor() as usize;
        return clean_text.chars().take(col).map(|c| c.len_utf8()).sum();
    }

    for (i, glyph) in glyphs.iter().enumerate() {
        let midpoint = glyph.x + glyph.width * 0.5;
        if rel_x < midpoint {
            return glyph.cluster;
        }

        let is_last = i + 1 >= glyphs.len();
        if is_last {
            return clean_text.len();
        }

        let next_glyph = &glyphs[i + 1];
        let next_midpoint = next_glyph.x + next_glyph.width * 0.5;
        if rel_x < next_midpoint {
            return next_glyph.cluster;
        }
    }

    clean_text.len()
}

struct StyledSpan {
    pub text: String,
    pub x_offset: f32,
    pub color: Color,
}

fn push_styled_span(result: &mut Vec<StyledSpan>, text: String, x_offset: f32, color: Color) {
    if text.is_empty() {
        return;
    }
    if let Some(last) = result.last_mut() {
        if last.color == color {
            last.text.push_str(&text);
            return;
        }
    }
    result.push(StyledSpan { text, x_offset, color });
}

fn get_styled_spans(
    line_text: &str,
    spans: &[(usize, usize, Color)],
    default_color: Color,
    shaped: &ShapedBuffer,
) -> Vec<StyledSpan> {
    let mut result = Vec::new();
    if spans.is_empty() {
        if !line_text.is_empty() {
            result.push(StyledSpan { text: line_text.to_string(), x_offset: 0.0, color: default_color });
        }
        return result;
    }

    let is_ascii = line_text.is_ascii();
    let char_to_byte = |char_offset: usize| -> usize {
        if char_offset == 0 { return 0; }
        if is_ascii {
            return char_offset.min(line_text.len());
        }
        line_text.char_indices()
            .nth(char_offset)
            .map(|(b, _)| b)
            .unwrap_or(line_text.len())
    };

    let char_count = if is_ascii { line_text.len() } else { line_text.chars().count() };
    let mut last_char = 0;

    for &(start_char, end_char, color) in spans {
        let start_char = start_char.min(char_count);
        let end_char = end_char.min(char_count);

        if start_char > last_char {
            let gap_start_byte = char_to_byte(last_char);
            let gap_end_byte = char_to_byte(start_char);
            let text = line_text[gap_start_byte..gap_end_byte].to_string();
            let x_offset = find_x_for_byte(shaped, line_text, gap_start_byte);
            push_styled_span(&mut result, text, x_offset, default_color);
        }

        if end_char > start_char {
            let start_byte = char_to_byte(start_char);
            let end_byte = char_to_byte(end_char);
            let text = line_text[start_byte..end_byte].to_string();
            let x_offset = find_x_for_byte(shaped, line_text, start_byte);
            push_styled_span(&mut result, text, x_offset, color);
        }

        last_char = end_char;
    }

    if last_char < char_count {
        let tail_start_byte = char_to_byte(last_char);
        let text = line_text[tail_start_byte..].to_string();
        let x_offset = find_x_for_byte(shaped, line_text, tail_start_byte);
        push_styled_span(&mut result, text, x_offset, default_color);
    }

    if result.is_empty() && !line_text.is_empty() {
        result.push(StyledSpan { text: line_text.to_string(), x_offset: 0.0, color: default_color });
    }

    result
}
