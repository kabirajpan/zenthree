use std::path::PathBuf;
use crate::state::{ZenthreeState, path_matches, DiffLineState};
use crate::lsp::protocol::DiagnosticSeverity;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context};
use zenthra_widgets::ui::{DrawCommand, TextDraw, RectDraw, ScrollDrag};
use zenthra_render::RectInstance;
use zenthra_text::prelude::{CosmicFontProvider, TextOptions, ShapedBuffer};
use editor::buffer::Point;
use editor::selection::Selection;

pub fn draw_main_buffer(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    // Drain any fresh completions delivered by LSP
    if let Some(items) = state.lsp_manager.latest_completions.take() {
        if !items.is_empty() {
            state.completion_items = items;
            state.completion_selected_idx = 0;
            state.completion_active = true;
        } else {
            state.completion_items.clear();
            state.completion_active = false;
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
                            .weight(FontWeight::Bold)
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
                                                        .weight(FontWeight::Bold)
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
                let mut active_tab_idx = state.active_tab.unwrap_or(0);

                let tab_bar_h = 26.0;

                // ─── Tab Bar ────────────────────────────────────────────────
                ui.container()
                    .width(width)
                    .height(tab_bar_h)
                    .bg(colors.bg_inactive_tab)
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
                            let bg = if is_active { colors.bg_active_tab } else { colors.bg_inactive_tab };
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
                                .border(colors.border, 1.0)
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .padding_left(9.0)
                                .padding_right(6.0)
                                .hover_bg(colors.highlight)
                                .show(|ui| {
                                    icon_source.show(ui, 12.0);

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
                                        .hover_bg(Color::rgb(30.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0))
                                        .text_color(colors.text_primary)
                                        .radius_all(3.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(9.0)
                                        .show();

                                    if close_resp.clicked {
                                        close_clicked = true;
                                    }
                                });

                            if close_clicked {
                                tab_to_close = Some(idx);
                            } else if resp.clicked {
                                tab_to_activate = Some(idx);
                            }
                        }

                        if let Some(idx) = tab_to_close {
                            state.close_tab(idx);
                            active_tab_idx = state.active_tab.unwrap_or(0);
                            ui.request_redraw();
                        } else if let Some(idx) = tab_to_activate {
                            state.set_active_tab(Some(idx));
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
                    .bg(colors.border)
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

                    let mut path_container = ui.container()
                        .width(width)
                        .height(path_bar_h)
                        .bg(colors.bg_editor)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .padding_left(12.0)
                        .padding_right(12.0)
                        .no_wrap();

                    if is_scrolled {
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
                                        .hover_bg(colors.highlight)
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

                                    if seg_resp.clicked {
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
                                .border(if tab.is_markdown_preview { colors.accent.with_alpha(0.4) } else { colors.border.with_alpha(0.3) }, 0.5)
                                .hover_bg(colors.highlight)
                                .show(|ui| {
                                    ui.text(icon).size(9.0).color(if tab.is_markdown_preview { colors.accent } else { colors.text_muted }).show();
                                    ui.text(label).size(10.5).weight(FontWeight::Medium).color(if tab.is_markdown_preview { colors.accent } else { colors.text_muted }).show();
                                });

                            if toggle_resp.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if toggle_resp.clicked {
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
                    let has_active_review = state.active_review().is_some();
                    let is_focused = ui.focused_id == Some(editor_id);
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
                        let line_height_pixels = (state.font_size * 1.54).round().max(14.0);
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
                                        tab_mut.scroll_y = (drag.start_scroll + delta_y * ratio).clamp(0.0, max_scroll_y);
                                        ui.needs_redraw = true;
                                    }
                                }
                            }

                            if !is_dragging && ui.clicked && !is_near_panel_edge && !is_resizing && !is_occluded_by_ai {
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

                        let is_hovered = !is_occluded_by_ai && ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
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
                                tab_mut.scroll_y = (tab_mut.scroll_y - step).clamp(0.0, max_scroll_y);
                                ui.needs_redraw = true;
                            }
                        }
                    }

                    let mut auto_scroll_to_cursor = false;

                    {
                        let tab = &mut state.tabs[active_tab_idx];

                        // Process input events if editor is focused
                        if is_focused {
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
                                            tab.editor.insert(&c.to_string(), tab.path.as_deref(), Some(&state.indent_calc));
                                            changed = true;
                                            if let Some(ref p) = tab.path {
                                                let cur = tab.editor.cursor();
                                                state.lsp_manager.request_completion(p, cur.row as u32, cur.column as u32);
                                            }
                                        }
                                    }
                                    zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                                        match key {
                                            winit::keyboard::KeyCode::Escape => {
                                                if state.completion_active {
                                                    state.completion_active = false;
                                                    state.completion_items.clear();
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Space if state.ctrl_pressed => {
                                                if let Some(ref p) = tab.path {
                                                    let cur = tab.editor.cursor();
                                                    state.lsp_manager.request_completion(p, cur.row as u32, cur.column as u32);
                                                    state.completion_active = true;
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Backspace => {
                                                if state.ctrl_pressed && has_active_review {
                                                    reject_review_action = true;
                                                    changed = true;
                                                } else {
                                                    tab.editor.backspace();
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Delete => {
                                                tab.editor.delete();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Enter => {
                                                if state.ctrl_pressed && has_active_review {
                                                    accept_review_action = true;
                                                    changed = true;
                                                } else if state.completion_active && !state.completion_items.is_empty() {
                                                    if let Some(item) = state.completion_items.get(state.completion_selected_idx) {
                                                        let to_insert = item.insert_text.as_deref().unwrap_or(&item.label);
                                                        tab.editor.insert(to_insert, tab.path.as_deref(), Some(&state.indent_calc));
                                                    }
                                                    state.completion_active = false;
                                                    state.completion_items.clear();
                                                    changed = true;
                                                } else {
                                                    tab.editor.insert("\n", tab.path.as_deref(), Some(&state.indent_calc));
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::Tab => {
                                                if state.completion_active && !state.completion_items.is_empty() {
                                                    if let Some(item) = state.completion_items.get(state.completion_selected_idx) {
                                                        let to_insert = item.insert_text.as_deref().unwrap_or(&item.label);
                                                        tab.editor.insert(to_insert, tab.path.as_deref(), Some(&state.indent_calc));
                                                    }
                                                    state.completion_active = false;
                                                    state.completion_items.clear();
                                                    changed = true;
                                                } else {
                                                    tab.editor.insert("    ", tab.path.as_deref(), Some(&state.indent_calc));
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::ArrowLeft => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_left();
                                                } else {
                                                    tab.editor.move_left();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::ArrowRight => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_right();
                                                } else {
                                                    tab.editor.move_right();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::ArrowUp => {
                                                if state.completion_active && !state.completion_items.is_empty() {
                                                    state.completion_selected_idx = state.completion_selected_idx.saturating_sub(1);
                                                    changed = true;
                                                } else {
                                                    if state.shift_pressed {
                                                        tab.editor.extend_selection_up();
                                                    } else {
                                                        tab.editor.move_up();
                                                    }
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::ArrowDown => {
                                                if state.completion_active && !state.completion_items.is_empty() {
                                                    state.completion_selected_idx = (state.completion_selected_idx + 1).min(state.completion_items.len() - 1);
                                                    changed = true;
                                                } else {
                                                    if state.shift_pressed {
                                                        tab.editor.extend_selection_down();
                                                    } else {
                                                        tab.editor.move_down();
                                                    }
                                                    changed = true;
                                                }
                                            }
                                            winit::keyboard::KeyCode::PageUp => {
                                                for _ in 0..10 {
                                                    tab.editor.move_up();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::PageDown => {
                                                for _ in 0..10 {
                                                    tab.editor.move_down();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Home => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_to_line_start();
                                                } else {
                                                    tab.editor.move_to_line_start();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::End => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_to_line_end();
                                                } else {
                                                    tab.editor.move_to_line_end();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::KeyA if state.ctrl_pressed => {
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

                    // Layout positions
                    let editor_x = ui.cursor_x;
                    let editor_y = ui.cursor_y;
                    let editor_w = width;

                    // Dimensions of typography
                    let line_height_pixels = (state.font_size * 1.54).round().max(14.0);
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

                    let tab_mut = &mut state.tabs[active_tab_idx];
                    let cursor_p = tab_mut.editor.cursor();

                    // Auto-scroll viewport so cursor row is visible ONLY when navigating/typing
                    if auto_scroll_to_cursor {
                        let cursor_top = cursor_p.row as f32 * line_height_pixels;
                        let cursor_bottom = cursor_top + line_height_pixels;

                        if cursor_top < tab_mut.scroll_y {
                            tab_mut.scroll_y = cursor_top;
                        } else if cursor_bottom > tab_mut.scroll_y + editor_h - 10.0 {
                            let max_scroll = (content_h - editor_h).max(0.0);
                            tab_mut.scroll_y = (cursor_bottom - editor_h + 10.0).clamp(0.0, max_scroll);
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
                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                    instance: RectInstance {
                                        pos: [editor_x, editor_y],
                                        size: [gutter_w, editor_h],
                                        color: colors.bg_gutter.to_array(),
                                        radius: [0.0; 4],
                                        clip_rect: clip,
                                        ..Default::default()
                                    }
                                }));
                            }

                            if let Some(fs) = ui.font_system.as_ref() {
                                let adapter = state.font_adapter.get_or_insert_with(|| {
                                    let mut a = CosmicFontProvider::new_with_system(fs.clone());
                                    a.set_layout_size(10000.0, 10000.0);
                                    a
                                });

                                for i in visible_start..visible_end {
                                    let line_y = editor_y + (i as f32) * line_height_pixels - scroll_y;
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
                                                options: TextOptions::new().font_size(12.0).font_family("monospace").color(Color::rgb(0.35, 0.85, 0.45)),
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
                                                options: TextOptions::new().font_size(12.0).font_family("monospace").color(Color::rgb(0.97, 0.45, 0.45)),
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
                                            options: TextOptions::new().font_size((state.font_size - 1.0).max(10.0)).font_family("monospace").color(num_color),
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

                                    // Get code line text
                                    let line_text = tab.editor.buffer.line(i).unwrap_or_default();
                                    let clean_text = line_text.trim_end_matches(&['\r', '\n'][..]);

                                    // Shape code line (cached)
                                    let opts = TextOptions::new().font_size(state.font_size).font_family("monospace").color(colors.text_primary);
                                    let shaped = {
                                        let needs_update = match tab.shaping_cache.get(&i) {
                                            Some((cached_text, _)) => cached_text != clean_text,
                                            None => true,
                                        };
                                        if needs_update {
                                            let shaped = adapter.shape(clean_text, &opts);
                                            tab.shaping_cache.insert(i, (clean_text.to_string(), shaped));
                                        }
                                        tab.shaping_cache.get(&i).unwrap().1.clone()
                                    };

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
                                        let removed_opts = TextOptions::new().font_size(state.font_size).font_family("monospace").color(Color::rgb(0.97, 0.50, 0.50));
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
                                                    .font_size(state.font_size)
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
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [editor_x + gutter_w + cursor_x, line_y + 2.0],
                                                    size: [2.0, (state.font_size + 3.0).max(14.0)],
                                                    color: colors.accent.to_array(),
                                                    radius: [0.0; 4],
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                }
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
                        let line_y = editor_y + (cur.row as f32) * line_height_pixels - scroll_y;
                        if line_y >= editor_y && line_y <= editor_y + editor_h {
                            Some([editor_x + gutter_w + (cur.column as f32 * 7.8), line_y + line_height_pixels])
                        } else {
                            None
                        }
                    });

                    // Set focus and mouse selection logic on click
                    let is_hovered = ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
                    if ui.clicked && is_hovered {
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
                            ui.needs_redraw = true;

                            let rel_y = ui.mouse_y - (editor_y + ui.offset_y) + tab.scroll_y;
                            let line_idx = ((rel_y / line_height_pixels).max(0.0) as usize).min(total_lines.saturating_sub(1));

                            let gutter_w = if state.line_numbers_visible { 44.0 } else { 8.0 };
                            let rel_x = ui.mouse_x - (editor_x + ui.offset_x) - gutter_w;
                            let rel_x = rel_x.max(0.0);

                            let mut col_idx = 0;
                            if let Some(fs) = ui.font_system.as_ref() {
                                let line_text = tab.editor.buffer.line(line_idx).unwrap_or_default();
                                let clean_text = line_text.trim_end_matches(&['\r', '\n'][..]);
                                let opts = TextOptions::new().font_size(state.font_size).font_family("monospace");
                                let shaped = {
                                    let needs_update = match tab.shaping_cache.get(&line_idx) {
                                        Some((cached_text, _)) => cached_text != clean_text,
                                        None => true,
                                    };
                                    if needs_update {
                                        let adapter = state.font_adapter.get_or_insert_with(|| {
                                            let mut a = CosmicFontProvider::new_with_system(fs.clone());
                                            a.set_layout_size(10000.0, 10000.0);
                                            a
                                        });
                                        let shaped = adapter.shape(clean_text, &opts);
                                        tab.shaping_cache.insert(line_idx, (clean_text.to_string(), shaped));
                                    }
                                    tab.shaping_cache.get(&line_idx).unwrap().1.clone()
                                };
                                let byte_idx = shaped.index_at(rel_x, 5.0);
                                col_idx = clean_text[..byte_idx.min(clean_text.len())].chars().count();
                            }

                            if state.shift_pressed {
                                let saved = tab.editor.selection().start;
                                tab.editor.set_selection(Selection::new(saved, Point::new(line_idx, col_idx)));
                            } else {
                                tab.editor.set_cursor(Point::new(line_idx, col_idx));
                            }
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
                            .absolute(px, py)
                            .overlay()
                            .width(popup_w)
                            .height(popup_h)
                            .scroll_y(true)
                            .bg(popup_bg)
                            .border(colors.border, 1.0)
                            .radius_all(0.0)
                            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.4), 0.0, 4.0, 8.0)
                            .padding(4.0, 4.0, 4.0, 4.0)
                            .column()
                            .gap(1.0)
                            .show(|ui| {
                                if items.is_empty() {
                                    ui.container()
                                        .full_width()
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
                                            .full_width()
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
                                                .hover_bg(colors.highlight);
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

                // ─── Render LSP Autocomplete Floating Popup Overlay ──────────
                if state.completion_active && !state.completion_items.is_empty() {
                    if let Some([cur_x, cur_y]) = cursor_screen_pos {
                        let popup_w = 320.0f32;
                        let item_h = 22.0f32;
                        let max_visible = 8;
                        let visible_count = state.completion_items.len().min(max_visible);
                        let popup_h = (visible_count as f32 * item_h + 8.0).clamp(32.0, 220.0);

                        let mut px = cur_x;
                        if px + popup_w > ui.width - 12.0 {
                            px = (ui.width - popup_w - 12.0).max(12.0);
                        }
                        let mut py = cur_y + 2.0;
                        if py + popup_h > ui.height - 28.0 {
                            py = (cur_y - popup_h - 22.0).max(10.0);
                        }

                        let is_mouse_inside = ui.mouse_x >= px && ui.mouse_x <= px + popup_w
                            && ui.mouse_y >= py && ui.mouse_y <= py + popup_h;

                        let mut item_to_insert = None;
                        let mut dismiss_comp = false;

                        if ui.clicked && !is_mouse_inside {
                            dismiss_comp = true;
                        }

                        let comp_popup_id = Id::from_u64(999999777);
                        let comp_bg = colors.bg_sidebar;
                        let selected_idx = state.completion_selected_idx;

                        ui.overlay(|ui| {
                            ui.container()
                                .raw_id(comp_popup_id)
                                .absolute(px, py)
                                .overlay()
                                .width(popup_w)
                                .height(popup_h)
                                .scroll_y(true)
                                .bg(comp_bg)
                                .border(colors.border, 1.0)
                                .radius_all(4.0)
                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.45), 0.0, 4.0, 10.0)
                                .padding(4.0, 4.0, 4.0, 4.0)
                                .column()
                                .gap(1.0)
                                .show(|ui| {
                                    for (idx, item) in state.completion_items.iter().enumerate() {
                                        let is_selected = idx == selected_idx;
                                        let item_id = Id::from_u64(999999800 + idx as u64);
                                        let (kind_icon, kind_color) = completion_kind_icon(item.kind);

                                        let mut row = ui.container()
                                            .id(item_id)
                                            .full_width()
                                            .height(item_h)
                                            .row()
                                            .valign(Align::Center)
                                            .padding_left(6.0)
                                            .padding_right(6.0)
                                            .gap(6.0)
                                            .radius_all(3.0);

                                        if is_selected {
                                            row = row.bg(Color::rgba(18.0 / 255.0, 68.0 / 255.0, 105.0 / 255.0, 1.0));
                                        } else {
                                            row = row.bg(Color::TRANSPARENT).hover_bg(colors.highlight);
                                        }

                                        let resp = row.show(|ui| {
                                            ui.text(kind_icon)
                                                .size(10.0)
                                                .color(kind_color)
                                                .show();

                                            ui.text(&item.label)
                                                .size(11.5)
                                                .weight(FontWeight::Medium)
                                                .color(colors.text_primary)
                                                .show();

                                            if let Some(ref detail) = item.detail {
                                                let clean_detail = detail.lines().next().unwrap_or("").trim();
                                                if !clean_detail.is_empty() {
                                                    ui.text(clean_detail)
                                                        .size(10.5)
                                                        .color(colors.text_dim)
                                                        .show();
                                                }
                                            }
                                        });

                                        if resp.clicked {
                                            let text = item.insert_text.clone().unwrap_or_else(|| item.label.clone());
                                            item_to_insert = Some(text);
                                        }
                                    }
                                });
                        });

                        if let Some(insert_text) = item_to_insert {
                            let tab_path = state.tabs[active_tab_idx].path.clone();
                            state.tabs[active_tab_idx].editor.insert(&insert_text, tab_path.as_deref(), Some(&state.indent_calc));
                            state.on_buffer_changed(active_tab_idx);
                            state.completion_active = false;
                            state.completion_items.clear();
                            ui.request_redraw();
                        } else if dismiss_comp {
                            state.completion_active = false;
                            state.completion_items.clear();
                            ui.request_redraw();
                        }
                    }
                }
            }
        });
}

fn completion_kind_icon(kind: Option<crate::lsp::protocol::CompletionItemKind>) -> (&'static str, Color) {
    use crate::lsp::protocol::CompletionItemKind;
    match kind {
        Some(CompletionItemKind::Function) | Some(CompletionItemKind::Method) => ("\u{f013}", zenthra::Color::rgb(0.75, 0.45, 0.95)),
        Some(CompletionItemKind::Variable) | Some(CompletionItemKind::Field) | Some(CompletionItemKind::Property) => ("\u{f02b}", zenthra::Color::rgb(0.35, 0.70, 0.95)),
        Some(CompletionItemKind::Class) | Some(CompletionItemKind::Struct) | Some(CompletionItemKind::Interface) => ("\u{f1b2}", zenthra::Color::rgb(0.95, 0.75, 0.30)),
        Some(CompletionItemKind::Keyword) => ("\u{f084}", zenthra::Color::rgb(0.90, 0.45, 0.55)),
        Some(CompletionItemKind::Snippet) => ("\u{f0c5}", zenthra::Color::rgb(0.40, 0.85, 0.55)),
        Some(CompletionItemKind::Module) => ("\u{f07b}", zenthra::Color::rgb(0.95, 0.65, 0.35)),
        Some(CompletionItemKind::Constant) | Some(CompletionItemKind::Value) | Some(CompletionItemKind::EnumMember) => ("\u{f005}", zenthra::Color::rgb(0.90, 0.55, 0.30)),
        _ => ("\u{f15b}", zenthra::Color::rgb(0.65, 0.70, 0.75)),
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

struct StyledSpan {
    pub text: String,
    pub x_offset: f32,
    pub color: Color,
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

    // Convert char offset to byte offset without collecting all indices into a Vec.
    // We iterate char_indices lazily, which avoids a heap allocation per line per frame.
    let char_to_byte = |char_offset: usize| -> usize {
        if char_offset == 0 { return 0; }
        line_text.char_indices()
            .nth(char_offset)
            .map(|(b, _)| b)
            .unwrap_or(line_text.len())
    };

    let char_count = line_text.chars().count();
    let mut last_char = 0;

    for &(start_char, end_char, color) in spans {
        let start_char = start_char.min(char_count);
        let end_char = end_char.min(char_count);

        if start_char > last_char {
            let gap_start_byte = char_to_byte(last_char);
            let gap_end_byte = char_to_byte(start_char);
            let text = line_text[gap_start_byte..gap_end_byte].to_string();
            let x_offset = find_x_for_byte(shaped, line_text, gap_start_byte);
            result.push(StyledSpan { text, x_offset, color: default_color });
        }

        if end_char > start_char {
            let start_byte = char_to_byte(start_char);
            let end_byte = char_to_byte(end_char);
            let text = line_text[start_byte..end_byte].to_string();
            let x_offset = find_x_for_byte(shaped, line_text, start_byte);
            result.push(StyledSpan { text, x_offset, color });
        }

        last_char = end_char;
    }

    if last_char < char_count {
        let tail_start_byte = char_to_byte(last_char);
        let text = line_text[tail_start_byte..].to_string();
        let x_offset = find_x_for_byte(shaped, line_text, tail_start_byte);
        result.push(StyledSpan { text, x_offset, color: default_color });
    }

    if result.is_empty() && !line_text.is_empty() {
        result.push(StyledSpan { text: line_text.to_string(), x_offset: 0.0, color: default_color });
    }

    result
}
