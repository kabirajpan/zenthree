use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context};
use crate::theme::ThemeColors;

pub fn draw_terminal_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    state.ensure_terminal_session();

    let term_id = Id::from_u64(999999800);
    let text_area_id = Id::from_u64(999999801);
    let is_focused = ui.focused_id == Some(term_id) || ui.focused_id == Some(text_area_id);

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
    // ─── Input Event Processing directly inside Terminal ─────────
    if is_focused {
        // Intercept special terminal keys before text_area:
        for event in &ui.input_events {
            if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
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
                    winit::keyboard::KeyCode::Tab => {
                        if let Some(ref mut sess) = state.terminal_session {
                            sess.write_raw(b"\t");
                        }
                        ui.needs_redraw = true;
                    }
                    _ => {}
                }
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
                                state.clear_terminal();
                                state.ensure_terminal_session();
                                ui.focused_id = Some(term_id);
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
                                let lines: Vec<String> = if let Some(ref sess) = state.terminal_session {
                                    sess.lines.lock().map(|l| l.clone()).unwrap_or_default()
                                } else {
                                    state.terminal_lines.iter().map(|l| l.text.clone()).collect()
                                };
                                let full_text = lines.join("\n");
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
                                    state.terminal_input.push_str(&text);
                                    ui.focused_id = Some(term_id);
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

                    // 2. Active Inline Command Line (Prompt + ui.text_area with accurate cursor positioning)
                    ui.container()
                        .full_width()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            ui.text("\u{f0142}")
                                .size(11.0)
                                .weight(FontWeight::Bold)
                                .color(colors.accent)
                                .show();

                            let input_w = (width - 45.0).max(60.0);
                            let input_res = ui.text_area(&mut state.terminal_input, text_area_id)
                                .width(input_w)
                                .height(18.0)
                                .size(11.0)
                                .line_height(1.2)
                                .color(colors.text_primary)
                                .bg(Color::TRANSPARENT)
                                .border(Color::TRANSPARENT, 0.0)
                                .focus_border(Color::TRANSPARENT, 0.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .text_padding(0.0, 0.0, 0.0, 0.0)
                                .submit_on_enter(true)
                                .show();

                            if input_res.submitted {
                                let cmd = state.terminal_input.clone();
                                state.send_terminal_input(&cmd);
                                ui.needs_redraw = true;
                            }

                            if input_res.clicked {
                                ui.focused_id = Some(text_area_id);
                                ui.needs_redraw = true;
                            }
                        });
                });

            if body_resp.clicked {
                ui.focused_id = Some(text_area_id);
                ui.needs_redraw = true;
            }
        });

    if term_panel.clicked {
        ui.focused_id = Some(text_area_id);
        ui.needs_redraw = true;
    }
}
