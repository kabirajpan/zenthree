use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context};
use crate::theme::ThemeColors;
use zenthra_text::prelude::TextOptions;
use zenthra_widgets::ui::{DrawCommand, TextDraw, RectDraw};
use zenthra_render::RectInstance;

fn convert_vt_color(c: vt100::Color, default_color: Color) -> Color {
    match c {
        vt100::Color::Default => default_color,
        vt100::Color::Rgb(r, g, b) => Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0),
        vt100::Color::Idx(idx) => match idx {
            0 => Color::rgb(0.0, 0.0, 0.0),
            1 => Color::rgb(0.85, 0.25, 0.25),
            2 => Color::rgb(0.25, 0.85, 0.25),
            3 => Color::rgb(0.85, 0.85, 0.25),
            4 => Color::rgb(0.35, 0.55, 0.95),
            5 => Color::rgb(0.85, 0.35, 0.85),
            6 => Color::rgb(0.25, 0.85, 0.85),
            7 => Color::rgb(0.85, 0.85, 0.85),
            8 => Color::rgb(0.45, 0.45, 0.45),
            9 => Color::rgb(1.0, 0.35, 0.35),
            10 => Color::rgb(0.35, 1.0, 0.35),
            11 => Color::rgb(1.0, 1.0, 0.35),
            12 => Color::rgb(0.45, 0.65, 1.0),
            13 => Color::rgb(1.0, 0.45, 1.0),
            14 => Color::rgb(0.35, 1.0, 1.0),
            15 => Color::rgb(1.0, 1.0, 1.0),
            16..=231 => {
                let i = idx - 16;
                let b = (i % 6) as f32 / 5.0;
                let g = ((i / 6) % 6) as f32 / 5.0;
                let r = (i / 36) as f32 / 5.0;
                Color::rgb(r, g, b)
            }
            232..=255 => {
                let gray = (idx - 232) as f32 / 23.0 * 0.9 + 0.05;
                Color::rgb(gray, gray, gray)
            }
        },
    }
}

pub fn draw_terminal_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    state.ensure_terminal_session();

    let term_id = Id::from_u64(999999800);
    let is_focused = state.focused_panel == crate::state::types::FocusedPanel::Terminal;

    if is_focused {
        if ui.focused_id != Some(term_id) {
            ui.focused_id = Some(term_id);
        }
        // Cursor blink schedule
        let elapsed_ms = state.last_cursor_activity.elapsed().as_millis();
        let ms_until_next_blink = if elapsed_ms < 500 {
            500 - elapsed_ms
        } else {
            530 - ((elapsed_ms - 500) % 530)
        };
        ui.request_redraw_after(std::time::Duration::from_millis(
            ms_until_next_blink as u64,
        ));
    }

    // Exact screen coordinates of the entire terminal panel
    let panel_x = ui.cursor_x;
    let panel_y = ui.cursor_y;
    let panel_w = width;
    let panel_h = height;

    // ─── Top Horizontal Edge Handler (Resize Splitter) ───────────
    let splitter_h = 6.0;
    let splitter_x = panel_x;
    let splitter_y = panel_y;
    let splitter_w = width;
    let is_blocked = ui.modal_active || state.is_modal_or_dialog_open();
    let is_splitter_hovered = !is_blocked && ui.mouse_in_rect(
        splitter_x,
        splitter_y - splitter_h / 2.0,
        splitter_w,
        splitter_h,
    );

    // Click anywhere in the terminal panel to focus terminal
    let mouse_in_terminal = !is_blocked
        && !is_splitter_hovered
        && !state.active_resize_terminal
        && ui.mouse_in_rect(panel_x, panel_y, panel_w, panel_h);

    if mouse_in_terminal && (ui.clicked || (ui.mouse_down && ui.active_drag.is_none())) {
        ui.focused_id = Some(term_id);
        state.focused_panel = crate::state::types::FocusedPanel::Terminal;
        state.last_cursor_activity = std::time::Instant::now();
        ui.needs_redraw = true;
    }

    if is_blocked {
        state.active_resize_terminal = false;
    }

    if is_splitter_hovered || state.active_resize_terminal {
        ui.cursor_icon = CursorIcon::RowResize;
        ui.resize_active = true;
    }

    if is_splitter_hovered && ui.mouse_down && ui.active_drag.is_none() {
        state.active_resize_terminal = true;
    }

    if state.active_resize_terminal {
        if !ui.mouse_down {
            state.active_resize_terminal = false;
            state.save_persistent_state();
        } else {
            let status_bar_h = if state.status_bar_visible { 24.0 } else { 0.0 };
            let bottom_y = ui.height - status_bar_h;
            let target_h = bottom_y - ui.mouse_y;
            state.terminal_height = target_h.clamp(70.0, (ui.height * 0.7).max(100.0));
            ui.request_redraw();
        }
    }

    // Visual feedback for horizontal edge handler
    let edge_color = if state.active_resize_terminal {
        colors.accent
    } else if is_splitter_hovered {
        colors.accent.with_alpha(0.8)
    } else {
        colors.border
    };

    let edge_thickness = if state.active_resize_terminal || is_splitter_hovered { 2.0 } else { 1.0 };
    let edge_draw_y = splitter_y - 1.0;

    ui.draws.push(DrawCommand::Rect(RectDraw {
        instance: RectInstance {
            pos: [splitter_x, edge_draw_y],
            size: [splitter_w, edge_thickness],
            color: edge_color.to_array(),
            radius: [0.0; 4],
            border_width: 0.0,
            border_color: [0.0; 4],
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));

    // Center pill grip handle when hovered or active
    if is_splitter_hovered || state.active_resize_terminal {
        let grip_w = 36.0;
        let grip_h = 4.0;
        let grip_x = splitter_x + (splitter_w - grip_w) / 2.0;
        let grip_y = splitter_y - grip_h / 2.0;
        ui.draws.push(DrawCommand::Rect(RectDraw {
            instance: RectInstance {
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

    // ─── PTY Output Streaming Poll ───────────────────────────────
    if let Some(ref sess) = state.terminal_session {
        if sess.has_new_output.swap(false, std::sync::atomic::Ordering::Relaxed) {
            ui.needs_redraw = true;
        }
        ui.request_redraw_after(std::time::Duration::from_millis(30));
    }

    // ─── Input Event Processing directly inside Terminal ─────────
    if is_focused {
        let mut events = std::mem::take(&mut ui.input_events);
        events.retain(|event| {
            match event {
                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                    if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                        state.last_cursor_activity = std::time::Instant::now();
                        let mut buf = [0u8; 4];
                        let bytes = c.encode_utf8(&mut buf).as_bytes();
                        if let Some(ref mut sess) = state.terminal_session {
                            sess.write_raw(bytes);
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                }
                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                    state.last_cursor_activity = std::time::Instant::now();
                    let mut handled = true;
                    let bytes: Option<&[u8]> = if state.ctrl_pressed {
                        match key {
                            winit::keyboard::KeyCode::KeyC => Some(b"\x03"),
                            winit::keyboard::KeyCode::KeyD => Some(b"\x04"),
                            winit::keyboard::KeyCode::KeyZ => Some(b"\x1a"),
                            winit::keyboard::KeyCode::KeyL => Some(b"\x0c"),
                            winit::keyboard::KeyCode::KeyA => Some(b"\x01"),
                            winit::keyboard::KeyCode::KeyE => Some(b"\x05"),
                            winit::keyboard::KeyCode::KeyU => Some(b"\x15"),
                            winit::keyboard::KeyCode::KeyK => Some(b"\x0b"),
                            winit::keyboard::KeyCode::KeyW => Some(b"\x17"),
                            winit::keyboard::KeyCode::KeyR => Some(b"\x12"),
                            winit::keyboard::KeyCode::KeyV => {
                                if let Some(text) = state.get_clipboard() {
                                    if let Some(ref mut sess) = state.terminal_session {
                                        sess.write_raw(text.as_bytes());
                                    }
                                }
                                None
                            }
                            _ => { handled = false; None }
                        }
                    } else {
                        match key {
                            winit::keyboard::KeyCode::Enter => Some(b"\r"),
                            winit::keyboard::KeyCode::Backspace => Some(b"\x7f"),
                            winit::keyboard::KeyCode::Tab => Some(b"\t"),
                            winit::keyboard::KeyCode::Escape => Some(b"\x1b"),
                            winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[A"),
                            winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[B"),
                            winit::keyboard::KeyCode::ArrowRight => Some(b"\x1b[C"),
                            winit::keyboard::KeyCode::ArrowLeft => Some(b"\x1b[D"),
                            winit::keyboard::KeyCode::Home => Some(b"\x1b[H"),
                            winit::keyboard::KeyCode::End => Some(b"\x1b[F"),
                            winit::keyboard::KeyCode::Delete => Some(b"\x1b[3~"),
                            winit::keyboard::KeyCode::PageUp => Some(b"\x1b[5~"),
                            winit::keyboard::KeyCode::PageDown => Some(b"\x1b[6~"),
                            _ => { handled = false; None }
                        }
                    };

                    if let Some(raw) = bytes {
                        if let Some(ref mut sess) = state.terminal_session {
                            sess.write_raw(raw);
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    if handled {
                        ui.needs_redraw = true;
                        return false;
                    }
                }
                _ => {}
            }
            true
        });
        ui.input_events = events;
    }

    // ─── Main Terminal Panel Box ─────────────────────────────────
    let term_panel = ui.container()
        .id(term_id)
        .width(width)
        .height(height)
        .bg(colors.bg_base)
        .column()
        .show(|ui| {
            // ─── Header Bar using SpaceBetween ───────────────────────
            const HEADER_H: f32 = 28.0;
            ui.container()
                .width(width)
                .height(HEADER_H)
                .bg(colors.bg_sidebar)
                .padding_left(8.0)
                .padding_right(8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .no_wrap()
                .show(|ui| {
                    // Child 1: Left Title (Terminal Icon + Label) & New Terminal Button
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            // Terminal Icon (\u{f489}) + Text
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    ui.text("\u{f489}")
                                        .size(12.0)
                                        .color(colors.accent)
                                        .show();

                                    if width >= 320.0 {
                                        ui.text("TERMINAL")
                                            .size(11.0)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();
                                    } else if width >= 240.0 {
                                        ui.text("TERM")
                                            .size(10.5)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();
                                    }
                                });

                            // New Terminal Button (\u{f0415})
                            let new_term_btn = ui.container()
                                .width(20.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f0415}")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if new_term_btn.clicked {
                                state.terminal_session = None;
                                state.ensure_terminal_session();
                                state.last_cursor_activity = std::time::Instant::now();
                                ui.focused_id = Some(term_id);
                                state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                                ui.needs_redraw = true;
                            }
                        });

                    // Child 2: Right Actions: Split, Copy, Paste, Clear, Close
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            // Split (\u{eb56})
                            let split_btn = ui.container()
                                .width(22.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{eb56}")
                                        .size(11.5)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if split_btn.clicked {
                                state.set_status("Terminal split (coming soon)");
                                ui.needs_redraw = true;
                            }

                            // Copy (\u{f018f})
                            let copy_btn = ui.container()
                                .width(22.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f018f}")
                                        .size(11.5)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if copy_btn.clicked {
                                let full_text = if let Some(ref sess) = state.terminal_session {
                                    if let Ok(p) = sess.parser.lock() {
                                        p.screen().contents()
                                    } else {
                                        String::new()
                                    }
                                } else {
                                    String::new()
                                };
                                state.set_clipboard(full_text);
                                state.set_status("Terminal output copied to clipboard");
                                ui.needs_redraw = true;
                            }

                            // Paste (\u{f0192})
                            let paste_btn = ui.container()
                                .width(22.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f0192}")
                                        .size(11.5)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if paste_btn.clicked {
                                if let Some(text) = state.get_clipboard() {
                                    if let Some(ref mut sess) = state.terminal_session {
                                        sess.write_raw(text.as_bytes());
                                    }
                                    state.last_cursor_activity = std::time::Instant::now();
                                    ui.focused_id = Some(term_id);
                                    state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                                    ui.needs_redraw = true;
                                }
                            }

                            // Clear text button
                            let clear_btn = ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .hover_bg(colors.highlight)
                                .radius_all(3.0)
                                .show(|ui| {
                                    ui.text("Clear")
                                        .size(10.5)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if clear_btn.clicked {
                                state.clear_terminal();
                                state.last_cursor_activity = std::time::Instant::now();
                                ui.focused_id = Some(term_id);
                                state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                                ui.needs_redraw = true;
                            }

                            // Close Button (\u{f0156})
                            let close_btn = ui.container()
                                .width(22.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f0156}")
                                        .size(11.5)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if close_btn.clicked {
                                state.terminal_visible = false;
                                if state.focused_panel == crate::state::types::FocusedPanel::Terminal {
                                    state.focused_panel = crate::state::types::FocusedPanel::Editor;
                                    ui.focused_id = state.active_tab.map(|idx| Id::from_u64(999996000 + idx as u64));
                                }
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }
                        });
                });

            // ─── Free Native Terminal Console Screen ─────────────────
            let body_h = (height - 28.0 - 1.0).max(40.0);
            let term_body_x = ui.cursor_x;
            let term_body_y = ui.cursor_y;

            let is_term_hovered = !is_blocked && ui.mouse_in_rect(term_body_x, term_body_y, width, body_h);
            if is_term_hovered {
                ui.cursor_icon = CursorIcon::Text;
                if ui.clicked || (ui.mouse_down && ui.active_drag.is_none()) {
                    ui.focused_id = Some(term_id);
                    state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                    state.last_cursor_activity = std::time::Instant::now();
                    ui.needs_redraw = true;
                }
            }

            let font_sz = 11.0 * ui.font_scale;
            let (cell_w, cell_h) = if let Some(fs) = ui.font_system.as_ref() {
                let mut adapter = zenthra_text::prelude::CosmicFontProvider::new_with_system(fs.clone());
                let opts = TextOptions::new().font_size(font_sz).font_family("monospace");
                let shaped = adapter.shape("M", &opts);
                let w = shaped.glyphs().first().map(|g| g.width).unwrap_or(0.0);
                let final_w = if w > 3.0 && w < 25.0 { w } else { 7.81 * ui.font_scale };
                (final_w, 17.0 * ui.font_scale)
            } else {
                (7.81 * ui.font_scale, 17.0 * ui.font_scale)
            };
            let pad_left = 8.0;
            let pad_top = 4.0;

            let target_cols = ((width - pad_left - 8.0) / cell_w).floor().max(20.0) as u16;
            let target_rows = ((body_h - pad_top - 4.0) / cell_h).floor().max(4.0) as u16;

            if let Some(ref mut sess) = state.terminal_session {
                let (cur_rows, cur_cols) = {
                    let p = sess.parser.lock().unwrap();
                    p.screen().size()
                };
                if cur_rows != target_rows || cur_cols != target_cols {
                    sess.resize(target_rows, target_cols);
                }
            }

            let clip = [term_body_x, term_body_y, width, body_h];

            let body_resp = ui.container()
                .id(Id::from_u64(999999802))
                .width(width)
                .height(body_h)
                .bg(colors.bg_base)
                .show(|ui| {
                    if let Some(ref sess) = state.terminal_session {
                        if let Ok(p) = sess.parser.lock() {
                            let screen = p.screen();
                            let (rows, cols) = screen.size();
                            let (cursor_row, cursor_col) = screen.cursor_position();
                            let hide_cursor = screen.hide_cursor();

                            for r in 0..rows {
                                let row_y = term_body_y + pad_top + r as f32 * cell_h;
                                if row_y + cell_h < term_body_y || row_y > term_body_y + body_h {
                                    continue;
                                }

                                // Scan row into spans of contiguous cells with identical attributes
                                let mut cur_span_text = String::new();
                                let mut cur_span_start = 0u16;
                                let mut cur_span_fg = colors.text_primary;
                                let mut cur_span_bg: Option<Color> = None;
                                let mut cur_span_bold = false;
                                let mut in_span = false;

                                for c in 0..cols {
                                    let cell = screen.cell(r, c);
                                    let contents = cell.map(|cell_ref| cell_ref.contents()).unwrap_or(" ");
                                    let fg = cell.map(|cell_ref| convert_vt_color(cell_ref.fgcolor(), colors.text_primary)).unwrap_or(colors.text_primary);
                                    let bg = cell.and_then(|cell_ref| match cell_ref.bgcolor() {
                                        vt100::Color::Default => None,
                                        other => Some(convert_vt_color(other, Color::TRANSPARENT)),
                                    });
                                    let bold = cell.map(|cell_ref| cell_ref.bold()).unwrap_or(false);

                                    // Handle inverse video
                                    let (fg, bg) = if cell.map(|cell_ref| cell_ref.inverse()).unwrap_or(false) {
                                        (bg.unwrap_or(colors.bg_base), Some(fg))
                                    } else {
                                        (fg, bg)
                                    };

                                    if in_span && (fg != cur_span_fg || bg != cur_span_bg || bold != cur_span_bold) {
                                        // Flush current span
                                        let span_x = term_body_x + pad_left + cur_span_start as f32 * cell_w;
                                        let span_w = (c - cur_span_start) as f32 * cell_w;
                                        if let Some(bg_c) = cur_span_bg {
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [span_x, row_y],
                                                    size: [span_w, cell_h],
                                                    color: bg_c.to_array(),
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                },
                                            }));
                                        }
                                        if !cur_span_text.trim().is_empty() {
                                            let mut opts = TextOptions::new()
                                                .font_size(font_sz)
                                                .scale_factor(ui.scale_factor)
                                                .font_family("monospace")
                                                .color(cur_span_fg);
                                            if cur_span_bold {
                                                opts = opts.font_weight(FontWeight::Bold);
                                            }
                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                text: std::mem::take(&mut cur_span_text),
                                                pos: [span_x, row_y + 1.0],
                                                options: opts,
                                                clip,
                                            }));
                                        } else {
                                            cur_span_text.clear();
                                        }
                                        in_span = false;
                                    }

                                    if !in_span {
                                        cur_span_start = c;
                                        cur_span_fg = fg;
                                        cur_span_bg = bg;
                                        cur_span_bold = bold;
                                        in_span = true;
                                    }

                                    cur_span_text.push_str(if contents.is_empty() { " " } else { contents });
                                }

                                if in_span {
                                    let span_x = term_body_x + pad_left + cur_span_start as f32 * cell_w;
                                    let span_w = (cols - cur_span_start) as f32 * cell_w;
                                    if let Some(bg_c) = cur_span_bg {
                                        ui.draws.push(DrawCommand::Rect(RectDraw {
                                            instance: RectInstance {
                                                pos: [span_x, row_y],
                                                size: [span_w, cell_h],
                                                color: bg_c.to_array(),
                                                clip_rect: clip,
                                                ..Default::default()
                                            },
                                        }));
                                    }
                                    if !cur_span_text.trim().is_empty() {
                                        let mut opts = TextOptions::new()
                                            .font_size(font_sz)
                                            .scale_factor(ui.scale_factor)
                                            .font_family("monospace")
                                            .color(cur_span_fg);
                                        if cur_span_bold {
                                            opts = opts.font_weight(FontWeight::Bold);
                                        }
                                        ui.draws.push(DrawCommand::Text(TextDraw {
                                            text: cur_span_text,
                                            pos: [span_x, row_y + 1.0],
                                            options: opts,
                                            clip,
                                        }));
                                    }
                                }
                            }

                            // Draw cursor at screen.cursor_position()
                            if !hide_cursor && rows > 0 && cols > 0 {
                                let safe_col = cursor_col.min(cols.saturating_sub(1));
                                let safe_row = cursor_row.min(rows.saturating_sub(1));
                                let cur_x = term_body_x + pad_left + safe_col as f32 * cell_w;
                                let cur_y = term_body_y + pad_top + safe_row as f32 * cell_h;

                                if is_focused {
                                    let elapsed_ms = state.last_cursor_activity.elapsed().as_millis();
                                    let show_cursor = if elapsed_ms < 500 {
                                        true
                                    } else {
                                        ((elapsed_ms - 500) / 530) % 2 == 0
                                    };
                                    if show_cursor {
                                        ui.draws.push(DrawCommand::Rect(RectDraw {
                                            instance: RectInstance {
                                                pos: [cur_x, cur_y + 1.0],
                                                size: [cell_w.max(2.0), (font_sz + 3.0).max(14.0)],
                                                color: colors.accent.to_array(),
                                                radius: [0.0; 4],
                                                clip_rect: clip,
                                                ..Default::default()
                                            },
                                        }));
                                    }
                                } else {
                                    // Hollow cursor when terminal is not actively focused
                                    ui.draws.push(DrawCommand::Rect(RectDraw {
                                        instance: RectInstance {
                                            pos: [cur_x, cur_y + 1.0],
                                            size: [cell_w.max(2.0), (font_sz + 3.0).max(14.0)],
                                            color: [0.0; 4],
                                            border_width: 1.0,
                                            border_color: colors.text_muted.to_array(),
                                            radius: [0.0; 4],
                                            clip_rect: clip,
                                            ..Default::default()
                                        },
                                    }));
                                }
                            }
                        }
                    } else {
                        ui.text("Initializing terminal...")
                            .size(11.0)
                            .monospace()
                            .color(colors.text_muted)
                            .show();
                    }
                });

            if body_resp.clicked {
                ui.focused_id = Some(term_id);
                state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                state.last_cursor_activity = std::time::Instant::now();
                ui.needs_redraw = true;
            }
        });

    if term_panel.clicked {
        ui.focused_id = Some(term_id);
        state.focused_panel = crate::state::types::FocusedPanel::Terminal;
        state.last_cursor_activity = std::time::Instant::now();
        ui.needs_redraw = true;
    }
}
