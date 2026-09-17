use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};

pub fn draw_terminal_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = state.colors();
    state.ensure_terminal_session();

    let term_id = Id::from_u64(999999800);
    let is_focused = ui.focused_id == Some(term_id);

    // ─── Top Horizontal Edge Handler (Resize Splitter) ───────────
    let splitter_h = 6.0;
    let splitter_x = ui.cursor_x;
    let splitter_y = ui.cursor_y;
    let splitter_w = width;
    let is_splitter_hovered = ui.mouse_in_rect(
        splitter_x,
        splitter_y - splitter_h / 2.0,
        splitter_w,
        splitter_h,
    );

    if is_splitter_hovered || state.active_resize_terminal {
        ui.cursor_icon = CursorIcon::RowResize;
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
    let edge_draw_y = splitter_y - edge_thickness / 2.0;

    ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
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

    let is_panel_hovered = ui.mouse_in_rect(splitter_x, splitter_y, width, height);

    // Mouse wheel scrolling over terminal panel
    if is_panel_hovered {
        for event in &ui.input_events {
            if let zenthra_platform::event::PlatformEvent::MouseWheel { delta_y, .. } = event {
                if *delta_y > 0.0 {
                    state.terminal_scroll_offset = state.terminal_scroll_offset.saturating_add(3);
                    ui.needs_redraw = true;
                } else if *delta_y < 0.0 {
                    state.terminal_scroll_offset = state.terminal_scroll_offset.saturating_sub(3);
                    ui.needs_redraw = true;
                }
            }
        }
    }

/// Truncates a string to at most `max_chars` characters, safely aligning to UTF-8 character boundaries.
pub fn safe_truncate_str(s: &str, max_chars: usize) -> &str {
    match s.char_indices().nth(max_chars) {
        Some((idx, _)) => &s[..idx],
        None => s,
    }
}

    // ─── Input Event Processing directly inside Terminal ─────────
    if is_focused {
        ui.request_redraw_after(std::time::Duration::from_millis(16));
        let events = std::mem::take(&mut ui.input_events);

        for event in &events {
            match event {
                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                    if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                        state.terminal_input.push(*c);
                        state.terminal_scroll_offset = 0;
                        state.terminal_last_activity = std::time::Instant::now();
                        ui.needs_redraw = true;
                    }
                }
                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                    match key {
                        winit::keyboard::KeyCode::KeyC if state.ctrl_pressed => {
                            state.send_terminal_sigint();
                            state.terminal_scroll_offset = 0;
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::KeyL if state.ctrl_pressed => {
                            state.clear_terminal();
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::KeyV if state.ctrl_pressed => {
                            if let Some(text) = state.get_clipboard() {
                                state.terminal_input.push_str(&text);
                                state.terminal_scroll_offset = 0;
                                state.terminal_last_activity = std::time::Instant::now();
                                ui.needs_redraw = true;
                            }
                        }
                        winit::keyboard::KeyCode::ArrowUp => {
                            state.terminal_history_prev();
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::ArrowDown => {
                            state.terminal_history_next();
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            state.terminal_input.pop();
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Enter => {
                            let cmd = state.terminal_input.clone();
                            state.send_terminal_input(&cmd);
                            state.terminal_scroll_offset = 0;
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Tab => {
                            if let Some(ref mut sess) = state.terminal_session {
                                sess.write_raw(b"\t");
                            }
                            state.terminal_last_activity = std::time::Instant::now();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Escape => {
                            ui.focused_id = None;
                            ui.needs_redraw = true;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // ─── Main Terminal Panel Box ─────────────────────────────────
    let term_panel = ui.container()
        .id(term_id)
        .width(width)
        .height(height)
        .bg(match state.theme {
            crate::theme::ThemeMode::Dark => Color::rgb(22.0 / 255.0, 24.0 / 255.0, 26.0 / 255.0),
            crate::theme::ThemeMode::Light => Color::rgb(245.0 / 255.0, 243.0 / 255.0, 235.0 / 255.0),
        })
        .border(colors.border, 1.0)
        .column()
        .show(|ui| {
            // ─── Header Bar using SpaceBetween ───────────────────────
            const HEADER_H: f32 = 28.0;
            ui.container()
                .width(width)
                .height(HEADER_H)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding_left(8.0)
                .padding_right(8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .no_wrap()
                .show(|ui| {
                    // Child 1: Left Title + Shell Badge + Quick Command Chips
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text(">_")
                                .size(11.0)
                                .weight(FontWeight::Bold)
                                .color(colors.accent)
                                .show();

                            if width >= 340.0 {
                                ui.text("TERMINAL")
                                    .size(11.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            } else if width >= 260.0 {
                                ui.text("TERM")
                                    .size(10.5)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            }

                            // Native PTY shell badge
                            let shell = std::env::var("SHELL").unwrap_or_else(|_| "sh".to_string());
                            let shell_name = std::path::Path::new(&shell)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("sh");

                            if width >= 640.0 {
                                ui.container()
                                    .padding(1.0, 5.0, 1.0, 5.0)
                                    .bg(colors.bg_base)
                                    .border(colors.border, 1.0)
                                    .radius_all(3.0)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(4.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(5.0)
                                            .height(5.0)
                                            .radius_all(2.5)
                                            .bg(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                                            .show(|_| {});

                                        ui.text(&format!("{} (native PTY)", shell_name))
                                            .size(9.5)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                            } else if width >= 480.0 {
                                ui.container()
                                    .padding(1.0, 5.0, 1.0, 5.0)
                                    .bg(colors.bg_base)
                                    .border(colors.border, 1.0)
                                    .radius_all(3.0)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(4.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(5.0)
                                            .height(5.0)
                                            .radius_all(2.5)
                                            .bg(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                                            .show(|_| {});

                                        ui.text(shell_name)
                                            .size(9.5)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                            } else if width >= 360.0 {
                                ui.container()
                                    .width(6.0)
                                    .height(6.0)
                                    .radius_all(3.0)
                                    .bg(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                                    .show(|_| {});
                            }

                            // Command Chips: click to execute directly in terminal
                            let chips: &[&str] = if width >= 800.0 {
                                &["cargo check", "cargo test", "git status", "ls -la"]
                            } else if width >= 660.0 {
                                &["cargo check", "cargo test", "git status"]
                            } else if width >= 540.0 {
                                &["cargo check", "git status"]
                            } else if width >= 440.0 {
                                &["cargo check"]
                            } else {
                                &[]
                            };

                            for chip in chips {
                                let chip_btn = ui.container()
                                    .padding(2.0, 5.0, 2.0, 5.0)
                                    .bg(colors.bg_base)
                                    .hover_bg(colors.highlight)
                                    .border(colors.border, 1.0)
                                    .radius_all(3.0)
                                    .show(|ui| {
                                        ui.text(*chip)
                                            .size(9.5)
                                            .color(colors.text_muted)
                                            .show();
                                    });

                                if chip_btn.clicked {
                                    state.send_terminal_input(chip);
                                    ui.focused_id = Some(term_id);
                                    ui.needs_redraw = true;
                                }
                            }
                        });

                    // Child 2: Right Actions: Clear + Close
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            if width >= 360.0 {
                                let clear_btn = ui.container()
                                    .padding(2.0, 6.0, 2.0, 6.0)
                                    .hover_bg(colors.highlight)
                                    .radius_all(3.0)
                                    .show(|ui| {
                                        ui.text("Clear")
                                            .size(10.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });

                                if clear_btn.clicked {
                                    state.clear_terminal();
                                    ui.needs_redraw = true;
                                }
                            } else {
                                let clear_btn = ui.container()
                                    .width(18.0)
                                    .height(18.0)
                                    .radius_all(3.0)
                                    .hover_bg(colors.highlight)
                                    .halign(Align::Center)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text("\u{f1f8}")
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });

                                if clear_btn.clicked {
                                    state.clear_terminal();
                                    ui.needs_redraw = true;
                                }
                            }

                            let close_btn = ui.container()
                                .width(18.0)
                                .height(18.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f00d}")
                                        .size(9.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if close_btn.clicked {
                                state.terminal_visible = false;
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }
                        });
                });

            // ─── Free Native Terminal Console Screen ─────────────────
            // Directly interactive: click anywhere to focus, type freely with inline prompt & blinking cursor
            let body_h = (height - 28.0).max(40.0);
            let term_body_x = ui.cursor_x;
            let term_body_y = ui.cursor_y;

            let is_term_hovered = ui.mouse_in_rect(term_body_x, term_body_y, width, body_h);
            if is_term_hovered {
                ui.cursor_icon = CursorIcon::Text;
            }

            let body_resp = ui.container()
                .width(width)
                .height(body_h)
                .padding(6.0, 10.0, 6.0, 10.0)
                .column()
                .gap(2.0)
                .show(|ui| {
                    let line_h = 17.0;
                    let max_visible = ((body_h - 24.0) / line_h).floor().max(1.0) as usize;

                    let (visible_lines, pending) = if let Some(ref sess) = state.terminal_session {
                        sess.get_visible_lines(max_visible, state.terminal_scroll_offset)
                    } else {
                        let total = state.terminal_lines.len();
                        let effective_end = total.saturating_sub(state.terminal_scroll_offset);
                        let start = effective_end.saturating_sub(max_visible);
                        let slice = if start < effective_end && effective_end <= total {
                            state.terminal_lines[start..effective_end].iter().map(|l| l.text.clone()).collect()
                        } else {
                            Vec::new()
                        };
                        (slice, String::new())
                    };

                    let max_chars = ((width - 24.0) / 7.2).max(10.0) as usize;

                    // 1. Output Lines
                    for line in &visible_lines {
                        let text_color = if line.starts_with('$') || line.starts_with('❯') {
                            colors.accent
                        } else if line.contains("error:") || line.contains("Error:") || line.contains("fatal:") {
                            Color::rgb(251.0 / 255.0, 73.0 / 255.0, 52.0 / 255.0)
                        } else if line.contains("warning:") || line.contains("Warning:") {
                            Color::rgb(250.0 / 255.0, 189.0 / 255.0, 47.0 / 255.0)
                        } else if line.contains("Compiling") || line.contains("Finished") || line.contains("ok") {
                            Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0)
                        } else {
                            colors.text_primary
                        };

                        let display_line = safe_truncate_str(line, max_chars);

                        ui.text(display_line)
                            .size(11.0)
                            .color(text_color)
                            .show();
                    }

                    // 2. Active Streaming / Pending Output Line (if at bottom)
                    if state.terminal_scroll_offset == 0 && !pending.is_empty() {
                        let display_pending = safe_truncate_str(&pending, max_chars);

                        ui.text(display_pending)
                            .size(11.0)
                            .color(colors.text_muted)
                            .show();
                    }

                    // 3. Active Inline Command Line (Prompt + User Typed Text + Blinking Cursor)
                    ui.container()
                        .full_width()
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text("❯")
                                .size(11.0)
                                .weight(FontWeight::Bold)
                                .color(colors.accent)
                                .show();

                            let is_typing_active = state.terminal_last_activity.elapsed() < std::time::Duration::from_millis(500);
                            let blink = if is_focused {
                                if is_typing_active {
                                    true // Solid cursor while actively typing: zero flicker
                                } else {
                                    (ui.elapsed_time * 2.5).fract() < 0.6
                                }
                            } else {
                                false
                            };
                            let cursor_char = if is_focused && blink { "█" } else { " " };

                            let input_str = &state.terminal_input;
                            let total_chars = input_str.chars().count();
                            let display_input = if total_chars > max_chars.saturating_sub(2) {
                                let skip = total_chars.saturating_sub(max_chars.saturating_sub(2));
                                let start_idx = input_str.char_indices().nth(skip).map(|(i, _)| i).unwrap_or(0);
                                &input_str[start_idx..]
                            } else {
                                input_str.as_str()
                            };
                            let prompt_display = format!("{}{}", display_input, cursor_char);

                            ui.text(&prompt_display)
                                .size(11.0)
                                .color(colors.text_primary)
                                .show();

                            if state.terminal_scroll_offset > 0 {
                                ui.text(&format!("(history +{} lines)", state.terminal_scroll_offset))
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            }
                        });
                });

            if body_resp.clicked {
                ui.focused_id = Some(term_id);
                ui.needs_redraw = true;
            }
        });

    if term_panel.clicked {
        ui.focused_id = Some(term_id);
        ui.needs_redraw = true;
    }
}
