use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = state.colors();
    let ai_panel_w = width;
    let ai_id = Id::from_u64(999999801);
    let ai_input_id = ui.id_from("ai_prompt_text_area");

    // Left vertical splitter handle for resizing panel width
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
            state.ai_panel_width = target_w.clamp(180.0, (ui.width * 0.5).max(200.0));
            ui.request_redraw();
        }
    }

    // Visual feedback for AI panel edge handler
    let edge_color = if state.active_resize_ai_panel {
        colors.accent
    } else if is_splitter_hovered {
        colors.accent.with_alpha(0.8)
    } else {
        colors.border
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
        let grip_h = 36.0;
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

    // Main AI Agent Panel
    let panel_container = ui.container()
        .id(ai_id)
        .width(ai_panel_w)
        .height(height)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .column()
        .show(|ui| {
            // ─── Header (Responsive & SpaceBetween) ───────────────────
            const HEADER_H: f32 = 36.0;
            ui.container()
                .width(ai_panel_w)
                .height(HEADER_H)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding_left(10.0)
                .padding_right(8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .no_wrap()
                .show(|ui| {
                    // Child 1: Left Icon, Title & Model Badge
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .show(|ui| {
                            // Sparkle glyph
                            ui.text("\u{f005}")
                                .size(11.5)
                                .color(colors.accent)
                                .show();

                            if ai_panel_w >= 320.0 {
                                ui.text("AI Assistant")
                                    .size(11.5)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            } else if ai_panel_w >= 240.0 {
                                ui.text("Assistant")
                                    .size(11.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            } else {
                                ui.text("AI")
                                    .size(11.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            }

                            // Responsive model badge
                            if ai_panel_w >= 340.0 {
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

                                        ui.text("Claude 3.7")
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                            } else if ai_panel_w >= 270.0 {
                                ui.container()
                                    .padding(1.0, 4.0, 1.0, 4.0)
                                    .bg(colors.bg_base)
                                    .border(colors.border, 1.0)
                                    .radius_all(3.0)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(3.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(5.0)
                                            .height(5.0)
                                            .radius_all(2.5)
                                            .bg(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                                            .show(|_| {});

                                        ui.text("3.7")
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                            }
                        });

                    // Child 2: Right action icons (Reset + Close) securely pinned to right edge
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            // Reset conversation button
                            let reset_btn = ui.container()
                                .width(18.0)
                                .height(18.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f01e}")
                                        .size(9.5)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if reset_btn.clicked {
                                state.ai_messages.clear();
                                state.ai_messages.push(crate::state::AiMessage {
                                    is_user: false,
                                    content: "Conversation cleared. What would you like to work on next?".to_string(),
                                    code_snippet: None,
                                    thoughts: None,
                                });
                                ui.needs_redraw = true;
                            }

                            // Close panel button
                            let close_btn = ui.container()
                                .width(18.0)
                                .height(18.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f00d}")
                                        .size(9.5)
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

            // ─── Active Context Banner (Responsive & SpaceBetween) ──
            let active_filename = state.active_tab
                .and_then(|idx| state.tabs.get(idx))
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "No active file".to_string());

            let max_name_len = if ai_panel_w >= 360.0 {
                22
            } else if ai_panel_w >= 280.0 {
                14
            } else {
                8
            };

            let display_name = if active_filename.len() > max_name_len && max_name_len > 4 {
                let ext_pos = active_filename.rfind('.');
                if let Some(pos) = ext_pos {
                    let ext = &active_filename[pos..];
                    let prefix_len = (max_name_len.saturating_sub(ext.len() + 2)).max(2);
                    format!("{}..{}", &active_filename[..prefix_len.min(active_filename.len())], ext)
                } else {
                    format!("{}...", &active_filename[..max_name_len - 3])
                }
            } else {
                active_filename.clone()
            };

            let context_label = if ai_panel_w >= 290.0 {
                format!("Context: {}", display_name)
            } else {
                display_name
            };

            ui.container()
                .width(ai_panel_w)
                .padding(5.0, 8.0, 5.0, 8.0)
                .bg(colors.bg_base)
                .border(colors.border, 1.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .no_wrap()
                .show(|ui| {
                    // Child 1: Left Context Info
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .show(|ui| {
                            ui.text("\u{f15c}")
                                .size(10.0)
                                .color(colors.accent)
                                .show();

                            ui.text(&context_label)
                                .size(10.0)
                                .weight(FontWeight::Medium)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Child 2: Right Status Info
                    if ai_panel_w >= 330.0 {
                        ui.text("AST Synced")
                            .size(9.0)
                            .color(colors.text_muted)
                            .show();
                    } else if ai_panel_w >= 250.0 {
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(3.0)
                            .show(|ui| {
                                ui.container()
                                    .width(4.0)
                                    .height(4.0)
                                    .radius_all(2.0)
                                    .bg(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                                    .show(|_| {});
                                ui.text("Sync")
                                    .size(8.5)
                                    .color(colors.text_muted)
                                    .show();
                            });
                    } else {
                        ui.container()
                            .width(5.0)
                            .height(5.0)
                            .radius_all(2.5)
                            .bg(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                            .show(|_| {});
                    }
                });

            // ─── Bottom Prompt Input Height & Messages Scroll Area ─────
            let ai_input_id = ui.id_from("ai_prompt_text_area");
            let is_input_focused = ui.focused_id == Some(ai_input_id);

            // Dynamic prompt box height: min 36.0 (~1 line), max 240.0 (~14 lines), then scrolls
            const MIN_PROMPT_H: f32 = 36.0;
            const MAX_PROMPT_H: f32 = 240.0;
            let prompt_box_h = if state.ai_input.is_empty() {
                MIN_PROMPT_H
            } else if let Some((rect, _)) = ui.get_recorded_layout(ai_input_id) {
                rect.size.height.clamp(MIN_PROMPT_H, MAX_PROMPT_H)
            } else {
                let line_count = state.ai_input.split('\n').count();
                (line_count as f32 * 16.0 + 14.0).clamp(MIN_PROMPT_H, MAX_PROMPT_H)
            };

            const CONTEXT_BANNER_H: f32 = 28.0;
            const CHIPS_H: f32 = 26.0;
            let bottom_input_total_h = prompt_box_h + 16.0;
            let msgs_h = (height - HEADER_H - CONTEXT_BANNER_H - CHIPS_H - bottom_input_total_h).max(60.0);
            let max_content_w = (ai_panel_w - 24.0).max(80.0);

            let msgs_scroll_id = Id::from_u64(999999802);
            ui.container()
                .id(msgs_scroll_id)
                .width(ai_panel_w)
                .height(msgs_h)
                .padding(8.0, 8.0, 8.0, 8.0)
                .column()
                .gap(10.0)
                .scroll_y(true)
                .show(|ui| {
                    let msgs = state.ai_messages.clone();
                    for msg in msgs {
                        if msg.is_user {
                            // User Message Bubble
                            let bubble_max_w = (ai_panel_w - 36.0).max(80.0);
                            let user_text_w = (bubble_max_w - 20.0).max(60.0);
                            ui.container()
                                .full_width()
                                .row()
                                .halign(Align::Right)
                                .show(|ui| {
                                    ui.container()
                                        .max_width(bubble_max_w)
                                        .padding(6.0, 10.0, 6.0, 10.0)
                                        .bg(match state.theme {
                                            crate::theme::ThemeMode::Dark => Color::rgba(99.0 / 255.0, 208.0 / 255.0, 223.0 / 255.0, 0.15),
                                            crate::theme::ThemeMode::Light => Color::rgba(7.0 / 255.0, 102.0 / 255.0, 120.0 / 255.0, 0.12),
                                        })
                                        .border(colors.accent, 1.0)
                                        .radius_all(6.0)
                                        .column()
                                        .gap(2.0)
                                        .show(|ui| {
                                            ui.text("You")
                                                .size(9.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                            ui.text(&msg.content)
                                                .size(11.0)
                                                .line_height(1.3)
                                                .color(colors.text_primary)
                                                .max_width(user_text_w)
                                                .wrap(zenthra_text::prelude::TextWrap::Word)
                                                .show();
                                        });
                                });
                        } else {
                            // Assistant Message Bubble
                            let assistant_text_w = (max_content_w - 16.0).max(60.0);
                            ui.container()
                                .full_width()
                                .padding(8.0, 8.0, 8.0, 8.0)
                                .bg(colors.bg_base)
                                .border(colors.border, 1.0)
                                .radius_all(6.0)
                                .column()
                                .gap(6.0)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(5.0)
                                        .show(|ui| {
                                            ui.text("\u{f005}")
                                                .size(9.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.text("Zenthra Assistant")
                                                .size(9.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });

                                    // Thoughts reasoning block
                                    if let Some(ref thoughts) = msg.thoughts {
                                        ui.container()
                                            .full_width()
                                            .padding(4.0, 6.0, 4.0, 6.0)
                                            .bg(colors.bg_sidebar)
                                            .border(colors.border, 1.0)
                                            .radius_all(3.0)
                                            .show(|ui| {
                                                ui.text(&format!("🧠 {}", thoughts))
                                                    .size(9.5)
                                                    .line_height(1.25)
                                                    .color(colors.text_muted)
                                                    .max_width((max_content_w - 24.0).max(50.0))
                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                    .show();
                                            });
                                    }

                                    // Content text with full word and character wrapping
                                    ui.text(&msg.content)
                                        .size(11.0)
                                        .line_height(1.3)
                                        .color(colors.text_primary)
                                        .max_width(assistant_text_w)
                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                        .show();

                                    // Code snippet block
                                    if let Some(ref code) = msg.code_snippet {
                                        ui.container()
                                            .full_width()
                                            .padding(6.0, 8.0, 6.0, 8.0)
                                            .bg(match state.theme {
                                                crate::theme::ThemeMode::Dark => Color::rgb(29.0 / 255.0, 32.0 / 255.0, 33.0 / 255.0),
                                                crate::theme::ThemeMode::Light => Color::rgb(235.0 / 255.0, 230.0 / 255.0, 220.0 / 255.0),
                                            })
                                            .border(colors.border, 1.0)
                                            .radius_all(4.0)
                                            .column()
                                            .gap(4.0)
                                            .show(|ui| {
                                                ui.text(code)
                                                    .size(10.5)
                                                    .line_height(1.25)
                                                    .color(colors.text_primary)
                                                    .max_width((max_content_w - 24.0).max(50.0))
                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                    .show();

                                                ui.container()
                                                    .row()
                                                    .gap(6.0)
                                                    .show(|ui| {
                                                        let run_label = if ai_panel_w >= 300.0 {
                                                            "Run in Terminal"
                                                        } else {
                                                            "Run >_"
                                                        };

                                                        let run_term_btn = ui.container()
                                                            .padding(2.0, 6.0, 2.0, 6.0)
                                                            .bg(colors.bg_base)
                                                            .hover_bg(colors.highlight)
                                                            .border(colors.border, 1.0)
                                                            .radius_all(3.0)
                                                            .show(|ui| {
                                                                ui.text(run_label)
                                                                    .size(9.0)
                                                                    .color(colors.text_muted)
                                                                    .show();
                                                            });

                                                        if run_term_btn.clicked {
                                                            state.send_terminal_input(code);
                                                            ui.needs_redraw = true;
                                                        }

                                                        let copy_btn = ui.container()
                                                            .padding(2.0, 6.0, 2.0, 6.0)
                                                            .bg(colors.bg_base)
                                                            .hover_bg(colors.highlight)
                                                            .border(colors.border, 1.0)
                                                            .radius_all(3.0)
                                                            .show(|ui| {
                                                                ui.text("Copy")
                                                                    .size(9.0)
                                                                    .color(colors.text_muted)
                                                                    .show();
                                                            });

                                                        if copy_btn.clicked {
                                                            state.set_clipboard(code.clone());
                                                            state.status_message = "Snippet copied to clipboard".to_string();
                                                            ui.needs_redraw = true;
                                                        }
                                                    });
                                            });
                                    }
                                });
                        }
                    }
                });

            // ─── Quick Prompt Chips ──────────────────────────────────
            let prompt_chips: &[&str] = if ai_panel_w >= 340.0 {
                &["Explain Code", "Refactor", "Generate Tests"]
            } else if ai_panel_w >= 280.0 {
                &["Explain", "Refactor"]
            } else {
                &["Explain"]
            };

            ui.container()
                .width(ai_panel_w)
                .height(CHIPS_H)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding_left(8.0)
                .padding_right(8.0)
                .row()
                .valign(Align::Center)
                .gap(4.0)
                .show(|ui| {
                    for prompt in prompt_chips {
                        let btn = ui.container()
                            .padding(2.0, 5.0, 2.0, 5.0)
                            .bg(colors.bg_base)
                            .hover_bg(colors.highlight)
                            .border(colors.border, 1.0)
                            .radius_all(3.0)
                            .show(|ui| {
                                ui.text(*prompt)
                                    .size(9.0)
                                    .color(colors.text_muted)
                                    .show();
                            });

                        if btn.clicked {
                            state.send_ai_message(prompt);
                            ui.needs_redraw = true;
                        }
                    }
                });

            // ─── Bottom Prompt Input (Auto-growing TextArea & Send Button) ─────
            ui.container()
                .width(ai_panel_w)
                .height(prompt_box_h + 16.0)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding(6.0, 8.0, 6.0, 8.0)
                .column()
                .gap(4.0)
                .show(|ui| {
                    // Enter key handling: Enter sends message, Shift+Enter inserts newline
                    let mut trigger_send = false;
                    if is_input_focused {
                        ui.input_events.retain(|event| {
                            match event {
                                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                                    if *key == winit::keyboard::KeyCode::Enter || *key == winit::keyboard::KeyCode::NumpadEnter {
                                        if !state.shift_pressed {
                                            trigger_send = true;
                                            return false; // Consume Enter so text_area doesn't add trailing newline
                                        }
                                    } else if *key == winit::keyboard::KeyCode::Escape {
                                        ui.focused_id = None;
                                        ui.needs_redraw = true;
                                        return false;
                                    }
                                }
                                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                                    if (*c == '\r' || *c == '\n') && !state.shift_pressed {
                                        return false; // Consume newline character
                                    }
                                }
                                _ => {}
                            }
                            true
                        });
                    }

                    ui.container()
                        .full_width()
                        .row()
                        .valign(Align::Bottom)
                        .halign(Align::SpaceBetween)
                        .no_wrap()
                        .gap(6.0)
                        .show(|ui| {
                            let send_btn_w: f32 = 28.0;
                            let avail_input_w = (ai_panel_w - 16.0 - 2.0 - send_btn_w - 6.0).max(40.0);

                            let placeholder = if ai_panel_w >= 340.0 {
                                "Ask AI assistant... (Shift+Enter for newline)"
                            } else if ai_panel_w >= 260.0 {
                                "Ask AI assistant..."
                            } else {
                                "Ask..."
                            };

                            // Child 1: Native Zenthra framework TextArea widget (auto-grows up to 14 lines, then scrolls)
                            let text_res = ui.text_area(&mut state.ai_input, "ai_prompt_text_area")
                                .width(avail_input_w)
                                .height(MIN_PROMPT_H)
                                .max_height(MAX_PROMPT_H)
                                .size(11.5)
                                .line_height(1.25)
                                .color(colors.text_primary)
                                .bg(colors.bg_base)
                                .border(colors.border, 1.0)
                                .focus_border(colors.accent, 1.0)
                                .radius_all(4.0)
                                .padding(5.0, 7.0, 5.0, 7.0)
                                .placeholder(placeholder)
                                .placeholder_color(colors.text_muted.with_alpha(0.5))
                                .wrap(zenthra_text::prelude::TextWrap::Word)
                                .submit_on_enter(true)
                                .show();

                            // Child 2: Send Button (pinned to bottom-right)
                            let send_btn = ui.button("➤")
                                .id("ai_send_button")
                                .width(send_btn_w)
                                .height(MIN_PROMPT_H)
                                .bg(colors.accent)
                                .hover_bg(colors.highlight)
                                .text_color(Color::WHITE)
                                .size(10.0)
                                .radius_all(4.0)
                                .padding(0.0, 0.0, 0.0, 0.0)
                                .align(zenthra_core::Align::Center)
                                .show();

                            if trigger_send || text_res.submitted || send_btn.clicked {
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
        });

    if panel_container.clicked && ui.focused_id != Some(ai_input_id) {
        ui.focused_id = Some(ai_id);
        ui.needs_redraw = true;
    }
}
