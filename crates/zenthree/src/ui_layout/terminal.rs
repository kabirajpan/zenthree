use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context};
use crate::theme::ThemeColors;

pub fn draw_terminal_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
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
    let edge_draw_y = if edge_thickness == 1.0 { splitter_y - 1.0 } else { splitter_y - 1.0 };

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

    // ─── PTY Output Poll ─────────────────────────────────────────
    // Poll for new PTY output regardless of focus — output from running commands
    // should appear even while the user is typing in the editor.
    // Only schedule the next poll when a real PTY session is active; when
    // there is no session there is nothing to poll and no reason to wake.
    if state.terminal_session.is_some() {
        ui.request_redraw_after(std::time::Duration::from_millis(250));
    }

    // ─── Input Event Processing directly inside Terminal ─────────
    if is_focused {
        let events = std::mem::take(&mut ui.input_events);

        for event in &events {
            match event {
                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                    if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                        state.terminal_input.push(*c);
                        ui.needs_redraw = true;
                    }
                }
                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                    match key {
                        winit::keyboard::KeyCode::KeyC if state.ctrl_pressed => {
                            state.send_terminal_sigint();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::KeyL if state.ctrl_pressed => {
                            state.clear_terminal();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::ArrowUp => {
                            state.terminal_history_prev();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::ArrowDown => {
                            state.terminal_history_next();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Backspace => {
                            state.terminal_input.pop();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Enter => {
                            let cmd = state.terminal_input.clone();
                            state.send_terminal_input(&cmd);
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Tab => {
                            if let Some(ref mut sess) = state.terminal_session {
                                sess.write_raw(b"\t");
                            }
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

            // 1px divider under terminal header
            ui.container()
                .width(width)
                .height(1.0)
                .bg(colors.border)
                .show(|_| {});

            // ─── Free Native Terminal Console Screen ─────────────────
            // Directly interactive: click anywhere to focus, type freely with inline prompt & blinking cursor
            let body_h = (height - 28.0 - 1.0).max(40.0);
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
                .scroll_y(true)
                .show(|ui| {
                    let lines: Vec<String> = if let Some(ref sess) = state.terminal_session {
                        if let Ok(l) = sess.lines.lock() {
                            l.clone()
                        } else {
                            Vec::new()
                        }
                    } else {
                        state.terminal_lines.iter().map(|l| l.text.clone()).collect()
                    };

                    let line_h = 17.0;
                    let max_visible = ((body_h - 22.0) / line_h).floor() as usize;
                    let skip = if lines.len() > max_visible {
                        lines.len() - max_visible
                    } else {
                        0
                    };

                    let max_chars = ((width - 24.0) / 7.2).max(10.0) as usize;

                    // 1. Output Lines
                    for line in lines.iter().skip(skip) {
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

                        let display_line = if line.len() > max_chars {
                            &line[..max_chars]
                        } else {
                            line.as_str()
                        };

                        ui.text(display_line)
                            .size(11.0)
                            .color(text_color)
                            .show();
                    }

                    // 2. Active Inline Command Line (Prompt + User Typed Text + Blinking Cursor)
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

                            let blink = if is_focused {
                                (ui.elapsed_time * 2.2).fract() < 0.65
                            } else {
                                false
                            };
                            let cursor_char = if is_focused && blink { "█" } else if is_focused { " " } else { " " };
                            let prompt_display = format!("{}{}", state.terminal_input, cursor_char);

                            ui.text(&prompt_display)
                                .size(11.0)
                                .color(colors.text_primary)
                                .show();
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
