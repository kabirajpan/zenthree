use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id};

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = state.colors();
    let ai_panel_w = width;
    let ai_id = Id::from_u64(999999801);
    let is_focused = ui.focused_id == Some(ai_id);

    // Left vertical splitter handle for resizing panel width
    let handle_w = 6.0;
    let splitter_x = ui.cursor_x;
    let is_splitter_hovered = ui.mouse_in_rect(
        splitter_x - handle_w / 2.0,
        ui.cursor_y,
        handle_w,
        height,
    );

    if is_splitter_hovered && ui.mouse_down {
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

    // Input processing when AI input is focused
    if is_focused {
        let events = std::mem::take(&mut ui.input_events);
        let mut trigger_send = false;

        for event in &events {
            match event {
                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                    if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                        state.ai_input.push(*c);
                        ui.needs_redraw = true;
                    }
                }
                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                    match key {
                        winit::keyboard::KeyCode::Backspace => {
                            state.ai_input.pop();
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
            let msg = state.ai_input.clone();
            state.send_ai_message(&msg);
            ui.needs_redraw = true;
        }
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

            // ─── Messages Scroll Area ────────────────────────────────
            const INPUT_AREA_H: f32 = 82.0;
            let msgs_h = (height - 36.0 - 28.0 - INPUT_AREA_H).max(60.0);
            let max_content_w = (ai_panel_w - 24.0).max(80.0);

            ui.container()
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
                                                .color(colors.text_primary)
                                                .max_width(bubble_max_w - 20.0)
                                                .show();
                                        });
                                });
                        } else {
                            // Assistant Message Bubble
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
                                                    .color(colors.text_muted)
                                                    .max_width(max_content_w - 24.0)
                                                    .show();
                                            });
                                    }

                                    // Content text
                                    ui.text(&msg.content)
                                        .size(11.0)
                                        .color(colors.text_primary)
                                        .max_width(max_content_w - 16.0)
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
                                                    .color(colors.text_primary)
                                                    .max_width(max_content_w - 24.0)
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
                .height(26.0)
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

            // ─── Bottom Prompt Input (Responsive & SpaceBetween) ─────
            ui.container()
                .width(ai_panel_w)
                .height(56.0)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding(6.0, 8.0, 6.0, 8.0)
                .column()
                .gap(4.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .no_wrap()
                        .show(|ui| {
                            let send_btn_w: f32 = 24.0;
                            let avail_input_w = (ai_panel_w - 16.0 - 2.0 - send_btn_w - 6.0).max(40.0);

                            let placeholder = if ai_panel_w >= 320.0 {
                                "Ask AI assistant anything..."
                            } else if ai_panel_w >= 240.0 {
                                "Ask assistant..."
                            } else {
                                "Ask..."
                            };

                            let input_text = if state.ai_input.is_empty() {
                                if is_focused { "|" } else { placeholder }
                            } else {
                                &state.ai_input
                            };

                            let text_col = if state.ai_input.is_empty() && !is_focused {
                                colors.text_muted.with_alpha(0.5)
                            } else {
                                colors.text_primary
                            };

                            let max_ai_chars = ((avail_input_w - 16.0) / 7.2).max(6.0) as usize;
                            let display = if is_focused && !state.ai_input.is_empty() {
                                let full = format!("{}|", input_text);
                                if full.len() > max_ai_chars {
                                    format!("...{}", &full[full.len().saturating_sub(max_ai_chars - 3)..])
                                } else {
                                    full
                                }
                            } else if input_text.len() > max_ai_chars {
                                format!("...{}", &input_text[input_text.len().saturating_sub(max_ai_chars - 3)..])
                            } else {
                                input_text.to_string()
                            };

                            // Child 1: Input text box (explicit width, never pushes siblings off)
                            let box_resp = ui.container()
                                .width(avail_input_w)
                                .padding(4.0, 6.0, 4.0, 6.0)
                                .bg(colors.bg_base)
                                .border(if is_focused { colors.accent } else { colors.border }, 1.0)
                                .radius_all(4.0)
                                .row()
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text(&display)
                                        .size(11.0)
                                        .color(text_col)
                                        .show();
                                });

                            if box_resp.clicked {
                                ui.focused_id = Some(ai_id);
                                ui.needs_redraw = true;
                            }

                            // Child 2: Send button (pinned cleanly at the right edge)
                            let send_btn = ui.container()
                                .width(send_btn_w)
                                .height(24.0)
                                .bg(colors.accent)
                                .hover_bg(colors.highlight)
                                .radius_all(4.0)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("➤")
                                        .size(9.0)
                                        .color(Color::WHITE)
                                        .show();
                                });

                            if send_btn.clicked {
                                let msg = state.ai_input.clone();
                                state.send_ai_message(&msg);
                                ui.needs_redraw = true;
                            }
                        });
                });
        });

    if panel_container.clicked {
        ui.focused_id = Some(ai_id);
        ui.needs_redraw = true;
    }
}
