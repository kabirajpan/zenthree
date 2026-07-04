use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, ObjectFit};
use zenthra_widgets::ui::{DrawCommand, TextDraw, RectDraw, ScrollDrag};
use zenthra_render::RectInstance;
use zenthra_text::prelude::{CosmicFontProvider, TextOptions, ShapedBuffer};
use editor::buffer::Point;
use editor::selection::Selection;

pub fn draw_main_buffer(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();

    ui.container()
        .fill()
        .bg(colors.bg_editor)
        .column()
        .show(|ui| {
            if state.tabs.is_empty() {
                // Welcome / Empty State
                ui.container()
                    .fill()
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
                let active_tab_idx = state.active_tab.unwrap_or(0);

                // ─── Tab Bar ────────────────────────────────────────────────
                ui.container()
                    .full_width()
                    .height(32.0)
                    .bg(colors.bg_inactive_tab)
                    .row()
                    .valign(Align::Center)
                    .show(|ui| {
                        let mut tab_to_close = None;
                        let mut tab_to_activate = None;

                        for idx in 0..state.tabs.len() {
                            let tab_is_dirty = state.tabs[idx].is_dirty;
                            let tab_name = state.tabs[idx].name.clone();
                            let is_active = idx == active_tab_idx;

                            let tab_id = Id::from_u64(999994000 + idx as u64);
                            let tab_close_id = Id::from_u64(999995000 + idx as u64);
                            let bg = if is_active { colors.bg_active_tab } else { colors.bg_inactive_tab };
                            let text_color = if is_active { colors.text_primary } else { colors.text_muted };

                            let icon_theme = crate::theme::IconTheme::new("gruvbox");
                            let icon_source = if let Some(ref path) = state.tabs[idx].path {
                                let ext = path.extension()
                                    .and_then(|e| e.to_str())
                                    .unwrap_or("")
                                    .to_lowercase();
                                icon_theme.get_icon_source("text", &ext)
                            } else {
                                icon_theme.get_icon_source("text", "")
                            };

                            let mut close_clicked = false;

                            let resp = ui.container()
                                .id(tab_id)
                                .height(32.0)
                                .bg(bg)
                                .border(colors.border, 1.0)
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .padding_left(12.0)
                                .padding_right(8.0)
                                .hover_bg(colors.highlight)
                                .show(|ui| {
                                    ui.image(icon_source)
                                        .size(14.0, 14.0)
                                        .fit(ObjectFit::Contain)
                                        .show();

                                    ui.text(&tab_name)
                                        .size(11.5)
                                        .color(text_color)
                                        .show();

                                    if tab_is_dirty {
                                        ui.container()
                                            .width(6.0)
                                            .height(6.0)
                                            .radius_all(3.0)
                                            .bg(colors.accent)
                                            .show(|_| {});
                                    }

                                    // Close button
                                    let close_resp = ui.button("\u{f00d}")
                                        .id(tab_close_id)
                                        .width(18.0)
                                        .height(18.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgb(30.0 / 255.0, 144.0 / 255.0, 255.0 / 255.0))
                                        .text_color(colors.text_primary)
                                        .radius_all(4.0)
                                        .padding(0.0, 0.0, 0.0, 0.0)
                                        .size(10.5)
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
                            ui.request_redraw();
                        } else if let Some(idx) = tab_to_activate {
                            state.active_tab = Some(idx);
                            ui.request_redraw();
                        }
                    });

                // ─── Editor Area ────────────────────────────────────────────
                if active_tab_idx < state.tabs.len() {
                    let editor_h = ui.height - 36.0 - 24.0 - 32.0;
                    let editor_id = Id::from_u64(999996000 + active_tab_idx as u64);

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
                    let is_focused = ui.focused_id == Some(editor_id);

                    // Handle scrollbar drag and scroll wheel before input events are taken/drained
                    let editor_x = ui.cursor_x;
                    let editor_y = ui.cursor_y;
                    let editor_w = ui.available_width;

                    {
                        let tab_mut = &mut state.tabs[active_tab_idx];
                        let line_height_pixels = 20.0;
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
                                if drag.id == scrollbar_id {
                                    is_dragging = true;
                                    if ui.mouse_down {
                                        let ratio = if track_h > thumb_h { max_scroll_y / (track_h - thumb_h) } else { 0.0 };
                                        let delta_y = ui.mouse_y - drag.start_mouse;
                                        tab_mut.scroll_y = (drag.start_scroll + delta_y * ratio).clamp(0.0, max_scroll_y);
                                        ui.needs_redraw = true;
                                    }
                                }
                            }

                            if !is_dragging && ui.clicked {
                                let mouse_in_track = ui.mouse_x >= scrollbar_x - 6.0 && ui.mouse_x <= scrollbar_x + scrollbar_w + 6.0
                                    && ui.mouse_y >= editor_y && ui.mouse_y <= editor_y + track_h;
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

                        let is_hovered = ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
                        if is_hovered {
                            let mut scroll_delta_y = 0.0;
                            for event in &ui.input_events {
                                if let zenthra_platform::event::PlatformEvent::MouseWheel { delta_y, .. } = event {
                                    scroll_delta_y += *delta_y;
                                }
                            }
                            if scroll_delta_y != 0.0 {
                                tab_mut.scroll_y = (tab_mut.scroll_y - scroll_delta_y * 15.0).clamp(0.0, max_scroll_y);
                                ui.needs_redraw = true;
                            }
                        }
                    }

                    {
                        let tab = &mut state.tabs[active_tab_idx];

                        // Process input events if editor is focused
                        if is_focused {
                            let events = std::mem::take(&mut ui.input_events);
                            let mut changed = false;

                            // We only care about non-MouseWheel events for typing/navigation
                            let (_, other_events): (Vec<_>, Vec<_>) = events.into_iter().partition(|e| {
                                matches!(e, zenthra_platform::event::PlatformEvent::MouseWheel { .. })
                            });

                            for event in &other_events {
                                match event {
                                    zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                                        if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                                            tab.editor.insert(&c.to_string(), tab.path.as_deref(), Some(&state.indent_calc));
                                            changed = true;
                                        }
                                    }
                                    zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                                        match key {
                                            winit::keyboard::KeyCode::Backspace => {
                                                tab.editor.backspace();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Delete => {
                                                tab.editor.delete();
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Enter => {
                                                tab.editor.insert("\n", tab.path.as_deref(), Some(&state.indent_calc));
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::Tab => {
                                                tab.editor.insert("    ", tab.path.as_deref(), Some(&state.indent_calc));
                                                changed = true;
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
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_up();
                                                } else {
                                                    tab.editor.move_up();
                                                }
                                                changed = true;
                                            }
                                            winit::keyboard::KeyCode::ArrowDown => {
                                                if state.shift_pressed {
                                                    tab.editor.extend_selection_down();
                                                } else {
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
                                            winit::keyboard::KeyCode::KeyS if state.ctrl_pressed => {
                                                save_tab = true;
                                            }
                                            winit::keyboard::KeyCode::KeyF if state.ctrl_pressed && state.shift_pressed => {
                                                if let Err(e) = state.formatter.format_editor(&mut tab.editor, tab.path.as_deref()) {
                                                    state.status_message = format!("Format error: {}", e);
                                                } else {
                                                    state.status_message = "Formatted successfully!".to_string();
                                                    tab.is_dirty = true;
                                                    tab.version += 1;
                                                }
                                                changed = true;
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {}
                                }
                            }
                            if changed {
                                tab.is_dirty = true;
                                tab.version += 1;
                                ui.needs_redraw = true;
                            }
                        }
                    }

                    if save_tab {
                        state.save_active_tab();
                        ui.request_redraw();
                    }

                    // Layout positions
                    let editor_x = ui.cursor_x;
                    let editor_y = ui.cursor_y;
                    let editor_w = ui.available_width;

                    // Dimensions of typography
                    let line_height_pixels = 20.0;
                    let total_lines = state.tabs[active_tab_idx].editor.buffer.line_count();
                    let content_h = total_lines as f32 * line_height_pixels;

                    let tab_mut = &mut state.tabs[active_tab_idx];
                    let scroll_y = tab_mut.scroll_y;

                    // Pre-cache the visible line range before entering the render loop closure
                    let start_y_visible = scroll_y;
                    let end_y_visible = scroll_y + editor_h;
                    let visible_start = (start_y_visible.max(0.0) / line_height_pixels) as usize;
                    let visible_end = ((end_y_visible.max(0.0) / line_height_pixels) as usize + 1).min(total_lines);
                    tab_mut.editor.buffer.ensure_range_cached(visible_start..(visible_end + 2).min(total_lines + 1));

                    // Sync highlighter theme with current state theme
                    if state.theme == crate::theme::ThemeMode::Light {
                        state.highlighter.set_theme(syntax::SyntaxTheme::light());
                    } else {
                        state.highlighter.set_theme(syntax::SyntaxTheme::dark());
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
                        visible_end,
                    );

                    // Container wrapper - not scrollable, but clips
                    ui.container()
                        .id(editor_id)
                        .fill_x()
                        .height(editor_h)
                        .clip(true)
                        .show(|ui| {
                            let tab = &mut state.tabs[active_tab_idx];
                            if tab.shaping_cache.len() > 1000 {
                                tab.shaping_cache.clear();
                            }

                            let draw_start = ui.draws.len();

                            let clip = [editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h];

                            // 1. Draw gutter background
                            let gutter_w = 44.0;
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

                            // 2. Draw border separator
                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                instance: RectInstance {
                                    pos: [editor_x + gutter_w - 1.0, editor_y],
                                    size: [1.0, editor_h],
                                    color: colors.border.to_array(),
                                    radius: [0.0; 4],
                                    clip_rect: clip,
                                    ..Default::default()
                                }
                            }));

                            // 3. Draw lines of text, selection, and cursor
                            if let Some(fs) = ui.font_system.as_ref() {
                                let mut adapter = CosmicFontProvider::new_with_system(fs.clone());
                                adapter.set_layout_size(10000.0, 10000.0);

                                for i in visible_start..visible_end {
                                    let line_y = editor_y + (i as f32) * line_height_pixels - scroll_y;

                                    // Line highlighting for cursor line
                                    let cursor_p = tab.editor.cursor();
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

                                    // Draw line number
                                    let num_str = format!("{}", i + 1);
                                    let num_x = editor_x + gutter_w - 16.0 - (num_str.len() as f32 * 6.5);
                                    let num_color = if cursor_p.row == i && is_focused { colors.text_primary } else { colors.text_dim };

                                    ui.draws.push(DrawCommand::Text(TextDraw {
                                        text: num_str,
                                        pos: [num_x, line_y + 3.0],
                                        options: TextOptions::new().font_size(12.0).font_family("monospace").color(num_color),
                                        clip,
                                    }));

                                    // Get code line text
                                    let line_text = tab.editor.buffer.line(i).unwrap_or_default();
                                    let clean_text = line_text.trim_end_matches(&['\r', '\n'][..]);

                                    // Shape code line (cached)
                                    let opts = TextOptions::new().font_size(13.0).font_family("monospace").color(colors.text_primary);
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

                                                let start_x = find_x_for_byte(&shaped, start_byte);
                                                let end_x = find_x_for_byte(&shaped, end_byte);

                                                let sel_w = (end_x - start_x).max(8.0);
                                                let sel_color = Color::rgba(48.0 / 255.0, 120.0 / 255.0, 200.0 / 255.0, 0.35);

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
                                    let line_spans = highlights.get(&i);
                                    if let Some(spans) = line_spans {
                                        let styled_spans = get_styled_spans(clean_text, spans, colors.text_primary, &shaped);
                                        for span in styled_spans {
                                            let span_opts = TextOptions::new()
                                                .font_size(13.0)
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

                                    // Draw blinking cursor
                                    if cursor_p.row == i && is_focused {
                                        let cursor_byte = clean_text.chars().take(cursor_p.column).map(|c| c.len_utf8()).sum();
                                        let cursor_x = find_x_for_byte(&shaped, cursor_byte);

                                        let show_cursor = (ui.elapsed_time * 2.0) as usize % 2 == 0;
                                        if show_cursor {
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [editor_x + gutter_w + cursor_x, line_y + 2.0],
                                                    size: [2.0, 16.0],
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

                            // Advance the layout in immediate-mode container context
                            ui.advance(editor_w, editor_h, draw_start);
                        });

                    // Set focus and mouse selection logic on click
                    let is_hovered = ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
                    if ui.clicked && is_hovered {
                        // Avoid positioning the cursor if clicking on the scrollbar
                        let scrollbar_w = 6.0;
                        let scrollbar_x = editor_x + editor_w - scrollbar_w - 4.0;
                        let max_scroll_y = (content_h - editor_h).max(0.0);
                        let mouse_in_scrollbar = max_scroll_y > 0.0
                            && ui.mouse_x >= scrollbar_x - 6.0
                            && ui.mouse_x <= scrollbar_x + scrollbar_w + 6.0;

                        if !mouse_in_scrollbar {
                            let tab = &mut state.tabs[active_tab_idx];
                            ui.focused_id = Some(editor_id);
                            ui.needs_redraw = true;

                            // Calculate click position
                            let rel_y = ui.mouse_y - (editor_y + ui.offset_y) + tab.scroll_y;
                            let line_idx = ((rel_y / line_height_pixels).max(0.0) as usize).min(total_lines.saturating_sub(1));

                            let gutter_w = 44.0;
                            let rel_x = ui.mouse_x - (editor_x + ui.offset_x) - gutter_w;
                            let rel_x = rel_x.max(0.0);

                            let mut col_idx = 0;
                            if let Some(fs) = ui.font_system.as_ref() {
                                let line_text = tab.editor.buffer.line(line_idx).unwrap_or_default();
                                let clean_text = line_text.trim_end_matches(&['\r', '\n'][..]);
                                let opts = TextOptions::new().font_size(13.0).font_family("monospace");
                                let shaped = {
                                    let needs_update = match tab.shaping_cache.get(&line_idx) {
                                        Some((cached_text, _)) => cached_text != clean_text,
                                        None => true,
                                    };
                                    if needs_update {
                                        let mut adapter = CosmicFontProvider::new_with_system(fs.clone());
                                        adapter.set_layout_size(10000.0, 10000.0);
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
                        }
                    }
                }
            }
        });
}

fn find_x_for_byte(shaped: &ShapedBuffer, byte_idx: usize) -> f32 {
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
            best_x = last.x + last.width;
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
    let char_indices: Vec<(usize, char)> = line_text.char_indices().collect();
    let char_count = char_indices.len();
    
    let mut last_idx = 0;
    
    for &(start_char, end_char, color) in spans {
        let start_char = start_char.min(char_count);
        let end_char = end_char.min(char_count);
        
        if start_char > last_idx {
            let gap_start_byte = char_indices[last_idx].0;
            let gap_end_byte = if start_char < char_count {
                char_indices[start_char].0
            } else {
                line_text.len()
            };
            let text = line_text[gap_start_byte..gap_end_byte].to_string();
            let x_offset = find_x_for_byte(shaped, gap_start_byte);
            result.push(StyledSpan { text, x_offset, color: default_color });
        }
        
        if end_char > start_char {
            let start_byte = char_indices[start_char].0;
            let end_byte = if end_char < char_count {
                char_indices[end_char].0
            } else {
                line_text.len()
            };
            let text = line_text[start_byte..end_byte].to_string();
            let x_offset = find_x_for_byte(shaped, start_byte);
            result.push(StyledSpan { text, x_offset, color });
        }
        
        last_idx = end_char;
    }
    
    if last_idx < char_count {
        let tail_start_byte = char_indices[last_idx].0;
        let text = line_text[tail_start_byte..].to_string();
        let x_offset = find_x_for_byte(shaped, tail_start_byte);
        result.push(StyledSpan { text, x_offset, color: default_color });
    }
    
    if result.is_empty() && !line_text.is_empty() {
        result.push(StyledSpan { text: line_text.to_string(), x_offset: 0.0, color: default_color });
    }
    
    result
}
