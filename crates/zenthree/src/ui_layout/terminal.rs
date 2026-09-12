use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id};

pub fn draw_terminal_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = state.colors();
    state.ensure_terminal_session();

    let term_id = Id::from_u64(999999800);
    let is_focused = ui.focused_id == Some(term_id);

    // Top horizontal splitter for dragging terminal height
    let splitter_h = 6.0;
    let splitter_y = ui.cursor_y;
    let splitter_w = width;
    let is_splitter_hovered = ui.mouse_in_rect(
        ui.cursor_x,
        splitter_y - splitter_h / 2.0,
        splitter_w,
        splitter_h,
    );

    if is_splitter_hovered && ui.mouse_down {
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
            state.terminal_height = target_h.clamp(80.0, (ui.height * 0.7).max(120.0));
            ui.request_redraw();
        }
    }

    // Input event processing when terminal input is focused
    if is_focused {
        let events = std::mem::take(&mut ui.input_events);
        let mut trigger_send = false;

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
                        winit::keyboard::KeyCode::Backspace => {
                            state.terminal_input.pop();
                            ui.needs_redraw = true;
                        }
                        winit::keyboard::KeyCode::Enter => {
                            trigger_send = true;
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

        if trigger_send {
            let cmd = state.terminal_input.clone();
            state.send_terminal_input(&cmd);
            ui.needs_redraw = true;
        }
    }

    // Main Terminal Panel Box
    let term_panel = ui.container()
        .id(term_id)
        .width(width)
        .height(height)
        .bg(match state.theme {
            crate::theme::ThemeMode::Dark => Color::rgb(24.0 / 255.0, 26.0 / 255.0, 27.0 / 255.0),
            crate::theme::ThemeMode::Light => Color::rgb(245.0 / 255.0, 243.0 / 255.0, 235.0 / 255.0),
        })
        .border(colors.border, 1.0)
        .column()
        .show(|ui| {
            // ─── Header Bar using SpaceBetween (Left items vs Right action buttons) ─
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
                .halign(Align::SpaceBetween) // Uses SpaceBetween properly!
                .no_wrap()
                .show(|ui| {
                    // Child 1: Left Title + Shell Badge + Command Chips
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

                            // Command Chips (adapted to width)
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
                                    ui.needs_redraw = true;
                                }
                            }
                        });

                    // Child 2: Right Actions: Clear + Close (always pinned to the right edge)
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
                                    if let Some(ref sess) = state.terminal_session {
                                        if let Ok(mut l) = sess.lines.lock() {
                                            l.clear();
                                        }
                                    }
                                    state.terminal_lines.clear();
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
                                    if let Some(ref sess) = state.terminal_session {
                                        if let Ok(mut l) = sess.lines.lock() {
                                            l.clear();
                                        }
                                    }
                                    state.terminal_lines.clear();
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

            // ─── Terminal Lines Output Area ──────────────────────────
            const INPUT_H: f32 = 30.0;
            let body_h = (height - 28.0 - INPUT_H).max(30.0);

            ui.container()
                .width(width)
                .height(body_h)
                .padding(6.0, 8.0, 4.0, 8.0)
                .column()
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

                    let line_h = 16.0;
                    let max_visible = (body_h / line_h).floor() as usize;
                    let skip = if lines.len() > max_visible {
                        lines.len() - max_visible
                    } else {
                        0
                    };

                    let max_chars = ((width - 24.0) / 7.2).max(10.0) as usize;

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
                });

            // ─── Input Prompt Bar (Align::SpaceBetween) ──────────────
            ui.container()
                .width(width)
                .height(INPUT_H)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding_left(8.0)
                .padding_right(8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween) // Uses SpaceBetween properly!
                .no_wrap()
                .show(|ui| {
                    let run_btn_w: f32 = if width >= 360.0 { 44.0 } else { 24.0 };
                    let input_box_w = (width - 16.0 - run_btn_w - 10.0).max(40.0);

                    let placeholder = if width >= 540.0 {
                        "Type a command... (e.g. cargo check, git status, ls)"
                    } else if width >= 380.0 {
                        "Type a command... (e.g. cargo check)"
                    } else if width >= 260.0 {
                        "Type a command..."
                    } else {
                        ""
                    };

                    let prompt_text = if state.terminal_input.is_empty() {
                        if is_focused { "|" } else { placeholder }
                    } else {
                        &state.terminal_input
                    };

                    let text_color = if state.terminal_input.is_empty() && !is_focused {
                        colors.text_muted.with_alpha(0.5)
                    } else {
                        colors.text_primary
                    };

                    let max_input_chars = ((input_box_w - 24.0) / 7.2).max(8.0) as usize;
                    let display_str = if is_focused && !state.terminal_input.is_empty() {
                        let full = format!("{}|", prompt_text);
                        if full.len() > max_input_chars {
                            format!("...{}", &full[full.len().saturating_sub(max_input_chars - 3)..])
                        } else {
                            full
                        }
                    } else if prompt_text.len() > max_input_chars {
                        format!("...{}", &prompt_text[prompt_text.len().saturating_sub(max_input_chars - 3)..])
                    } else {
                        prompt_text.to_string()
                    };

                    // Child 1: Input text container
                    let input_area = ui.container()
                        .width(input_box_w)
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text("❯")
                                .size(11.0)
                                .weight(FontWeight::Bold)
                                .color(colors.accent)
                                .show();

                            ui.text(&display_str)
                                .size(11.0)
                                .color(text_color)
                                .show();
                        });

                    if input_area.clicked {
                        ui.focused_id = Some(term_id);
                        ui.needs_redraw = true;
                    }

                    // Child 2: Run button (pinned to right)
                    let send_btn = if width >= 360.0 {
                        ui.container()
                            .width(run_btn_w)
                            .height(22.0)
                            .bg(colors.accent)
                            .hover_bg(colors.highlight)
                            .radius_all(3.0)
                            .halign(Align::Center)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("Run")
                                    .size(10.0)
                                    .weight(FontWeight::Bold)
                                    .color(Color::WHITE)
                                    .show();
                            })
                    } else {
                        ui.container()
                            .width(run_btn_w)
                            .height(20.0)
                            .bg(colors.accent)
                            .hover_bg(colors.highlight)
                            .radius_all(3.0)
                            .halign(Align::Center)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("➤")
                                    .size(8.5)
                                    .color(Color::WHITE)
                                    .show();
                            })
                    };

                    if send_btn.clicked {
                        let cmd = state.terminal_input.clone();
                        state.send_terminal_input(&cmd);
                        ui.needs_redraw = true;
                    }
                });
        });

    if term_panel.clicked {
        ui.focused_id = Some(term_id);
        ui.needs_redraw = true;
    }
}
