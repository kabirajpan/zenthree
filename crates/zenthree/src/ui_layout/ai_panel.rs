use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = state.colors();
    let ai_panel_w = width;
    let ai_id = Id::from_u64(999999801);
    let ai_input_id = ui.id_from("ai_prompt_text_area");

    // ─── Resizing Splitter Handle ─────────────────────────────────────────
    let handle_w = 6.0;
    let splitter_x = ui.cursor_x;
    let splitter_y = ui.cursor_y;
    let is_splitter_hovered = ui.mouse_in_rect(
        splitter_x - handle_w / 2.0,
        splitter_y,
        handle_w,
        height,
    );

    if is_splitter_hovered || state.active_resize_ai_panel {
        ui.cursor_icon = CursorIcon::ColResize;
    }

    if is_splitter_hovered && ui.mouse_down && ui.active_drag.is_none() {
        state.active_resize_ai_panel = true;
    }

    if state.active_resize_ai_panel {
        if !ui.mouse_down {
            state.active_resize_ai_panel = false;
            state.save_persistent_state();
        } else {
            let target_w = ui.width - ui.mouse_x;
            state.ai_panel_width = target_w.clamp(200.0, (ui.width * 0.55).max(220.0));
            ui.request_redraw();
        }
    }

    // Smooth subtle divider line
    let edge_color = if state.active_resize_ai_panel {
        colors.accent
    } else if is_splitter_hovered {
        colors.accent.with_alpha(0.6)
    } else {
        colors.border.with_alpha(0.7)
    };

    let edge_thickness = if state.active_resize_ai_panel || is_splitter_hovered { 2.0 } else { 1.0 };
    let edge_draw_x = splitter_x - edge_thickness / 2.0;

    ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [edge_draw_x, splitter_y],
            size: [edge_thickness, height],
            color: edge_color.to_array(),
            radius: [0.0; 4],
            border_width: 0.0,
            border_color: [0.0; 4],
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));

    // Center pill grip handle when hovered or active
    if is_splitter_hovered || state.active_resize_ai_panel {
        let grip_w = 4.0;
        let grip_h = 32.0;
        let grip_x = splitter_x - grip_w / 2.0;
        let grip_y = splitter_y + (height - grip_h) / 2.0;
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

    // ─── Main AI Agent Panel ──────────────────────────────────────────────
    let _panel_container = ui.container()
        .id(ai_id)
        .width(ai_panel_w)
        .height(height)
        .max_height(height)
        .clip(true)
        .bg(colors.bg_sidebar)
        .column()
        .show(|ui| {
            // ─── Header: Minimalist Top Bar ──────────────────────────────
            const HEADER_H: f32 = 36.0;
            ui.container()
                .width(ai_panel_w)
                .height(HEADER_H)
                .bg(colors.bg_sidebar)
                .padding_left(12.0)
                .padding_right(10.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .no_wrap()
                .show(|ui| {
                    // Left Brand & Status
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(7.0)
                        .show(|ui| {
                            // Pulsing status dot
                            ui.container()
                                .width(7.0)
                                .height(7.0)
                                .radius_all(3.5)
                                .bg(colors.accent)
                                .show(|_| {});

                            ui.text("Assistant")
                                .size(12.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Right Actions (Clear & Close)
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            // Clear chat button
                            let reset_btn = ui.container()
                                .width(22.0)
                                .height(22.0)
                                .radius_all(4.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f01e}")
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if reset_btn.clicked {
                                state.ai_messages.clear();
                                ui.needs_redraw = true;
                            }

                            // Close button
                            let close_btn = ui.container()
                                .width(22.0)
                                .height(22.0)
                                .radius_all(4.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f00d}")
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if close_btn.clicked {
                                state.ai_panel_visible = false;
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }
                        });
                });

            // Subtle 1px hairline below header
            ui.container()
                .width(ai_panel_w)
                .height(1.0)
                .bg(colors.border.with_alpha(0.5))
                .show(|_| {});

            // ─── Dynamic Height Calculations ─────────────────────────────
            let is_input_focused = ui.focused_id == Some(ai_input_id);
            const MIN_PROMPT_H: f32 = 32.0;
            const MAX_PROMPT_H: f32 = 120.0;

            // Immediately track visual wrapped lines while typing continuously without newline
            let chars_per_line = ((ai_panel_w - 44.0).max(60.0) / 6.6).max(8.0);
            let visual_lines: usize = state.ai_input.split('\n').map(|para| {
                let len = para.chars().count();
                if len == 0 { 1 } else { (len as f32 / chars_per_line).ceil() as usize }
            }).sum();
            let estimated_h = (visual_lines as f32 * 16.0 + 16.0).clamp(MIN_PROMPT_H, MAX_PROMPT_H);
            let recorded_h = ui.get_recorded_layout(ai_input_id).map(|(r, _)| r.size.height);
            let prompt_box_h = if state.ai_input.is_empty() {
                MIN_PROMPT_H
            } else if let Some(rec) = recorded_h {
                rec.max(estimated_h).clamp(MIN_PROMPT_H, MAX_PROMPT_H)
            } else {
                estimated_h
            };

            // Dock height: inner padding (12) + gap (4) + send row (22) + outer dock padding (14) = prompt_box_h + 52.0
            let dock_total_h = prompt_box_h + 52.0;
            let msgs_h = (height - HEADER_H - 1.0 - dock_total_h).max(30.0);

            // ─── Messages Scroll Area ────────────────────────────────────
            let msgs_scroll_id = Id::from_u64(999999802);
            ui.container()
                .id(msgs_scroll_id)
                .width(ai_panel_w)
                .height(msgs_h)
                .max_height(msgs_h)
                .clip(true)
                .padding(8.0, 12.0, 8.0, 12.0)
                .column()
                .gap(12.0)
                .scroll_y(true)
                .show(|ui| {
                    if state.ai_messages.is_empty() {
                        // Minimalist Welcome State
                        ui.container()
                            .full_width()
                            .padding_top(30.0)
                            .column()
                            .halign(Align::Center)
                            .gap(8.0)
                            .show(|ui| {
                                ui.container()
                                    .width(36.0)
                                    .height(36.0)
                                    .radius_all(18.0)
                                    .bg(colors.accent.with_alpha(0.12))
                                    .border(colors.accent.with_alpha(0.25), 1.0)
                                    .halign(Align::Center)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text("\u{f005}")
                                            .size(14.0)
                                            .color(colors.accent)
                                            .show();
                                    });

                                ui.text("How can I assist you today?")
                                    .size(12.0)
                                    .weight(FontWeight::Medium)
                                    .color(colors.text_primary)
                                    .show();

                                ui.text("Ask questions, refactor code, or generate tests.")
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();
                            });
                    } else {
                        let mut toggle_expand_idx: Option<usize> = None;
                        let mut copy_target_idx: Option<usize> = None;

                        for (msg_idx, msg) in state.ai_messages.iter().enumerate() {
                            let is_copied = state.copied_ai_msg
                                .as_ref()
                                .map(|(idx, t)| *idx == msg_idx && t.elapsed().as_secs_f32() < 2.0)
                                .unwrap_or(false);

                            let bubble_id = Id::from_u64(999990000 + (msg_idx as u64) * 2);
                            let copy_btn_id = Id::from_u64(999990000 + (msg_idx as u64) * 2 + 1);
                            let mut copy_clicked = false;

                            let full_text: &str = &msg.content;
                            let is_thinking = !msg.is_user && full_text == "Thinking...";

                            // Fast byte check for long text - zero heap allocation
                            let newline_count = full_text.as_bytes().iter().filter(|&&b| b == b'\n').count();
                            let is_long = !is_thinking && (newline_count >= 4 || full_text.len() > 200);
                            let is_expanded = msg.expanded;

                            // Truncate to first 4 lines directly for collapsed view - saves 95% text shaping overhead
                            let display_text: &str = if is_long && !is_expanded {
                                let mut end = full_text.len();
                                let mut lines = 0;
                                for (i, b) in full_text.bytes().enumerate() {
                                    if b == b'\n' {
                                        lines += 1;
                                        if lines >= 4 {
                                            end = i;
                                            break;
                                        }
                                    }
                                    if i >= 240 && lines >= 2 {
                                        end = i;
                                        break;
                                    }
                                }
                                let mut safe_end = end.min(full_text.len());
                                while !full_text.is_char_boundary(safe_end) {
                                    safe_end -= 1;
                                }
                                &full_text[..safe_end]
                            } else {
                                full_text
                            };

                            if msg.is_user {
                                // ─── User Message (Right-aligned, accent tint) ───
                                let bubble_max_w = (ai_panel_w - 36.0).max(80.0);
                                let user_text_w = (bubble_max_w - 20.0).max(60.0);

                                ui.container()
                                    .full_width()
                                    .row()
                                    .halign(Align::Right)
                                    .show(|ui| {
                                        let bubble_res = ui.container()
                                            .id(bubble_id)
                                            .max_width(bubble_max_w)
                                            .padding(8.0, 10.0, 6.0, 10.0)
                                            .bg(colors.accent.with_alpha(0.12))
                                            .hover_bg(colors.accent.with_alpha(0.16))
                                            .border(colors.accent.with_alpha(0.3), 0.5)
                                            .radius_all(10.0)
                                            .column()
                                            .gap(4.0)
                                            .show(|ui| {
                                                ui.text(display_text)
                                                    .size(11.0)
                                                    .line_height(1.35)
                                                    .color(colors.text_primary)
                                                    .max_width(user_text_w)
                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                    .show();

                                                ui.container()
                                                    .width(user_text_w)
                                                    .row()
                                                    .halign(Align::Right)
                                                    .valign(Align::Center)
                                                    .gap(6.0)
                                                    .show(|ui| {
                                                        if is_long {
                                                            ui.text(if is_expanded { "Show less" } else { "Show more" })
                                                                .size(9.0)
                                                                .color(colors.accent.with_alpha(0.85))
                                                                .show();
                                                        }

                                                        let copy_btn = ui.container()
                                                            .id(copy_btn_id)
                                                            .padding(2.0, 4.0, 2.0, 4.0)
                                                            .radius_all(4.0)
                                                            .hover_bg(colors.highlight)
                                                            .row()
                                                            .valign(Align::Center)
                                                            .gap(3.0)
                                                            .show(|ui| {
                                                                ui.text(if is_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                    .size(9.5)
                                                                    .color(if is_copied { colors.accent } else { colors.text_muted })
                                                                    .show();
                                                                if is_copied {
                                                                    ui.text("Copied")
                                                                        .size(9.0)
                                                                        .color(colors.accent)
                                                                        .show();
                                                                }
                                                            });

                                                        if copy_btn.hovered {
                                                            ui.cursor_icon = CursorIcon::Pointer;
                                                        }
                                                        if copy_btn.clicked {
                                                            copy_clicked = true;
                                                        }
                                                    });
                                            });

                                        if copy_clicked {
                                            copy_target_idx = Some(msg_idx);
                                        } else if bubble_res.clicked && is_long {
                                            toggle_expand_idx = Some(msg_idx);
                                        }
                                        if bubble_res.hovered && is_long {
                                            ui.cursor_icon = CursorIcon::Pointer;
                                        }
                                    });
                            } else {
                                // ─── Assistant Message (Left-aligned, matching card) ───
                                let bubble_max_w = (ai_panel_w - 28.0).max(80.0);
                                let ai_text_w = (bubble_max_w - 20.0).max(60.0);

                                ui.container()
                                    .full_width()
                                    .row()
                                    .halign(Align::Left)
                                    .show(|ui| {
                                        let bubble_res = ui.container()
                                            .id(bubble_id)
                                            .max_width(bubble_max_w)
                                            .padding(8.0, 10.0, 6.0, 10.0)
                                            .bg(colors.bg_sidebar.with_alpha(0.55))
                                            .hover_bg(colors.bg_sidebar.with_alpha(0.75))
                                            .border(colors.border.with_alpha(0.5), 0.5)
                                            .radius_all(10.0)
                                            .column()
                                            .gap(4.0)
                                            .show(|ui| {
                                                if is_thinking {
                                                    ui.text("Thinking...")
                                                        .size(11.0)
                                                        .italic()
                                                        .color(colors.accent)
                                                        .show();
                                                } else {
                                                    ui.text(display_text)
                                                        .size(11.0)
                                                        .line_height(1.35)
                                                        .color(colors.text_primary)
                                                        .max_width(ai_text_w)
                                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                                        .show();
                                                }

                                                if !is_thinking {
                                                    ui.container()
                                                        .width(ai_text_w)
                                                        .row()
                                                        .halign(Align::Right)
                                                        .valign(Align::Center)
                                                        .gap(6.0)
                                                        .show(|ui| {
                                                            if is_long {
                                                                ui.text(if is_expanded { "Show less" } else { "Show more" })
                                                                    .size(9.0)
                                                                    .color(colors.accent.with_alpha(0.85))
                                                                    .show();
                                                            }

                                                            let copy_btn = ui.container()
                                                                .id(copy_btn_id)
                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                .radius_all(4.0)
                                                                .hover_bg(colors.highlight)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(3.0)
                                                                .show(|ui| {
                                                                    ui.text(if is_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                        .size(9.5)
                                                                        .color(if is_copied { colors.accent } else { colors.text_muted })
                                                                        .show();
                                                                    if is_copied {
                                                                        ui.text("Copied")
                                                                            .size(9.0)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                    }
                                                                });

                                                            if copy_btn.hovered {
                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                            }
                                                            if copy_btn.clicked {
                                                                copy_clicked = true;
                                                            }
                                                        });
                                                }
                                            });

                                        if copy_clicked {
                                            copy_target_idx = Some(msg_idx);
                                        } else if bubble_res.clicked && is_long {
                                            toggle_expand_idx = Some(msg_idx);
                                        }
                                        if bubble_res.hovered && is_long {
                                            ui.cursor_icon = CursorIcon::Pointer;
                                        }
                                    });
                            }
                        }

                        if let Some(idx) = toggle_expand_idx {
                            if let Some(m) = state.ai_messages.get_mut(idx) {
                                m.expanded = !m.expanded;
                                ui.needs_redraw = true;
                            }
                        }

                        if let Some(idx) = copy_target_idx {
                            if let Some(m) = state.ai_messages.get(idx) {
                                if let Ok(mut cb) = arboard::Clipboard::new() {
                                    let _ = cb.set_text(m.content.clone());
                                }
                            }
                            state.copied_ai_msg = Some((idx, std::time::Instant::now()));
                            state.status_message = "Copied to clipboard".to_string();
                            ui.needs_redraw = true;
                        }
                    }
                });

            // ─── Floating Input Dock (Clean, No Badges, Precise Height) ──
            ui.container()
                .width(ai_panel_w)
                .height(dock_total_h)
                .max_height(dock_total_h)
                .padding(6.0, 10.0, 8.0, 10.0)
                .column()
                .show(|ui| {
                    let dock_border_color = if is_input_focused {
                        colors.accent.with_alpha(0.7)
                    } else {
                        colors.border.with_alpha(0.4)
                    };

                    let input_box = ui.container()
                        .full_width()
                        .bg(colors.bg_base)
                        .border(dock_border_color, 0.5) // Thin outline (half of 1.0)
                        .radius_all(7.0)
                        .padding(6.0, 8.0, 6.0, 8.0)
                        .column()
                        .gap(4.0)
                        .show(|ui| {
                            // Enter key handling: Enter sends, Shift+Enter inserts newline
                            let mut trigger_send = false;
                            if is_input_focused {
                                ui.input_events.retain(|event| {
                                    match event {
                                        zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                                            if *key == winit::keyboard::KeyCode::Enter || *key == winit::keyboard::KeyCode::NumpadEnter {
                                                if !state.shift_pressed {
                                                    trigger_send = true;
                                                    return false;
                                                }
                                            } else if *key == winit::keyboard::KeyCode::Escape {
                                                ui.focused_id = None;
                                                ui.needs_redraw = true;
                                                return false;
                                            }
                                        }
                                        zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                                            if (*c == '\r' || *c == '\n') && !state.shift_pressed {
                                                return false;
                                            }
                                        }
                                        _ => {}
                                    }
                                    true
                                });
                            }

                            // Text area
                            let placeholder = if ai_panel_w >= 280.0 {
                                "Ask assistant... (Enter to send)"
                            } else {
                                "Ask..."
                            };

                            let input_w = (ai_panel_w - 36.0).max(80.0);
                            let text_res = ui.text_area(&mut state.ai_input, "ai_prompt_text_area")
                                .width(input_w)
                                .height(prompt_box_h)
                                .max_height(MAX_PROMPT_H)
                                .scrollable(true)
                                .size(11.5)
                                .line_height(1.25)
                                .color(colors.text_primary)
                                .bg(colors.bg_base)
                                .border(Color::TRANSPARENT, 0.0)
                                .focus_border(Color::TRANSPARENT, 0.0)
                                .padding(2.0, 4.0, 2.0, 4.0)
                                .placeholder(placeholder)
                                .placeholder_color(colors.text_muted.with_alpha(0.45))
                                .wrap(zenthra_text::prelude::TextWrap::Word)
                                .submit_on_enter(true)
                                .show();

                            // Bottom bar of input dock: Send Button
                            ui.container()
                                .full_width()
                                .height(22.0)
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    // Left: Helper hint
                                    ui.text("Shift+Enter for newline")
                                        .size(8.5)
                                        .color(colors.text_muted.with_alpha(0.5))
                                        .show();

                                    // Right: Sleek Send Button (or Stop Button during processing)
                                    let is_processing = state.ai_response_rx.is_some();
                                    let has_text = !state.ai_input.trim().is_empty();

                                    let send_bg = if is_processing {
                                        Color::rgb(0.20, 0.20, 0.23)
                                    } else if has_text {
                                        colors.accent
                                    } else {
                                        colors.bg_sidebar
                                    };

                                    let send_hover_bg = if is_processing {
                                        Color::rgb(0.28, 0.28, 0.32)
                                    } else if has_text {
                                        colors.highlight
                                    } else {
                                        colors.bg_sidebar
                                    };

                                    let send_btn = ui.container()
                                        .width(26.0)
                                        .height(22.0)
                                        .radius_all(5.0)
                                        .bg(send_bg)
                                        .hover_bg(send_hover_bg)
                                        .halign(Align::Center)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            if is_processing {
                                                // Square red box with dark gray background
                                                ui.container()
                                                    .width(8.0)
                                                    .height(8.0)
                                                    .radius_all(1.5)
                                                    .bg(Color::rgb(0.92, 0.26, 0.26))
                                                    .show(|_| {});
                                            } else {
                                                let send_text_color = if has_text { Color::WHITE } else { colors.text_muted.with_alpha(0.5) };
                                                ui.text("➤")
                                                    .size(9.5)
                                                    .color(send_text_color)
                                                    .show();
                                            }
                                        });

                                    if send_btn.hovered && (is_processing || has_text) {
                                        ui.cursor_icon = CursorIcon::Pointer;
                                    }

                                    if is_processing {
                                        if send_btn.clicked {
                                            // Cancel ongoing AI request
                                            state.ai_response_rx = None;
                                            if let Some(last) = state.ai_messages.iter_mut().rev().find(|m| !m.is_user) {
                                                if last.content == "Thinking..." {
                                                    last.content = "Request stopped.".to_string();
                                                }
                                            }
                                            ui.needs_redraw = true;
                                        }
                                    } else if trigger_send || text_res.submitted || send_btn.clicked {
                                        let msg = state.ai_input.trim().to_string();
                                        if !msg.is_empty() {
                                            state.send_ai_message(&msg);
                                        }
                                        state.ai_input.clear();
                                        ui.cursor_state.insert(ai_input_id, 0);
                                        ui.needs_redraw = true;
                                    }
                                });
                        });

                    if input_box.clicked && ui.focused_id != Some(ai_input_id) {
                        ui.focused_id = Some(ai_input_id);
                        ui.needs_redraw = true;
                    }
                });
        });
}
