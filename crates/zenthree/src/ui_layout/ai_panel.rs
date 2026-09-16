use crate::state::{ZenthreeState, DiffLineState, EditedFileSummary, TaskStatus, PendingModification, AiInputDrawer};
use crate::session::{format_relative_time, format_bytes};
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    if state.poll_ai_responses() {
        ui.request_redraw();
    }
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
    const SUB_TOOLBAR_H: f32 = 22.0;
    let bottom_input_total_h = prompt_box_h + 46.0 + SUB_TOOLBAR_H;

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
                            // Left arrow to return to session list (only if saved sessions exist)
                            if !state.ai_showing_session_list && !state.session_list.is_empty() {
                                let back_btn = ui.container()
                                    .padding(2.0, 4.0, 2.0, 4.0)
                                    .radius_all(3.0)
                                    .hover_bg(colors.highlight)
                                    .row()
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text("\u{f053}")
                                            .size(9.5)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();
                                    });
                                if back_btn.clicked {
                                    state.session_list = crate::session::list_saved_sessions(&state.current_dir);
                                    state.ai_showing_session_list = true;
                                    ui.needs_redraw = true;
                                }
                            }

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

                            let (provider_label, provider_color) = if state.gemini_api_key.is_some() {
                                ("Gemini Flash", Color::rgb(0.28, 0.58, 0.98))
                            } else if state.groq_api_key.is_some() {
                                ("Groq", Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                            } else {
                                ("Offline", colors.text_muted)
                            };

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
                                            .bg(provider_color)
                                            .show(|_| {});

                                        ui.text(provider_label)
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
                                            .bg(provider_color)
                                            .show(|_| {});

                                        let short_label = if state.gemini_api_key.is_some() { "Gemini" } else { provider_label };
                                        ui.text(short_label)
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                            }
                        });

                    // Child 2: Right action icons (Cockpit + Reset + Close) securely pinned to right edge
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            // + New Agent Session Button
                            let new_session_btn = ui.container()
                                .height(18.0)
                                .padding(1.0, 5.0, 1.0, 5.0)
                                .radius_all(3.0)
                                .bg(colors.bg_base)
                                .border(colors.border, 1.0)
                                .hover_bg(colors.highlight)
                                .row()
                                .gap(2.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("+")
                                        .size(10.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.accent)
                                        .show();
                                    if ai_panel_w >= 310.0 {
                                        ui.text("New")
                                            .size(8.5)
                                            .weight(FontWeight::SemiBold)
                                            .color(colors.text_muted)
                                            .show();
                                    }
                                });

                            if new_session_btn.clicked {
                                state.start_new_session();
                                ui.needs_redraw = true;
                            }

                            // In Silico Connectome Cockpit Toggle Button
                            let cockpit_active = state.connectome_cockpit_open;
                            let cockpit_btn = ui.container()
                                .height(18.0)
                                .padding(1.0, 5.0, 1.0, 5.0)
                                .radius_all(3.0)
                                .bg(if cockpit_active { colors.accent.with_alpha(0.18) } else { colors.bg_base })
                                .border(if cockpit_active { colors.accent } else { colors.border }, 1.0)
                                .hover_bg(colors.highlight)
                                .row()
                                .gap(3.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f0e7}")
                                        .size(9.0)
                                        .color(if cockpit_active { colors.accent } else { colors.text_muted })
                                        .show();
                                    if ai_panel_w >= 290.0 {
                                        ui.text("Brain")
                                            .size(8.5)
                                            .weight(FontWeight::Bold)
                                            .color(if cockpit_active { colors.accent } else { colors.text_muted })
                                            .show();
                                    }
                                });

                            if cockpit_btn.clicked {
                                state.connectome_cockpit_open = !state.connectome_cockpit_open;
                                ui.needs_redraw = true;
                            }

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
                                state.save_current_session_messages();
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

            // ─── Default Session List View ──────────────────────────────
            if state.ai_showing_session_list && !state.session_list.is_empty() {
                let session_list_h = (height - HEADER_H).max(100.0);
                let session_scroll_id = Id::from_u64(999999805);

                ui.container()
                    .id(session_scroll_id)
                    .width(ai_panel_w)
                    .height(session_list_h)
                    .padding(10.0, 10.0, 10.0, 10.0)
                    .column()
                    .gap(10.0)
                    .scroll_y(true)
                    .show(|ui| {
                        let card_w = (ai_panel_w - 20.0).max(120.0);
                        let card_inner_w = (card_w - 20.0).max(80.0);

                        // Top "+ Start New Session" Action Card
                        let mut do_start_new = false;
                        let start_btn_w = 52.0;
                        let start_title_w = (card_inner_w - 30.0 - start_btn_w - 6.0).max(40.0);

                        let start_new_btn = ui.container()
                            .id(Id::from_u64(999999801))
                            .width(card_w)
                            .clip(true)
                            .bg(colors.bg_sidebar)
                            .border(colors.border, 1.0)
                            .hover_bg(colors.highlight)
                            .radius_all(6.0)
                            .padding(8.0, 10.0, 8.0, 10.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container()
                                    .width(30.0 + start_title_w)
                                    .row()
                                    .gap(8.0)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.container()
                                            .width(22.0)
                                            .height(22.0)
                                            .radius_all(4.0)
                                            .bg(colors.accent.with_alpha(0.18))
                                            .valign(Align::Center)
                                            .halign(Align::Center)
                                            .show(|ui| {
                                                ui.text("+")
                                                    .size(14.0)
                                                    .weight(FontWeight::Bold)
                                                    .color(colors.accent)
                                                    .show();
                                            });

                                        ui.container()
                                            .width(start_title_w)
                                            .clip(true)
                                            .column()
                                            .gap(2.0)
                                            .show(|ui| {
                                                ui.text("Start New Session")
                                                    .size(11.0)
                                                    .weight(FontWeight::SemiBold)
                                                    .color(colors.text_primary)
                                                    .max_width(start_title_w)
                                                    .ellipsis(true)
                                                    .show();
                                                ui.text("Begin fresh agent conversation")
                                                    .size(9.0)
                                                    .color(colors.text_muted)
                                                    .max_width(start_title_w)
                                                    .ellipsis(true)
                                                    .show();
                                            });
                                    });

                                let new_btn = ui.button("+ New")
                                    .id(Id::from_u64(999998799))
                                    .width(start_btn_w)
                                    .height(20.0)
                                    .bg(colors.accent)
                                    .hover_bg(colors.accent.with_alpha(0.85))
                                    .text_color(Color::WHITE)
                                    .radius_all(3.0)
                                    .padding(0.0, 0.0, 0.0, 0.0)
                                    .size(10.0)
                                    .show();

                                if new_btn.clicked {
                                    do_start_new = true;
                                }
                            });

                        if start_new_btn.clicked || do_start_new {
                            state.start_new_session();
                            ui.needs_redraw = true;
                        }

                        // Section Heading: Saved Sessions
                        ui.container()
                            .width(card_w)
                            .padding_top(6.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.text("Saved Sessions")
                                    .size(11.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                                let count_label = format!("{} session{}", state.session_list.len(), if state.session_list.len() == 1 { "" } else { "s" });
                                ui.text(&count_label)
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            });

                        if state.session_list.is_empty() {
                            ui.container()
                                .width(card_w)
                                .height(80.0)
                                .radius_all(6.0)
                                .bg(colors.bg_base)
                                .border(colors.border, 1.0)
                                .padding(12.0, 12.0, 12.0, 12.0)
                                .column()
                                .valign(Align::Center)
                                .halign(Align::Center)
                                .gap(4.0)
                                .show(|ui| {
                                    ui.text("No previous sessions found in workspace")
                                        .size(10.5)
                                        .color(colors.text_muted)
                                        .show();
                                    ui.text("Click above to start your first session")
                                        .size(9.5)
                                        .color(colors.accent)
                                        .show();
                                });
                        } else {
                            let mut selected_session = None;
                            let mut to_delete_session = None;
                            let right_w = 28.0;
                            let title_col_w = (card_inner_w - 30.0 - right_w - 8.0).max(40.0);

                            for (idx, sess) in state.session_list.iter().enumerate() {
                                let is_current = sess.id == state.current_session.id;
                                let mut del_clicked = false;
                                let sess_card = ui.container()
                                    .id(Id::from_u64(999998800 + idx as u64))
                                    .width(card_w)
                                    .clip(true)
                                    .bg(if is_current { colors.accent.with_alpha(0.08) } else { colors.bg_sidebar })
                                    .border(if is_current { colors.accent.with_alpha(0.6) } else { colors.border }, 1.0)
                                    .hover_bg(colors.highlight)
                                    .radius_all(6.0)
                                    .padding(8.0, 10.0, 8.0, 10.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.container()
                                            .width(30.0 + title_col_w)
                                            .row()
                                            .gap(8.0)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                // Icon glyph box matching extensions_panel.rs
                                                ui.container()
                                                    .width(22.0)
                                                    .height(22.0)
                                                    .bg(colors.accent.with_alpha(0.18))
                                                    .radius_all(4.0)
                                                    .valign(Align::Center)
                                                    .halign(Align::Center)
                                                    .show(|ui| {
                                                        ui.text("\u{ea7f}")
                                                            .size(12.0)
                                                            .color(colors.accent)
                                                            .show();
                                                    });

                                                ui.container()
                                                    .width(title_col_w)
                                                    .clip(true)
                                                    .column()
                                                    .gap(2.0)
                                                    .show(|ui| {
                                                        ui.text(&sess.title)
                                                            .size(11.0)
                                                            .weight(FontWeight::SemiBold)
                                                            .color(colors.text_primary)
                                                            .max_width(title_col_w)
                                                            .ellipsis(true)
                                                            .show();

                                                        let time_str = format_relative_time(sess.updated_at);
                                                        let meta_text = if sess.message_count > 0 {
                                                            format!("{} • {} msg{}", time_str, sess.message_count, if sess.message_count == 1 { "" } else { "s" })
                                                        } else {
                                                            time_str
                                                        };
                                                        ui.text(&meta_text)
                                                            .size(9.0)
                                                            .color(colors.text_muted)
                                                            .max_width(title_col_w)
                                                            .ellipsis(true)
                                                            .show();
                                                    });
                                            });

                                        // Right: trash delete icon button
                                        let del_btn = ui.container()
                                            .id(Id::from_u64(999998900 + idx as u64))
                                            .width(22.0)
                                            .height(22.0)
                                            .radius_all(3.0)
                                            .hover_bg(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0).with_alpha(0.18))
                                            .halign(Align::Center)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text("\u{f1f8}")
                                                    .size(11.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });

                                        if del_btn.clicked {
                                            del_clicked = true;
                                            to_delete_session = Some(sess.id.clone());
                                        }
                                    });

                                if sess_card.clicked && to_delete_session.is_none() && !del_clicked {
                                    selected_session = Some(sess.id.clone());
                                }
                            }

                            if let Some(id) = to_delete_session {
                                state.delete_session(&id);
                                ui.needs_redraw = true;
                            } else if let Some(id) = selected_session {
                                state.switch_session(&id);
                                ui.needs_redraw = true;
                            }
                        }
                    });

                return;
            }

            // ─── Active Context Banner (Responsive & SpaceBetween) ──
            let active_filename = state.active_tab
                .and_then(|idx| state.tabs.get(idx))
                .map(|t| t.name.clone())
                .unwrap_or_else(|| "No active file".to_string());

            let context_label = format!("Context: {}", active_filename);

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

            // ─── In Silico Connectome Cockpit & Biological Lab ───────────
            let cockpit_drawer_h = if state.connectome_cockpit_open {
                if state.connectome_ablation_enabled { 108.0 } else { 58.0 }
            } else {
                0.0
            };

            if state.connectome_cockpit_open {
                ui.container()
                    .full_width()
                    .padding(6.0, 8.0, 6.0, 8.0)
                    .bg(match state.theme {
                        crate::theme::ThemeMode::Dark => Color::rgb(18.0 / 255.0, 21.0 / 255.0, 26.0 / 255.0),
                        crate::theme::ThemeMode::Light => Color::rgb(238.0 / 255.0, 242.0 / 255.0, 246.0 / 255.0),
                    })
                    .border(colors.accent.with_alpha(0.3), 1.0)
                    .column()
                    .gap(4.0)
                    .show(|ui| {
                        // Title row
                        ui.container()
                            .full_width()
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container()
                                    .row()
                                    .gap(5.0)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text("\u{f0e7}")
                                            .size(9.0)
                                            .color(colors.accent)
                                            .show();
                                        ui.text("Connectome Cockpit")
                                            .size(9.0)
                                            .weight(FontWeight::Bold)
                                            .color(colors.accent)
                                            .show();
                                    });

                                ui.text("500 neurons • 5,399 synapses")
                                    .size(8.0)
                                    .color(colors.text_muted)
                                    .show();
                            });

                        // Lesion toggle switch
                        ui.container()
                            .full_width()
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.text("In Silico Lesion:")
                                    .size(8.5)
                                    .weight(FontWeight::Medium)
                                    .color(colors.text_primary)
                                    .show();

                                let is_on = state.connectome_ablation_enabled;
                                let toggle_btn = ui.container()
                                    .padding(2.0, 6.0, 2.0, 6.0)
                                    .radius_all(3.0)
                                    .bg(if is_on {
                                        Color::rgb(180.0 / 255.0, 50.0 / 255.0, 40.0 / 255.0).with_alpha(0.25)
                                    } else {
                                        colors.bg_base
                                    })
                                    .border(if is_on {
                                        Color::rgb(220.0 / 255.0, 70.0 / 255.0, 50.0 / 255.0)
                                    } else {
                                        colors.border
                                    }, 1.0)
                                    .hover_bg(colors.highlight)
                                    .show(|ui| {
                                        ui.text(if is_on { "\u{f0e7} ABLATION ON" } else { "DISABLED (INTACT)" })
                                            .size(8.0)
                                            .weight(FontWeight::Bold)
                                            .color(if is_on {
                                                Color::rgb(240.0 / 255.0, 100.0 / 255.0, 80.0 / 255.0)
                                            } else {
                                                colors.text_muted
                                            })
                                            .show();
                                    });

                                if toggle_btn.clicked {
                                    state.connectome_ablation_enabled = !state.connectome_ablation_enabled;
                                    ui.needs_redraw = true;
                                }
                            });

                        // Ablation mode & fraction controls if ablation enabled
                        if state.connectome_ablation_enabled {
                            // Mode pills
                            ui.container()
                                .full_width()
                                .row()
                                .gap(4.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let modes = [
                                        (connectome::AblationMode::TargetedHubs, "Hubs"),
                                        (connectome::AblationMode::Random, "Random"),
                                        (connectome::AblationMode::InhibitoryOnly, "GABA-"),
                                        (connectome::AblationMode::ExcitatoryOnly, "ACh+"),
                                    ];

                                    for (mode, label) in modes {
                                        let selected = state.connectome_ablation_mode == mode;
                                        let pill = ui.container()
                                            .padding(1.5, 5.0, 1.5, 5.0)
                                            .radius_all(2.5)
                                            .bg(if selected { colors.accent.with_alpha(0.25) } else { colors.bg_base })
                                            .border(if selected { colors.accent } else { colors.border }, 1.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text(label)
                                                    .size(8.0)
                                                    .weight(if selected { FontWeight::Bold } else { FontWeight::Regular })
                                                    .color(if selected { colors.accent } else { colors.text_muted })
                                                    .show();
                                            });

                                        if pill.clicked {
                                            state.connectome_ablation_mode = mode;
                                            ui.needs_redraw = true;
                                        }
                                    }
                                });

                            // Fraction selector
                            ui.container()
                                .full_width()
                                .row()
                                .gap(4.0)
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .gap(3.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            let fractions = [0.10f32, 0.20f32, 0.30f32, 0.50f32];
                                            for frac in fractions {
                                                let selected = (state.connectome_ablation_pct - frac).abs() < 0.01;
                                                let frac_btn = ui.container()
                                                    .padding(1.0, 4.0, 1.0, 4.0)
                                                    .radius_all(2.5)
                                                    .bg(if selected { colors.accent.with_alpha(0.2) } else { colors.bg_base })
                                                    .border(if selected { colors.accent } else { colors.border }, 1.0)
                                                    .hover_bg(colors.highlight)
                                                    .show(|ui| {
                                                        ui.text(&format!("{:.0}%", frac * 100.0))
                                                            .size(8.0)
                                                            .weight(if selected { FontWeight::Bold } else { FontWeight::Regular })
                                                            .color(if selected { colors.accent } else { colors.text_muted })
                                                            .show();
                                                    });

                                                if frac_btn.clicked {
                                                    state.connectome_ablation_pct = frac;
                                                    ui.needs_redraw = true;
                                                }
                                            }
                                        });

                                    let silenced_est = (state.connectome_ablation_pct * 460.0).round() as usize;
                                    ui.text(&format!("~{} silenced", silenced_est))
                                        .size(8.0)
                                        .color(Color::rgb(220.0 / 255.0, 100.0 / 255.0, 80.0 / 255.0))
                                        .show();
                                });
                        }
                    });
            }

            // ─── Bottom Prompt Input Height & Messages Scroll Area ─────
            let ai_input_id = ui.id_from("ai_prompt_text_area");
            let is_input_focused = ui.focused_id == Some(ai_input_id);

            const CONTEXT_BANNER_H: f32 = 28.0;
            const TOOLBAR_H: f32 = 28.0;

            let drawer_h = match state.active_input_drawer {
                AiInputDrawer::None => 0.0,
                AiInputDrawer::PendingModifications => {
                    let count = state.pending_modifications.len().max(if state.pending_modification.is_some() { 1 } else { 0 });
                    if count > 0 {
                        let file_list_h = (count as f32 * 26.0 + 8.0).min(130.0);
                        file_list_h + 1.0 + 34.0
                    } else {
                        38.0
                    }
                }
                AiInputDrawer::Terminals => 72.0,
                AiInputDrawer::Artifacts => {
                    let arts = state.current_session.list_artifacts();
                    (arts.len() as f32 * 26.0 + 38.0).clamp(44.0, 150.0)
                }
                AiInputDrawer::SessionChanges => {
                    let count = state.session_modified_files.len();
                    if count > 0 {
                        (count as f32 * 26.0 + 38.0).clamp(44.0, 160.0)
                    } else {
                        44.0
                    }
                }
            };

            let quota_banner_h = if state.last_provider_notice.is_some() { 26.0 } else { 20.0 };
            let msgs_h = (height - HEADER_H - CONTEXT_BANNER_H - bottom_input_total_h - cockpit_drawer_h - TOOLBAR_H - quota_banner_h - drawer_h - 12.0).max(60.0);
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
                    if msgs.is_empty() {
                        // Centered ZENE Agent Intro Screen (Bold Moderniz typography)
                        ui.container()
                            .full_width()
                            .column()
                            .halign(Align::Center)
                            .valign(Align::Center)
                            .padding(36.0, 16.0, 24.0, 16.0)
                            .gap(12.0)
                            .show(|ui| {
                                // Bold ZENE title in Moderniz font acting as brand title & icon
                                ui.text("ZENE")
                                    .family("MODERNIZ")
                                    .bold()
                                    .size(28.0)
                                    .color(colors.text_primary)
                                    .show();

                                // Option 1 Subtext
                                let subtext_w = (ai_panel_w - 48.0).max(120.0);
                                ui.text("Start working with ZENE to build, edit, and explore your codebase.")
                                    .size(11.5)
                                    .line_height(1.4)
                                    .color(colors.text_muted)
                                    .max_width(subtext_w)
                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                    .show();

                                ui.spacing(10.0);

                                // Quick starter suggestion buttons (no icons)
                                let prompt_card_w = (ai_panel_w - 40.0).max(120.0);
                                let suggestions = [
                                    ("Explain current file", "Explain the open file in detail and summarize its architecture."),
                                    ("Find bugs & edge cases", "Review this code for bugs, edge cases, and potential performance improvements."),
                                    ("Write unit tests", "Generate comprehensive unit tests covering key logic and edge cases."),
                                ];

                                for (label, prompt_fill) in suggestions {
                                    let card = ui.container()
                                        .width(prompt_card_w)
                                        .padding(8.0, 12.0, 8.0, 12.0)
                                        .bg(match state.theme {
                                            crate::theme::ThemeMode::Dark => Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.04),
                                            crate::theme::ThemeMode::Light => Color::rgba(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 0.03),
                                        })
                                        .border(colors.border, 1.0)
                                        .radius_all(8.0)
                                        .hover_bg(match state.theme {
                                            crate::theme::ThemeMode::Dark => Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.08),
                                            crate::theme::ThemeMode::Light => Color::rgba(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 0.06),
                                        })
                                        .row()
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text(label)
                                                .size(11.0)
                                                .color(colors.text_primary)
                                                .show();
                                        });

                                    if card.clicked {
                                        state.ai_input = prompt_fill.to_string();
                                        state.active_input_drawer = AiInputDrawer::None;
                                        ui.needs_redraw = true;
                                    }
                                }
                            });
                    }

                    let total_msgs = msgs.len();
                    let mut toggle_accordion_idx = None;
                    let mut open_file_target = None;
                    let mut open_session_target = None;

                    for (msg_idx, msg) in msgs.iter().enumerate() {
                        let is_last_msg = msg_idx + 1 == total_msgs;

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
                                        .padding(8.0, 14.0, 8.0, 14.0)
                                        .bg(match state.theme {
                                            crate::theme::ThemeMode::Dark => Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.06),
                                            crate::theme::ThemeMode::Light => Color::rgba(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 0.05),
                                        })
                                        .border(colors.border, 1.0)
                                        .radius_all(16.0)
                                        .show(|ui| {
                                            ui.text(&msg.content)
                                                .size(11.5)
                                                .line_height(1.35)
                                                .color(colors.text_primary)
                                                .max_width(user_text_w)
                                                .wrap(zenthra_text::prelude::TextWrap::Word)
                                                .show();
                                        });
                                });
                        } else {
                            // Assistant Message: Clean Accordions + Edited Badges + Markdown Flow
                            let assistant_text_w = max_content_w.max(60.0);
                            ui.container()
                                .full_width()
                                .padding(2.0, 2.0, 4.0, 2.0)
                                .column()
                                .gap(4.0)
                                .show(|ui| {
                                    // Sub-agent Badge (e.g. ZENE » 🔍 Explorer)
                                    if let Some(sub_agent) = msg.sub_agent {
                                        ui.container()
                                            .padding(1.0, 6.0, 1.0, 6.0)
                                            .bg(colors.accent.with_alpha(0.10))
                                            .radius_all(4.0)
                                            .row()
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text(&format!("ZENE » {}", sub_agent.badge()))
                                                    .size(9.0)
                                                    .weight(FontWeight::SemiBold)
                                                    .color(colors.accent)
                                                    .show();
                                            });
                                    }

                                    // 1. Collapsible "Explored N files >" / "Exploring N files v"
                                    if !msg.explored_items.is_empty() {
                                        let unique_count = msg.explored_items.iter().map(|i| &i.file_name).collect::<std::collections::HashSet<_>>().len();
                                        let is_expanded = state.expanded_explored_msgs.contains(&msg_idx)
                                            || (msg.is_working && !state.expanded_explored_msgs.contains(&(msg_idx + 1_000_000)));

                                        let arrow_sym = if is_expanded { "\u{f078}" } else { "\u{f054}" };
                                        let header_title = if msg.is_working {
                                            format!("Exploring {} file{}", unique_count, if unique_count == 1 { "" } else { "s" })
                                        } else {
                                            format!("Explored {} file{}", unique_count, if unique_count == 1 { "" } else { "s" })
                                        };

                                        let accordion_btn = ui.container()
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                            .radius_all(3.0)
                                            .row()
                                            .valign(Align::Center)
                                            .gap(5.0)
                                            .show(|ui| {
                                                ui.container()
                                                    .width(12.0)
                                                    .valign(Align::Center)
                                                    .halign(Align::Center)
                                                    .show(|ui| {
                                                        ui.text(arrow_sym)
                                                            .size(7.5)
                                                            .color(colors.text_muted)
                                                            .show();
                                                    });
                                                ui.text(&header_title)
                                                    .size(10.5)
                                                    .weight(FontWeight::Medium)
                                                    .color(colors.text_dim)
                                                    .show();
                                            });

                                        if accordion_btn.clicked {
                                            toggle_accordion_idx = Some(msg_idx);
                                        }

                                        if is_expanded {
                                            ui.container()
                                                .padding_left(12.0)
                                                .column()
                                                .gap(3.0)
                                                .show(|ui| {
                                                    for item in &msg.explored_items {
                                                        let glyph = get_file_glyph(&item.file_name);
                                                        ui.container()
                                                            .row()
                                                            .gap(5.0)
                                                            .valign(Align::Center)
                                                            .show(|ui| {
                                                                ui.text(&item.action_type)
                                                                    .size(10.5)
                                                                    .color(colors.text_dim)
                                                                    .show();
                                                                ui.text(glyph)
                                                                    .size(10.5)
                                                                    .color(colors.accent)
                                                                    .show();
                                                                ui.text(&item.file_name)
                                                                    .size(10.5)
                                                                    .weight(FontWeight::Medium)
                                                                    .color(colors.text_primary)
                                                                    .show();
                                                                if let Some(ref range) = item.line_range {
                                                                    ui.text(range)
                                                                        .size(10.0)
                                                                        .color(colors.text_dim)
                                                                        .show();
                                                                }
                                                            });
                                                    }

                                                    if msg.is_working {
                                                        let now_ms = std::time::SystemTime::now()
                                                            .duration_since(std::time::UNIX_EPOCH)
                                                            .map(|d| d.as_millis())
                                                            .unwrap_or(0);
                                                        let spinner_frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                                                        let spinner = spinner_frames[((now_ms / 80) % 10) as usize];
                                                        let dots_count = ((now_ms / 350) % 4) as usize;
                                                        let dots = match dots_count {
                                                            0 => "",
                                                            1 => ".",
                                                            2 => "..",
                                                            _ => "...",
                                                        };
                                                        let action_label = match msg.sub_agent {
                                                            Some(crate::agent::SubAgentRole::Explorer) => "Exploring workspace",
                                                            Some(crate::agent::SubAgentRole::Detector) => "Detecting files & env",
                                                            Some(crate::agent::SubAgentRole::Builder) => "Building solution",
                                                            Some(crate::agent::SubAgentRole::Tester) => "Verifying code",
                                                            None => "Analyzing & exploring",
                                                        };

                                                        ui.container()
                                                            .row()
                                                            .gap(5.0)
                                                            .valign(Align::Center)
                                                            .padding(2.0, 6.0, 2.0, 6.0)
                                                            .bg(colors.accent.with_alpha(0.08))
                                                            .border(colors.accent.with_alpha(0.25), 1.0)
                                                            .radius_all(10.0)
                                                            .show(|ui| {
                                                                ui.text(spinner)
                                                                    .size(11.0)
                                                                    .color(colors.accent)
                                                                    .show();
                                                                ui.text(&format!("{}{}", action_label, dots))
                                                                    .size(10.0)
                                                                    .weight(FontWeight::Medium)
                                                                    .color(colors.accent)
                                                                    .show();
                                                            });
                                                        ui.needs_redraw = true;
                                                    }
                                                });
                                        }
                                    }

                                    // 2. "Edited 🦀 <filename> +<added> -<removed>"
                                    let mut files_to_show = msg.edited_files.clone();
                                    if is_last_msg {
                                        if let Some(ref pending) = state.pending_modification {
                                            let fname = pending.file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                                            let added = pending.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                                            let removed = pending.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                                            if !files_to_show.iter().any(|e| e.file_name == fname) {
                                                files_to_show.push(EditedFileSummary {
                                                    file_name: fname,
                                                    lines_added: added,
                                                    lines_removed: removed,
                                                });
                                            }
                                        }
                                    }

                                    for edited in files_to_show {
                                        let glyph = get_file_glyph(&edited.file_name);
                                        let target_name = edited.file_name.clone();

                                        let edit_btn = ui.container()
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                            .radius_all(3.0)
                                            .row()
                                            .valign(Align::Center)
                                            .gap(6.0)
                                            .show(|ui| {
                                                ui.text("Edited")
                                                    .size(11.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                                ui.text(glyph)
                                                    .size(11.0)
                                                    .color(colors.accent)
                                                    .show();
                                                ui.text(&edited.file_name)
                                                    .size(11.0)
                                                    .weight(FontWeight::SemiBold)
                                                    .color(colors.text_primary)
                                                    .show();
                                                if edited.lines_added > 0 {
                                                    ui.text(&format!("+{}", edited.lines_added))
                                                        .size(10.5)
                                                        .weight(FontWeight::Bold)
                                                        .color(Color::rgb(0.35, 0.85, 0.45))
                                                        .show();
                                                }
                                                if edited.lines_removed > 0 || edited.lines_added == 0 {
                                                    ui.text(&format!("-{}", edited.lines_removed))
                                                        .size(10.5)
                                                        .weight(FontWeight::Bold)
                                                        .color(if edited.lines_removed > 0 {
                                                            Color::rgb(0.97, 0.45, 0.45)
                                                        } else {
                                                            colors.text_dim
                                                        })
                                                        .show();
                                                }
                                            });

                                        if edit_btn.clicked {
                                            open_file_target = Some(target_name);
                                        }
                                    }

                                    // 3. Markdown formatted response content
                                    if !msg.content.trim().is_empty() {
                                        let is_err_or_quota = msg.content.trim().starts_with("⚠️")
                                            || msg.content.contains("UNAVAILABLE")
                                            || msg.content.contains("quota limit")
                                            || msg.content.contains("RESOURCE_EXHAUSTED")
                                            || msg.content.contains("No capacity available");

                                        if is_err_or_quota {
                                            ui.container()
                                                .width(assistant_text_w)
                                                .padding(6.0, 8.0, 6.0, 8.0)
                                                .bg(Color::rgba(0.95, 0.45, 0.2, 0.12))
                                                .border(Color::rgba(0.95, 0.45, 0.2, 0.45), 1.0)
                                                .radius_all(6.0)
                                                .row()
                                                .gap(8.0)
                                                .valign(Align::Center)
                                                .show(|ui| {
                                                    ui.text("⚠️")
                                                        .size(14.0)
                                                        .show();
                                                    ui.container()
                                                        .column()
                                                        .gap(2.0)
                                                        .show(|ui| {
                                                            ui.text("AI Engine Service Notice")
                                                                .size(10.0)
                                                                .weight(FontWeight::Bold)
                                                                .color(Color::rgb(0.95, 0.65, 0.3))
                                                                .show();
                                                            ui.text("Quota limit or server capacity reached. Auto-routed request to active provider.")
                                                                .size(9.0)
                                                                .color(colors.text_dim)
                                                                .show();
                                                        });
                                                });
                                        }

                                        let mut copy_action = None;
                                        crate::markdown::render_markdown_preview(
                                            ui,
                                            &msg.content,
                                            assistant_text_w,
                                            &colors,
                                            state.theme,
                                            &mut copy_action,
                                            &mut state.collapsed_code_blocks,
                                        );

                                        // Code snippet block if separate
                                        if let Some(ref code) = msg.code_snippet {
                                            let mut snippet_copy = None;
                                            crate::markdown::render_markdown_preview(
                                                ui,
                                                &format!("```rust\n{}\n```", code),
                                                assistant_text_w,
                                                &colors,
                                                state.theme,
                                                &mut snippet_copy,
                                                &mut state.collapsed_code_blocks,
                                            );
                                            if let Some(c) = snippet_copy {
                                                copy_action = Some(c);
                                            }
                                        }

                                        if let Some(text_to_copy) = copy_action {
                                            state.set_clipboard(text_to_copy);
                                            state.status_message = "Code copied to clipboard".to_string();
                                            ui.needs_redraw = true;
                                        }
                                    }
                                });
                        }
                    }

                    // ─── Bottom of Response: SESSION FILES (Markdown Artifacts) ──────────
                    // As requested: Only show these files when workspace is open and mission exists.
                    // Clicking any file opens the .md file in the editor buffer tab as a preview.
                    if state.has_workspace && state.current_mission.is_some() {
                        let card_w = (ai_panel_w - 20.0).max(180.0);
                        let has_active_mission = state.current_mission.as_ref().map(|m| m.is_active).unwrap_or(false);

                        let is_collapsed = state.session_artifacts_collapsed;
                        let arrow_icon = if is_collapsed { "\u{f054}" } else { "\u{f078}" }; // ▶ right vs ▼ down

                        ui.container()
                            .width(card_w)
                            .bg(colors.bg_sidebar)
                            .border(if has_active_mission { colors.accent.with_alpha(0.5) } else { colors.border }, 1.0)
                            .radius_all(6.0)
                            .padding(6.0, 10.0, 6.0, 10.0)
                            .column()
                            .gap(if is_collapsed { 0.0 } else { 6.0 })
                            .show(|ui| {
                                // Header: Session Files Title & Status Badge
                                ui.container()
                                    .width(card_w - 20.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        let toggle_btn = ui.container()
                                            .row()
                                            .gap(5.0)
                                            .valign(Align::Center)
                                            .padding(1.0, 4.0, 1.0, 4.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text(arrow_icon)
                                                    .size(8.5)
                                                    .color(colors.accent)
                                                    .show();
                                                ui.text("\u{ea80}")
                                                    .size(11.0)
                                                    .color(colors.accent)
                                                    .show();
                                                ui.text(" SESSION ARTIFACTS")
                                                    .size(9.5)
                                                    .weight(FontWeight::Bold)
                                                    .color(colors.accent)
                                                    .show();
                                            });
                                        if toggle_btn.clicked {
                                            state.session_artifacts_collapsed = !state.session_artifacts_collapsed;
                                            ui.needs_redraw = true;
                                        }

                                        if let Some(ref mission) = state.current_mission {
                                            ui.container()
                                                .row()
                                                .gap(4.0)
                                                .valign(Align::Center)
                                                .show(|ui| {
                                                    let total_tasks = mission.tasks.len();
                                                    let completed_tasks = mission.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
                                                    let status_color = if !mission.is_active && completed_tasks == total_tasks && total_tasks > 0 {
                                                        Color::rgb(0.35, 0.85, 0.45)
                                                    } else if mission.is_active {
                                                        colors.accent
                                                    } else {
                                                        colors.text_muted
                                                    };
                                                    ui.container()
                                                        .padding(1.0, 5.0, 1.0, 5.0)
                                                        .bg(status_color.with_alpha(0.15))
                                                        .radius_all(3.0)
                                                        .show(|ui| {
                                                            ui.text(&format!("{}/{} tasks", completed_tasks, total_tasks))
                                                                .size(8.5)
                                                                .weight(FontWeight::SemiBold)
                                                                .color(status_color)
                                                                .show();
                                                        });

                                                    if let Some(active_idx) = mission.active_task_index {
                                                        if let Some(active_task) = mission.tasks.get(active_idx) {
                                                            if let Some(sub_agent) = active_task.sub_agent {
                                                                ui.container()
                                                                    .padding(1.0, 5.0, 1.0, 5.0)
                                                                    .bg(colors.accent.with_alpha(0.12))
                                                                    .radius_all(3.0)
                                                                    .show(|ui| {
                                                                        ui.text(&sub_agent.badge())
                                                                            .size(8.5)
                                                                            .weight(FontWeight::SemiBold)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                    });
                                                            }
                                                        }
                                                    }
                                                });
                                        }
                                    });

                                if !is_collapsed {
                                    // Progress Bar if mission is present
                                    if let Some(ref mission) = state.current_mission {
                                        let total_tasks = mission.tasks.len();
                                        let completed_tasks = mission.tasks.iter().filter(|t| t.status == TaskStatus::Completed).count();
                                        let progress_ratio = if total_tasks > 0 { completed_tasks as f32 / total_tasks as f32 } else { 0.0 };
                                        ui.container()
                                            .width(card_w - 20.0)
                                            .height(2.5)
                                            .bg(colors.bg_base)
                                            .radius_all(1.25)
                                            .show(|ui| {
                                                if progress_ratio > 0.0 {
                                                    ui.container()
                                                        .width((card_w - 20.0) * progress_ratio)
                                                        .height(2.5)
                                                        .bg(if progress_ratio >= 1.0 { Color::rgb(0.35, 0.85, 0.45) } else { colors.accent })
                                                        .radius_all(1.25)
                                                        .show(|_| {});
                                                }
                                            });
                                    }

                                    ui.text("Click to preview file in editor buffer:")
                                        .size(9.0)
                                        .color(colors.text_dim)
                                        .show();

                                    // 1. implementation_plan.md Chip
                                    let plan_pill = ui.container()
                                        .full_width()
                                        .height(22.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .bg(colors.bg_base)
                                        .hover_bg(colors.highlight)
                                        .border(colors.border, 1.0)
                                        .radius_all(4.0)
                                        .row()
                                        .valign(Align::Center)
                                        .gap(6.0)
                                        .show(|ui| {
                                            ui.text("\u{ea7f}")
                                                .size(11.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.text("implementation_plan.md")
                                                .size(9.5)
                                                .weight(FontWeight::SemiBold)
                                                .color(colors.text_primary)
                                                .show();
                                            ui.text("Plan")
                                                .size(8.5)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.text("↗")
                                                .size(9.0)
                                                .color(colors.accent)
                                                .show();
                                        });
                                    if plan_pill.clicked {
                                        open_session_target = Some("implementation_plan.md".to_string());
                                    }

                                    // 2. mission.md Chip (replaces small checklist)
                                    let mission_status_label = if let Some(ref m) = state.current_mission {
                                        if m.is_paused {
                                            "Paused"
                                        } else if m.is_active {
                                            "Active"
                                        } else {
                                            "Verified"
                                        }
                                    } else {
                                        "Ready"
                                    };

                                    let mission_pill = ui.container()
                                        .full_width()
                                        .height(22.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .bg(colors.bg_base)
                                        .hover_bg(colors.highlight)
                                        .border(colors.border, 1.0)
                                        .radius_all(4.0)
                                        .row()
                                        .valign(Align::Center)
                                        .gap(6.0)
                                        .show(|ui| {
                                            ui.text("\u{f0e7}")
                                                .size(10.5)
                                                .color(Color::rgb(0.9, 0.75, 0.2))
                                                .show();
                                            ui.text("mission.md")
                                                .size(9.5)
                                                .weight(FontWeight::SemiBold)
                                                .color(colors.text_primary)
                                                .show();
                                            ui.text(mission_status_label)
                                                .size(8.5)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.text("↗")
                                                .size(9.0)
                                                .color(colors.accent)
                                                .show();
                                        });
                                    if mission_pill.clicked {
                                        open_session_target = Some("mission.md".to_string());
                                    }

                                    // 3. walkthrough.md Chip
                                    let walkthrough_label = if let Some(ref m) = state.current_mission {
                                        if !m.is_active && m.tasks.iter().all(|t| t.status == TaskStatus::Completed) {
                                            "Completed"
                                        } else {
                                            "Summary"
                                        }
                                    } else {
                                        "Summary"
                                    };

                                    let walkthrough_pill = ui.container()
                                        .full_width()
                                        .height(22.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .bg(colors.bg_base)
                                        .hover_bg(colors.highlight)
                                        .border(colors.border, 1.0)
                                        .radius_all(4.0)
                                        .row()
                                        .valign(Align::Center)
                                        .gap(6.0)
                                        .show(|ui| {
                                            ui.text("\u{f15c}")
                                                .size(11.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.text("walkthrough.md")
                                                .size(9.5)
                                                .weight(FontWeight::SemiBold)
                                                .color(colors.text_primary)
                                                .show();
                                            ui.text(walkthrough_label)
                                                .size(8.5)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.text("↗")
                                                .size(9.0)
                                                .color(colors.accent)
                                                .show();
                                        });
                                    if walkthrough_pill.clicked {
                                        open_session_target = Some("walkthrough.md".to_string());
                                    }
                                }
                            });
                    }

                    if let Some(idx) = toggle_accordion_idx {
                        state.toggle_explored_accordion(idx);
                        ui.needs_redraw = true;
                    }
                    if let Some(target) = open_file_target {
                        state.open_file_by_name(&target);
                        ui.needs_redraw = true;
                    }
                    if let Some(target) = open_session_target {
                        state.open_session_file(&target);
                        ui.needs_redraw = true;
                    }
                });

            // ─── Active Drawer: Popover above Toolbar ──────────────────────
            let mut file_to_open = None;

            match state.active_input_drawer {
                AiInputDrawer::PendingModifications => {
                    let pending_list: Vec<&PendingModification> = if !state.pending_modifications.is_empty() {
                        state.pending_modifications.iter().collect()
                    } else if let Some(ref pending) = state.pending_modification {
                        vec![pending]
                    } else {
                        Vec::new()
                    };

                    if !pending_list.is_empty() {
                        let total_files = pending_list.len();
                        let file_list_h = (total_files as f32 * 26.0 + 8.0).min(130.0);
                        let mut accept_pressed = false;
                        let mut reject_pressed = false;

                        ui.container()
                            .width(ai_panel_w)
                            .bg(colors.bg_sidebar)
                            .border(colors.border, 1.0)
                            .radius(4.0, 4.0, 0.0, 0.0)
                            .column()
                            .show(|ui| {
                                ui.container()
                                    .id(Id::from_u64(999999860))
                                    .width(ai_panel_w - 2.0)
                                    .height(file_list_h)
                                    .padding(4.0, 8.0, 4.0, 8.0)
                                    .column()
                                    .gap(2.0)
                                    .scroll_y(true)
                                    .show(|ui| {
                                        for item in &pending_list {
                                            let added = item.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                                            let removed = item.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                                            let is_new = item.original_code.trim().is_empty() || (removed == 0 && added > 0);

                                            let file_name = item.file_path.file_name()
                                                .and_then(|n| n.to_str())
                                                .unwrap_or("file")
                                                .to_string();

                                            let rel_path = item.file_path.strip_prefix(&state.current_dir)
                                                .unwrap_or(&item.file_path)
                                                .to_string_lossy()
                                                .to_string();

                                            let file_row = ui.container()
                                                .width(ai_panel_w - 18.0)
                                                .height(24.0)
                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                .radius_all(3.0)
                                                .hover_bg(colors.highlight)
                                                .row()
                                                .valign(Align::Center)
                                                .gap(8.0)
                                                .show(|ui| {
                                                    if is_new {
                                                        ui.container()
                                                            .width(16.0)
                                                            .height(16.0)
                                                            .border(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0), 1.0)
                                                            .bg(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0).with_alpha(0.12))
                                                            .radius_all(3.0)
                                                            .valign(Align::Center)
                                                            .halign(Align::Center)
                                                            .show(|ui| {
                                                                ui.text("+")
                                                                    .size(11.0)
                                                                    .weight(FontWeight::Bold)
                                                                    .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                                    .show();
                                                            });
                                                    } else {
                                                        ui.container()
                                                            .width(16.0)
                                                            .height(16.0)
                                                            .border(Color::rgb(249.0 / 255.0, 115.0 / 255.0, 22.0 / 255.0), 1.0)
                                                            .bg(Color::rgb(249.0 / 255.0, 115.0 / 255.0, 22.0 / 255.0).with_alpha(0.12))
                                                            .radius_all(3.0)
                                                            .valign(Align::Center)
                                                            .halign(Align::Center)
                                                            .show(|ui| {
                                                                ui.text("•")
                                                                    .size(11.0)
                                                                    .weight(FontWeight::Bold)
                                                                    .color(Color::rgb(249.0 / 255.0, 115.0 / 255.0, 22.0 / 255.0))
                                                                    .show();
                                                            });
                                                    }

                                                    ui.container()
                                                        .row()
                                                        .gap(4.0)
                                                        .valign(Align::Center)
                                                        .show(|ui| {
                                                            ui.text(&format!("+{}", added))
                                                                .size(10.0)
                                                                .weight(FontWeight::Medium)
                                                                .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                                .show();
                                                            ui.text(&format!("-{}", removed))
                                                                .size(10.0)
                                                                .weight(FontWeight::Medium)
                                                                .color(if removed > 0 { Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0) } else { colors.text_dim })
                                                                .show();
                                                        });

                                                    ui.text(&file_name)
                                                        .size(10.5)
                                                        .weight(FontWeight::SemiBold)
                                                        .color(colors.text_primary)
                                                        .show();

                                                    ui.text(&rel_path)
                                                        .size(9.5)
                                                        .weight(FontWeight::Regular)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });

                                            if file_row.clicked {
                                                file_to_open = Some(item.file_path.clone());
                                            }
                                        }
                                    });

                                ui.container()
                                    .width(ai_panel_w - 2.0)
                                    .height(1.0)
                                    .bg(colors.border)
                                    .show(|_| {});

                                ui.container()
                                    .width(ai_panel_w - 2.0)
                                    .height(32.0)
                                    .padding(0.0, 8.0, 0.0, 8.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.container()
                                            .row()
                                            .gap(6.0)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text("\u{ea7f}")
                                                    .size(12.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                                let label = format!("{} File{} With Changes", total_files, if total_files == 1 { "" } else { "s" });
                                                ui.text(&label)
                                                    .size(10.0)
                                                    .weight(FontWeight::Medium)
                                                    .color(colors.text_primary)
                                                    .show();
                                            });

                                        ui.container()
                                            .row()
                                            .gap(6.0)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                let reject_btn = ui.container()
                                                    .padding(2.0, 6.0, 2.0, 6.0)
                                                    .bg(colors.bg_base)
                                                    .border(colors.border, 1.0)
                                                    .hover_bg(colors.highlight)
                                                    .radius_all(3.0)
                                                    .show(|ui| {
                                                        ui.text("Reject all")
                                                            .size(10.0)
                                                            .weight(FontWeight::Medium)
                                                            .color(colors.text_muted)
                                                            .show();
                                                    });
                                                if reject_btn.clicked {
                                                    reject_pressed = true;
                                                }

                                                let accept_btn = ui.container()
                                                    .padding(2.0, 8.0, 2.0, 8.0)
                                                    .bg(colors.accent)
                                                    .hover_bg(colors.accent.with_alpha(0.85))
                                                    .radius_all(3.0)
                                                    .row()
                                                    .gap(3.0)
                                                    .valign(Align::Center)
                                                    .show(|ui| {
                                                        ui.text("Accept all")
                                                            .size(10.0)
                                                            .weight(FontWeight::SemiBold)
                                                            .color(Color::WHITE)
                                                            .show();
                                                        ui.text("\u{f078}")
                                                            .size(7.5)
                                                            .color(Color::WHITE)
                                                            .show();
                                                    });
                                                if accept_btn.clicked {
                                                    accept_pressed = true;
                                                }
                                            });
                                    });
                            });

                        if accept_pressed {
                            state.accept_all_modifications();
                            ui.needs_redraw = true;
                        } else if reject_pressed {
                            state.reject_all_modifications();
                            ui.needs_redraw = true;
                        }
                    } else {
                        ui.container()
                            .width(ai_panel_w)
                            .height(38.0)
                            .bg(colors.bg_sidebar)
                            .border(colors.border, 1.0)
                            .padding_left(12.0)
                            .row()
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("No pending modifications to review")
                                    .size(10.5)
                                    .color(colors.text_muted)
                                    .show();
                            });
                    }
                }
                AiInputDrawer::Terminals => {
                    let mut close_term_requested = false;
                    let mut start_term_requested = false;

                    ui.container()
                        .width(ai_panel_w)
                        .height(72.0)
                        .bg(colors.bg_sidebar)
                        .border(colors.border, 1.0)
                        .radius(4.0, 4.0, 0.0, 0.0)
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .column()
                        .gap(4.0)
                        .show(|ui| {
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .gap(6.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{ea85}")
                                                .size(12.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.text("Background Terminals")
                                                .size(10.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if state.terminal_session.is_some() {
                                        ui.text("1 running")
                                            .size(9.5)
                                            .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                            .show();
                                    }
                                });

                            if state.terminal_session.is_some() {
                                ui.container()
                                    .width(ai_panel_w - 22.0)
                                    .height(28.0)
                                    .bg(colors.bg_base)
                                    .border(colors.border, 1.0)
                                    .padding(2.0, 6.0, 2.0, 6.0)
                                    .radius_all(3.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.container()
                                            .row()
                                            .gap(6.0)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text("●")
                                                    .size(6.0)
                                                    .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                    .show();
                                                ui.text("Terminal 1: bash (Interactive Shell)")
                                                    .size(10.0)
                                                    .weight(FontWeight::Medium)
                                                    .color(colors.text_primary)
                                                    .show();
                                            });

                                        let close_btn = ui.container()
                                            .padding(1.0, 5.0, 1.0, 5.0)
                                            .bg(colors.bg_sidebar)
                                            .border(colors.border, 1.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("✕")
                                                    .size(9.5)
                                                    .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                                    .show();
                                            });
                                        if close_btn.clicked {
                                            close_term_requested = true;
                                        }
                                    });
                            } else {
                                ui.container()
                                    .width(ai_panel_w - 22.0)
                                    .height(28.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.text("No background terminals running")
                                            .size(10.0)
                                            .color(colors.text_muted)
                                            .show();

                                        let start_btn = ui.container()
                                            .padding(2.0, 6.0, 2.0, 6.0)
                                            .bg(colors.bg_base)
                                            .border(colors.border, 1.0)
                                            .hover_bg(colors.highlight)
                                            .radius_all(3.0)
                                            .show(|ui| {
                                                ui.text("+ Start Terminal")
                                                    .size(9.5)
                                                    .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                    .show();
                                            });
                                        if start_btn.clicked {
                                            start_term_requested = true;
                                        }
                                    });
                            }
                        });

                    if close_term_requested {
                        state.terminal_session = None;
                        state.terminal_lines.clear();
                        state.terminal_visible = false;
                        ui.needs_redraw = true;
                    } else if start_term_requested {
                        state.ensure_terminal_session();
                        state.terminal_visible = true;
                        ui.needs_redraw = true;
                    }
                }
                AiInputDrawer::Artifacts => {
                    let arts = state.current_session.list_artifacts();
                    let arts_h = (arts.len() as f32 * 26.0 + 38.0).clamp(44.0, 150.0);

                    ui.container()
                        .width(ai_panel_w)
                        .height(arts_h)
                        .bg(colors.bg_sidebar)
                        .border(colors.border, 1.0)
                        .radius(4.0, 4.0, 0.0, 0.0)
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .gap(6.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{ea84}")
                                                .size(12.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.text("Session Artifacts")
                                                .size(10.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    ui.text("Updated-wise")
                                        .size(9.0)
                                        .color(colors.text_dim)
                                        .show();
                                });

                            for art in &arts {
                                let art_row = ui.container()
                                    .width(ai_panel_w - 22.0)
                                    .height(24.0)
                                    .padding(2.0, 4.0, 2.0, 4.0)
                                    .radius_all(3.0)
                                    .hover_bg(colors.highlight)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.container()
                                            .row()
                                            .gap(6.0)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text("\u{ea7f}")
                                                    .size(11.0)
                                                    .color(colors.accent)
                                                    .show();
                                                ui.text(&art.name)
                                                    .size(10.0)
                                                    .weight(FontWeight::Medium)
                                                    .color(colors.text_primary)
                                                    .show();
                                            });

                                        ui.container()
                                            .row()
                                            .gap(6.0)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text(&format_relative_time(art.updated_at))
                                                    .size(9.0)
                                                    .color(colors.text_dim)
                                                    .show();
                                                ui.text(&format_bytes(art.size_bytes))
                                                    .size(9.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                    });

                                if art_row.clicked {
                                    file_to_open = Some(art.path.clone());
                                }
                            }
                        });
                }
                AiInputDrawer::SessionChanges => {
                    let total = state.session_modified_files.len();
                    let rows_h = (total as f32 * 26.0 + 38.0).clamp(44.0, 160.0);

                    ui.container()
                        .width(ai_panel_w)
                        .height(rows_h)
                        .bg(colors.bg_sidebar)
                        .border(colors.border, 1.0)
                        .radius(4.0, 4.0, 0.0, 0.0)
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .gap(6.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{ea71}")
                                                .size(12.0)
                                                .color(colors.accent)
                                                .show();
                                            ui.text("Whole Session Code Review")
                                                .size(10.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    ui.text(&format!("{} modified file{}", total, if total == 1 { "" } else { "s" }))
                                        .size(9.0)
                                        .color(colors.text_dim)
                                        .show();
                                });

                            if state.session_modified_files.is_empty() {
                                ui.text("No files modified in this session yet")
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();
                            } else {
                                for item in &state.session_modified_files {
                                    let rel_path = item.file_path.strip_prefix(&state.current_dir)
                                        .unwrap_or(&item.file_path)
                                        .to_string_lossy()
                                        .to_string();

                                    let row = ui.container()
                                        .width(ai_panel_w - 22.0)
                                        .height(24.0)
                                        .padding(2.0, 4.0, 2.0, 4.0)
                                        .radius_all(3.0)
                                        .hover_bg(colors.highlight)
                                        .row()
                                        .valign(Align::Center)
                                        .halign(Align::SpaceBetween)
                                        .show(|ui| {
                                            ui.container()
                                                .row()
                                                .gap(6.0)
                                                .valign(Align::Center)
                                                .show(|ui| {
                                                    if item.is_new_file {
                                                        ui.text("[+]")
                                                            .size(10.0)
                                                            .weight(FontWeight::Bold)
                                                            .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                            .show();
                                                    } else {
                                                        ui.text("[•]")
                                                            .size(10.0)
                                                            .weight(FontWeight::Bold)
                                                            .color(Color::rgb(249.0 / 255.0, 115.0 / 255.0, 22.0 / 255.0))
                                                            .show();
                                                    }

                                                    ui.text(&format!("+{}", item.lines_added))
                                                        .size(9.5)
                                                        .weight(FontWeight::Medium)
                                                        .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                        .show();
                                                    ui.text(&format!("-{}", item.lines_removed))
                                                        .size(9.5)
                                                        .weight(FontWeight::Medium)
                                                        .color(if item.lines_removed > 0 { Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0) } else { colors.text_dim })
                                                        .show();

                                                    ui.text(&item.file_name)
                                                        .size(10.0)
                                                        .weight(FontWeight::SemiBold)
                                                        .color(colors.text_primary)
                                                        .show();

                                                    ui.text(&rel_path)
                                                        .size(9.0)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });

                                            ui.text(&format_relative_time(item.timestamp))
                                                .size(9.0)
                                                .color(colors.text_dim)
                                                .show();
                                        });

                                    if row.clicked {
                                        file_to_open = Some(item.file_path.clone());
                                    }
                                }
                            }
                        });
                }
                AiInputDrawer::None => {}
            }

            if let Some(target) = file_to_open {
                state.open_file(&target);
                ui.needs_redraw = true;
            }

            // ─── Top of Input Prompt Area: Toolbar (Unified Framework Badges) ───
            let has_pending = !state.pending_modifications.is_empty() || state.pending_modification.is_some();
            let has_running_term = state.terminal_session.is_some();
            let has_artifacts = state.current_session.has_plan || state.current_session.has_mission || state.current_session.has_walkthrough;

            ui.container()
                .width(ai_panel_w)
                .height(TOOLBAR_H)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding(0.0, 8.0, 0.0, 8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left toolbar badges: Modified Files, Terminal, Artifacts
                    ui.container()
                        .row()
                        .gap(6.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            // 1. Modified Files Badge
                            let is_pending_open = state.active_input_drawer == AiInputDrawer::PendingModifications;
                            let mod_files_btn = ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .bg(if is_pending_open { colors.highlight } else { colors.bg_base })
                                .border(if is_pending_open { colors.accent } else { colors.border }, 1.0)
                                .hover_bg(colors.highlight)
                                .radius_all(3.0)
                                .row()
                                .gap(3.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{ea7f}")
                                        .size(12.0)
                                        .color(if is_pending_open { colors.text_primary } else { colors.text_muted })
                                        .show();
                                    if has_pending {
                                        ui.text("●")
                                            .size(6.0)
                                            .color(colors.accent)
                                            .show();
                                    }
                                });
                            if mod_files_btn.clicked {
                                state.active_input_drawer = if is_pending_open { AiInputDrawer::None } else { AiInputDrawer::PendingModifications };
                                ui.needs_redraw = true;
                            }

                            // 2. Terminal List Badge
                            let is_term_open = state.active_input_drawer == AiInputDrawer::Terminals;
                            let term_btn = ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .bg(if is_term_open { colors.highlight } else { colors.bg_base })
                                .border(if is_term_open { colors.accent } else { colors.border }, 1.0)
                                .hover_bg(colors.highlight)
                                .radius_all(3.0)
                                .row()
                                .gap(3.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{ea85}")
                                        .size(12.0)
                                        .color(if is_term_open { colors.text_primary } else { colors.text_muted })
                                        .show();
                                    if has_running_term {
                                        ui.text("●")
                                            .size(6.0)
                                            .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                            .show();
                                    }
                                });
                            if term_btn.clicked {
                                state.active_input_drawer = if is_term_open { AiInputDrawer::None } else { AiInputDrawer::Terminals };
                                ui.needs_redraw = true;
                            }

                            // 3. Artifact Box Badge
                            let is_art_open = state.active_input_drawer == AiInputDrawer::Artifacts;
                            let art_btn = ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .bg(if is_art_open { colors.highlight } else { colors.bg_base })
                                .border(if is_art_open { colors.accent } else { colors.border }, 1.0)
                                .hover_bg(colors.highlight)
                                .radius_all(3.0)
                                .row()
                                .gap(3.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{ea84}")
                                        .size(12.0)
                                        .color(if is_art_open { colors.text_primary } else { colors.text_muted })
                                        .show();
                                    if has_artifacts {
                                        ui.text("●")
                                            .size(6.0)
                                            .color(colors.accent)
                                            .show();
                                    }
                                });
                            if art_btn.clicked {
                                state.active_input_drawer = if is_art_open { AiInputDrawer::None } else { AiInputDrawer::Artifacts };
                                ui.needs_redraw = true;
                            }
                        });

                    // Right toolbar: [ \u{ea71} Review Changes ] Badge
                    let is_rev_open = state.active_input_drawer == AiInputDrawer::SessionChanges;
                    let review_btn = ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .bg(if is_rev_open { colors.highlight } else { colors.bg_base })
                        .border(if is_rev_open { colors.accent } else { colors.border }, 1.0)
                        .hover_bg(colors.highlight)
                        .radius_all(3.0)
                        .row()
                        .gap(4.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{ea71}")
                                .size(12.0)
                                .color(if is_rev_open { colors.text_primary } else { colors.text_muted })
                                .show();
                            ui.text("Review Changes")
                                .size(9.0)
                                .weight(FontWeight::Medium)
                                .color(if is_rev_open { colors.text_primary } else { colors.text_muted })
                                .show();
                        });
                    if review_btn.clicked {
                        state.active_input_drawer = if is_rev_open { AiInputDrawer::None } else { AiInputDrawer::SessionChanges };
                        ui.needs_redraw = true;
                    }
                });

            // ─── AI Quota & Engine Status Bar ─────────────────────────────────
            let quota_banner_w = ai_panel_w;
            let active_notice = state.last_provider_notice.clone();
            let mut dismiss_notice = false;

            if let Some(ref notice) = active_notice {
                ui.container()
                    .width(quota_banner_w)
                    .bg(Color::rgba(0.95, 0.45, 0.2, 0.12))
                    .border(Color::rgba(0.95, 0.45, 0.2, 0.35), 1.0)
                    .padding(3.0, 8.0, 3.0, 8.0)
                    .row()
                    .gap(6.0)
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .show(|ui| {
                        ui.container()
                            .row()
                            .gap(5.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("⚠️")
                                    .size(10.0)
                                    .show();
                                ui.text(notice)
                                    .size(9.0)
                                    .weight(FontWeight::Medium)
                                    .color(Color::rgb(0.95, 0.65, 0.3))
                                    .show();
                            });
                        let dismiss = ui.container()
                            .padding(1.0, 4.0, 1.0, 4.0)
                            .hover_bg(colors.highlight)
                            .radius_all(2.0)
                            .show(|ui| {
                                ui.text("✕")
                                    .size(8.5)
                                    .color(colors.text_muted)
                                    .show();
                            });
                        if dismiss.clicked {
                            dismiss_notice = true;
                        }
                    });

                if dismiss_notice {
                    state.last_provider_notice = None;
                    ui.needs_redraw = true;
                }
            } else {
                ui.container()
                    .width(quota_banner_w)
                    .height(18.0)
                    .bg(colors.bg_sidebar)
                    .border(colors.border.with_alpha(0.4), 0.5)
                    .padding(1.0, 8.0, 1.0, 8.0)
                    .row()
                    .gap(8.0)
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .show(|ui| {
                        ui.container()
                            .row()
                            .gap(4.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("⚡")
                                    .size(8.5)
                                    .color(Color::rgb(0.35, 0.85, 0.45))
                                    .show();
                                ui.text("AI Engine:")
                                    .size(8.5)
                                    .color(colors.text_dim)
                                    .show();
                                ui.text("Auto-Failover Active · Groq & Gemini")
                                    .size(8.5)
                                    .weight(FontWeight::Medium)
                                    .color(colors.text_muted)
                                    .show();
                            });
                        ui.container()
                            .row()
                            .gap(4.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("Quota: Ready")
                                    .size(8.5)
                                    .weight(FontWeight::SemiBold)
                                    .color(Color::rgb(0.35, 0.85, 0.45))
                                    .show();
                            });
                    });
            }

            // ─── Bottom Prompt Input (Auto-growing TextArea, Context Menu, Model Selector & Circle Send Button) ─────
            ui.container()
                .width(ai_panel_w)
                .height(bottom_input_total_h)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .padding(6.0, 8.0, 6.0, 8.0)
                .column()
                .gap(6.0)
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

                    let avail_input_w = (ai_panel_w - 18.0).max(40.0);
                    let placeholder = if ai_panel_w >= 320.0 {
                        "Message AI, @ to include context, / for commands"
                    } else if ai_panel_w >= 240.0 {
                        "Message AI, @ for context..."
                    } else {
                        "Ask AI..."
                    };

                    // Child 1: Full-width Native Zenthra framework TextArea widget
                    let text_res = ui.text_area(&mut state.ai_input, "ai_prompt_text_area")
                        .width(avail_input_w)
                        .height(MIN_PROMPT_H)
                        .max_height(MAX_PROMPT_H)
                        .size(11.5)
                        .line_height(1.25)
                        .color(colors.text_primary)
                        .bg(colors.bg_sidebar)
                        .border(Color::TRANSPARENT, 0.0)
                        .focus_border(Color::TRANSPARENT, 0.0)
                        .padding(2.0, 2.0, 2.0, 2.0)
                        .placeholder(placeholder)
                        .placeholder_color(colors.text_muted.with_alpha(0.5))
                        .wrap(zenthra_text::prelude::TextWrap::Word)
                        .submit_on_enter(true)
                        .show();

                    // Child 2: Action Bar (Below TextArea)
                    ui.container()
                        .full_width()
                        .height(28.0)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .no_wrap()
                        .show(|ui| {
                            // Left group: '+' button + focus/agent crosshair button
                            ui.container()
                                .row()
                                .gap(4.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let add_ctx_btn = ui.container()
                                        .id(Id::from_u64(999998601))
                                        .width(24.0)
                                        .height(24.0)
                                        .radius_all(4.0)
                                        .bg(if state.ai_context_menu_open { Color::rgba(1.0, 1.0, 1.0, 0.14) } else { Color::TRANSPARENT })
                                        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                                        .halign(Align::Center)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("+")
                                                .size(14.0)
                                                .weight(FontWeight::Medium)
                                                .color(colors.text_muted)
                                                .show();
                                        });

                                    if add_ctx_btn.clicked {
                                        state.ai_context_menu_open = !state.ai_context_menu_open;
                                        state.ai_model_menu_open = false;
                                        ui.needs_redraw = true;
                                    }

                                    let focus_btn = ui.container()
                                        .id(Id::from_u64(999998602))
                                        .width(24.0)
                                        .height(24.0)
                                        .radius_all(4.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                                        .halign(Align::Center)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{f05b}")
                                                .size(11.0)
                                                .color(colors.text_muted)
                                                .show();
                                        });

                                    if focus_btn.clicked {
                                        state.active_input_drawer = if state.active_input_drawer == AiInputDrawer::None {
                                            AiInputDrawer::Artifacts
                                        } else {
                                            AiInputDrawer::None
                                        };
                                        ui.needs_redraw = true;
                                    }
                                });

                            // Right group: Model Selection button + Circle Send button
                            ui.container()
                                .row()
                                .gap(8.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let model_display = if state.model_auto_mode {
                                        "Auto"
                                    } else if !state.selected_model.is_empty() {
                                        friendly_model_name(&state.selected_model)
                                    } else {
                                        "Select a Model"
                                    };

                                    let model_btn = ui.container()
                                        .id(Id::from_u64(999998603))
                                        .height(24.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .radius_all(4.0)
                                        .bg(if state.ai_model_menu_open { Color::rgba(1.0, 1.0, 1.0, 0.14) } else { Color::TRANSPARENT })
                                        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                                        .row()
                                        .gap(3.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text(model_display)
                                                .size(10.5)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.text("\u{f107}") // ⌵ chevron
                                                .size(8.5)
                                                .color(colors.text_muted)
                                                .show();
                                        });

                                    if model_btn.clicked {
                                        state.ai_model_menu_open = !state.ai_model_menu_open;
                                        state.ai_context_menu_open = false;
                                        ui.needs_redraw = true;
                                    }

                                    // Simple arrow send button
                                    let is_working = state.is_agent_working();
                                    let is_paused = state.current_mission.as_ref().map(|m| m.is_paused).unwrap_or(false);
                                    let (icon, btn_bg, btn_hover) = if is_working {
                                        ("\u{f04c}", Color::rgb(0.92, 0.58, 0.20), Color::rgb(0.98, 0.68, 0.30)) // Amber Pause
                                    } else if is_paused && state.ai_input.trim().is_empty() {
                                        ("\u{f04b}", Color::rgb(0.35, 0.85, 0.45), Color::rgb(0.45, 0.95, 0.55)) // Green Resume
                                    } else {
                                        ("\u{f062}", colors.accent, colors.accent.with_alpha(0.85)) // Simple arrow up
                                    };

                                    let send_circle = ui.container()
                                        .id(Id::from_u64(999998604))
                                        .width(26.0)
                                        .height(26.0)
                                        .radius_all(13.0)
                                        .bg(btn_bg)
                                        .hover_bg(btn_hover)
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text(icon)
                                                .size(if is_working { 10.5 } else { 11.0 })
                                                .weight(FontWeight::Bold)
                                                .color(Color::WHITE)
                                                .show();
                                        });

                                    if send_circle.clicked {
                                        if is_working {
                                            state.pause_agent_mission();
                                            ui.needs_redraw = true;
                                        } else if is_paused && state.ai_input.trim().is_empty() {
                                            state.resume_agent_mission();
                                            ui.needs_redraw = true;
                                        } else {
                                            let msg = state.ai_input.trim().to_string();
                                            if !msg.is_empty() {
                                                state.send_ai_message(&msg);
                                            }
                                            state.ai_input.clear();
                                            ui.cursor_state.insert(ai_input_id, 0);
                                            ui.needs_redraw = true;
                                        }
                                    } else if trigger_send || text_res.submitted {
                                        if !is_working {
                                            let msg = state.ai_input.trim().to_string();
                                            if !msg.is_empty() {
                                                state.send_ai_message(&msg);
                                            }
                                            state.ai_input.clear();
                                            ui.cursor_state.insert(ai_input_id, 0);
                                            ui.needs_redraw = true;
                                        }
                                    }
                                });
                        });

                    // Child 3: Mode & Permissions Sub-Toolbar
                    ui.container()
                        .full_width()
                        .height(20.0)
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            let mode_btn = ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .bg(colors.bg_base)
                                .hover_bg(colors.highlight)
                                .border(colors.border, 1.0)
                                .radius_all(3.0)
                                .show(|ui| {
                                    ui.text("\u{f1d8} Agent")
                                        .size(8.5)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if mode_btn.clicked {
                                state.status_message = "Agent mode active".to_string();
                                ui.needs_redraw = true;
                            }

                            let auto_btn = ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .bg(colors.bg_base)
                                .hover_bg(colors.highlight)
                                .border(colors.border, 1.0)
                                .radius_all(3.0)
                                .show(|ui| {
                                    ui.text("\u{f085} Auto")
                                        .size(8.5)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if auto_btn.clicked {
                                state.status_message = "Auto mode active".to_string();
                                ui.needs_redraw = true;
                            }

                            let mic_btn = ui.container()
                                .width(20.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f130}")
                                        .size(9.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if mic_btn.clicked {
                                state.status_message = "Voice input is not available yet".to_string();
                                ui.needs_redraw = true;
                            }

                            ui.container()
                                .full_width()
                                .row()
                                .halign(Align::Right)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let interactive_btn = ui.container()
                                        .padding(2.0, 5.0, 2.0, 5.0)
                                        .hover_bg(colors.highlight)
                                        .show(|ui| {
                                            ui.text("\u{f03d} Interactive")
                                                .size(8.5)
                                                .color(colors.text_muted)
                                                .show();
                                        });
                                    if interactive_btn.clicked {
                                        state.status_message = "Interactive mode active".to_string();
                                        ui.needs_redraw = true;
                                    }

                                    let permissions_btn = ui.container()
                                        .padding(2.0, 5.0, 2.0, 5.0)
                                        .bg(if state.ai_manual_permissions {
                                            colors.accent.with_alpha(0.16)
                                        } else {
                                            colors.bg_base
                                        })
                                        .border(
                                            if state.ai_manual_permissions {
                                                colors.accent
                                            } else {
                                                colors.border
                                            },
                                            1.0,
                                        )
                                        .radius_all(3.0)
                                        .hover_bg(colors.highlight)
                                        .show(|ui| {
                                            ui.text(if state.ai_manual_permissions {
                                                "\u{f023} Manual permissions"
                                            } else {
                                                "\u{f09c} Auto permissions"
                                            })
                                                .size(8.5)
                                                .color(if state.ai_manual_permissions {
                                                    colors.accent
                                                } else {
                                                    colors.text_muted
                                                })
                                                .show();
                                        });
                                    if permissions_btn.clicked {
                                        state.ai_manual_permissions = !state.ai_manual_permissions;
                                        state.status_message = if state.ai_manual_permissions {
                                            "Manual permissions enabled".to_string()
                                        } else {
                                            "Auto permissions enabled".to_string()
                                        };
                                        ui.needs_redraw = true;
                                    }
                                });
                        });
                });
        });

    // ── Floating Context Menu (+ popup) ──────────────────────────
    if state.ai_context_menu_open {
        let menu_w = 185.0;
        let menu_h = 225.0;
        let menu_x = (splitter_x + 8.0).clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
        let menu_y = (splitter_y + height - bottom_input_total_h - menu_h - 4.0).max(splitter_y + 40.0);

        let mouse_in_menu = ui.mouse_in_rect(menu_x, menu_y, menu_w, menu_h);
        let mouse_in_add_btn = ui.mouse_in_rect(splitter_x + 8.0, splitter_y + height - 36.0, 32.0, 32.0);
        let mut close_menu = false;
        if (ui.clicked || ui.right_clicked) && !mouse_in_menu && !mouse_in_add_btn {
            close_menu = true;
        }

        ui.overlay(|ui| {
            ui.container()
                .raw_id(Id::from_u64(999998610))
                .absolute(menu_x, menu_y)
                .overlay()
                .width(menu_w)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .radius_all(6.0)
                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, 4.0, 16.0)
                .padding(4.0, 4.0, 4.0, 4.0)
                .column()
                .gap(1.0)
                .show(|ui| {
                    if draw_ctx_menu_item(ui, Id::from_u64(999998611), "\u{f15c}", "Files & Directories", "", &colors) {
                        if !state.ai_input.is_empty() && !state.ai_input.ends_with(' ') {
                            state.ai_input.push(' ');
                        }
                        state.ai_input.push_str("@file ");
                        close_menu = true;
                    }
                    if draw_ctx_menu_item(ui, Id::from_u64(999998612), "</>", "Symbols", "", &colors) {
                        if !state.ai_input.is_empty() && !state.ai_input.ends_with(' ') {
                            state.ai_input.push(' ');
                        }
                        state.ai_input.push_str("@symbol ");
                        close_menu = true;
                    }
                    if draw_ctx_menu_item(ui, Id::from_u64(999998613), "\u{f086}", "Threads", "", &colors) {
                        state.ai_showing_session_list = true;
                        close_menu = true;
                    }
                    if draw_ctx_menu_item(ui, Id::from_u64(999998614), "\u{ea7f}", "Skills", "\u{f105}", &colors) {
                        if !state.ai_input.is_empty() && !state.ai_input.ends_with(' ') {
                            state.ai_input.push(' ');
                        }
                        state.ai_input.push_str("/skill ");
                        close_menu = true;
                    }
                    if draw_ctx_menu_item(ui, Id::from_u64(999998615), "\u{f03e}", "Image", "", &colors) {
                        if !state.ai_input.is_empty() && !state.ai_input.ends_with(' ') {
                            state.ai_input.push(' ');
                        }
                        state.ai_input.push_str("@image ");
                        close_menu = true;
                    }
                    if draw_ctx_menu_item(ui, Id::from_u64(999998616), "\u{f036}", "Selection", "", &colors) {
                        if let Some(idx) = state.active_tab {
                            if let Some(tab) = state.tabs.get(idx) {
                                let sel = tab.editor.selected_text();
                                if !sel.trim().is_empty() {
                                    state.ai_input.push_str(&format!("\n```\n{}\n```\n", sel));
                                } else {
                                    state.ai_input.push_str(&format!("\n`{}`\n", tab.name));
                                }
                            }
                        }
                        close_menu = true;
                    }
                    if draw_ctx_menu_item(ui, Id::from_u64(999998617), "\u{f126}", "Branch Diff", "", &colors) {
                        if let Ok(out) = std::process::Command::new("git").args(["diff", "HEAD"]).current_dir(&state.current_dir).output() {
                            let diff_str = String::from_utf8_lossy(&out.stdout);
                            if !diff_str.trim().is_empty() {
                                let sample: String = diff_str.chars().take(2000).collect();
                                state.ai_input.push_str(&format!("\n```diff\n{}\n```\n", sample));
                            }
                        }
                        close_menu = true;
                    }
                });
        });

        if close_menu {
            state.ai_context_menu_open = false;
            ui.interaction_state.insert(Id::from_u64((999998610 << 8) | 99), 0.0);
            ui.request_redraw();
        }
    }

    // ── Floating Model Selection Menu ───────────────────────────
    if state.ai_model_menu_open {
        let menu_w = 230.0;
        let menu_h = if state.adding_custom_model { 320.0 } else { 290.0 };
        let menu_x = (splitter_x + ai_panel_w - menu_w - 8.0).clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
        let menu_y = (splitter_y + height - bottom_input_total_h - menu_h - 4.0).max(splitter_y + 40.0);

        let mouse_in_menu = ui.mouse_in_rect(menu_x, menu_y, menu_w, menu_h);
        let mouse_in_model_btn = ui.mouse_in_rect(splitter_x + ai_panel_w - 140.0, splitter_y + height - 36.0, 140.0, 32.0);
        let mut close_menu = false;
        if (ui.clicked || ui.right_clicked) && !mouse_in_menu && !mouse_in_model_btn {
            close_menu = true;
        }

        ui.overlay(|ui| {
            ui.container()
                .raw_id(Id::from_u64(999998620))
                .absolute(menu_x, menu_y)
                .overlay()
                .width(menu_w)
                .bg(colors.bg_sidebar)
                .border(colors.border, 1.0)
                .radius_all(6.0)
                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, 4.0, 16.0)
                .padding(6.0, 6.0, 6.0, 6.0)
                .column()
                .gap(2.0)
                .show(|ui| {
                    // 1. Auto toggle item
                    let auto_resp = ui.container()
                        .id(Id::from_u64(999998621))
                        .full_width()
                        .height(28.0)
                        .radius_all(4.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                        .padding_left(8.0)
                        .padding_right(8.0)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .show(|ui| {
                            ui.container()
                                .row()
                                .gap(6.0)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f5dc}")
                                        .size(11.0)
                                        .color(if state.model_auto_mode { colors.accent } else { colors.text_muted })
                                        .show();
                                    ui.text("Auto")
                                        .size(11.5)
                                        .weight(FontWeight::Medium)
                                        .color(colors.text_primary)
                                        .show();
                                });

                            ui.container()
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .radius_all(3.0)
                                .bg(if state.model_auto_mode { colors.accent.with_alpha(0.2) } else { colors.highlight })
                                .border(if state.model_auto_mode { colors.accent } else { colors.border }, 1.0)
                                .show(|ui| {
                                    ui.text(if state.model_auto_mode { "ON" } else { "OFF" })
                                        .size(9.0)
                                        .weight(FontWeight::Bold)
                                        .color(if state.model_auto_mode { colors.accent } else { colors.text_dim })
                                        .show();
                                });
                        });

                    if auto_resp.clicked {
                        state.model_auto_mode = !state.model_auto_mode;
                        if state.model_auto_mode {
                            state.selected_model = "Auto".to_string();
                            close_menu = true;
                        }
                        ui.needs_redraw = true;
                    }

                    // Divider
                    ui.container().full_width().height(1.0).bg(colors.border).show(|_| {});

                    // Header
                    ui.container()
                        .padding_left(8.0)
                        .padding_top(2.0)
                        .padding_bottom(2.0)
                        .show(|ui| {
                            ui.text("AVAILABLE MODELS")
                                .size(9.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_dim)
                                .show();
                        });

                    let models: &[(&str, &str, &str)] = &[
                        ("gemini-3.5-flash", "Gemini 3.5 Flash", "Google"),
                        ("gemini-3.1-flash-lite", "Gemini Flash Lite", "Google"),
                        ("openai/gpt-oss-120b", "GPT-OSS 120B", "Groq"),
                        ("qwen/qwen3.8-27b", "Qwen 3.8 27B", "Groq"),
                        ("gemini-3.7-flash", "Gemini 3.7 Flash", "Google"),
                        ("claude-3-7-sonnet", "Claude 3.7 Sonnet", "Anthropic"),
                        ("deepseek-r1", "DeepSeek R1", "Reasoning"),
                    ];

                    for (m_idx, (m_id, m_name, m_provider)) in models.iter().enumerate() {
                        let is_active = !state.model_auto_mode && state.selected_model == *m_id;
                        let m_resp = ui.container()
                            .id(Id::from_u64(999998630 + m_idx as u64))
                            .full_width()
                            .height(26.0)
                            .radius_all(4.0)
                            .bg(Color::TRANSPARENT)
                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                            .padding_left(8.0)
                            .padding_right(8.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container()
                                    .row()
                                    .gap(6.0)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text(m_name)
                                            .size(11.0)
                                            .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_active { colors.accent } else { colors.text_primary })
                                            .show();
                                        ui.text(m_provider)
                                            .size(9.0)
                                            .color(colors.text_dim)
                                            .show();
                                    });

                                if is_active {
                                    ui.text("\u{f00c}")
                                        .size(10.0)
                                        .color(colors.accent)
                                        .show();
                                }
                            });

                        if m_resp.clicked {
                            state.selected_model = m_id.to_string();
                            state.model_auto_mode = false;
                            close_menu = true;
                        }
                    }

                    for (c_idx, c_model) in state.custom_models.iter().enumerate() {
                        let is_active = !state.model_auto_mode && state.selected_model == *c_model;
                        let c_resp = ui.container()
                            .id(Id::from_u64(999998660 + c_idx as u64))
                            .full_width()
                            .height(26.0)
                            .radius_all(4.0)
                            .bg(Color::TRANSPARENT)
                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                            .padding_left(8.0)
                            .padding_right(8.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.text(c_model)
                                    .size(11.0)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active { colors.accent } else { colors.text_primary })
                                    .show();

                                if is_active {
                                    ui.text("\u{f00c}")
                                        .size(10.0)
                                        .color(colors.accent)
                                        .show();
                                }
                            });

                        if c_resp.clicked {
                            state.selected_model = c_model.clone();
                            state.model_auto_mode = false;
                            close_menu = true;
                        }
                    }

                    // Divider
                    ui.container().full_width().height(1.0).bg(colors.border).show(|_| {});

                    // Add Model Section
                    if !state.adding_custom_model {
                        let add_btn = ui.container()
                            .id(Id::from_u64(999998680))
                            .full_width()
                            .height(26.0)
                            .radius_all(4.0)
                            .bg(Color::TRANSPARENT)
                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                            .padding_left(8.0)
                            .padding_right(8.0)
                            .row()
                            .gap(6.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("+")
                                    .size(12.0)
                                    .color(colors.accent)
                                    .show();
                                ui.text("Add Model...")
                                    .size(10.5)
                                    .color(colors.text_primary)
                                    .show();
                            });

                        if add_btn.clicked {
                            state.adding_custom_model = true;
                            ui.needs_redraw = true;
                        }
                    } else {
                        ui.container()
                            .full_width()
                            .padding(2.0, 4.0, 2.0, 4.0)
                            .row()
                            .gap(4.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.input(&mut state.custom_model_input, Id::from_u64(999998685))
                                    .width(140.0)
                                    .min_width(60.0)
                                    .size(10.5)
                                    .color(colors.text_primary)
                                    .bg(colors.bg_base)
                                    .border(colors.accent, 1.0)
                                    .radius_all(3.0)
                                    .padding_x(4.0)
                                    .padding_y(2.0)
                                    .show();

                                let save_btn = ui.button("Add")
                                    .id(Id::from_u64(999998681))
                                    .width(32.0)
                                    .height(22.0)
                                    .bg(colors.accent)
                                    .text_color(Color::WHITE)
                                    .radius_all(3.0)
                                    .padding(0.0, 0.0, 0.0, 0.0)
                                    .size(9.5)
                                    .show();

                                if save_btn.clicked {
                                    let m = state.custom_model_input.trim().to_string();
                                    if !m.is_empty() {
                                        state.custom_models.push(m.clone());
                                        state.selected_model = m;
                                        state.model_auto_mode = false;
                                        state.custom_model_input.clear();
                                        state.adding_custom_model = false;
                                        close_menu = true;
                                    }
                                }

                                let cancel_btn = ui.container()
                                    .id(Id::from_u64(999998682))
                                    .width(20.0)
                                    .height(22.0)
                                    .radius_all(3.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                                    .halign(Align::Center)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text("\u{f00d}")
                                            .size(10.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });

                                if cancel_btn.clicked {
                                    state.adding_custom_model = false;
                                    state.custom_model_input.clear();
                                    ui.needs_redraw = true;
                                }
                            });
                    }
                });
        });

        if close_menu {
            state.ai_model_menu_open = false;
            ui.interaction_state.insert(Id::from_u64((999998620 << 8) | 99), 0.0);
            ui.request_redraw();
        }
    }

    if panel_container.clicked && ui.focused_id != Some(ai_input_id) {
        ui.focused_id = Some(ai_id);
        ui.needs_redraw = true;
    }
}

fn get_file_glyph(name: &str) -> &'static str {
    if name.ends_with(".rs") {
        "\u{e7a8}"
    } else if name.ends_with(".ts") || name.ends_with(".js") || name.ends_with(".tsx") || name.ends_with(".jsx") {
        "\u{e628}"
    } else if name.ends_with(".toml") || name.ends_with(".json") || name.ends_with(".yaml") || name.ends_with(".yml") {
        "\u{e60b}"
    } else if name.ends_with(".md") || name.ends_with(".txt") {
        "\u{f15c}"
    } else if name.ends_with(".py") {
        "\u{e73c}"
    } else if name.ends_with(".html") || name.ends_with(".css") {
        "\u{e736}"
    } else {
        "\u{ea7f}"
    }
}

fn friendly_model_name(id: &str) -> &str {
    match id {
        "gemini-3.5-flash" => "Gemini 3.5 Flash",
        "gemini-3.1-flash-lite" => "Gemini Flash Lite",
        "gemini-flash-lite-latest" => "Gemini Flash Lite",
        "gemini-2.5-flash" => "Gemini 3.5 Flash",
        "openai/gpt-oss-120b" => "GPT-OSS 120B",
        "openai/gpt-oss-20b" => "GPT-OSS 20B",
        "qwen/qwen3.8-27b" => "Qwen 3.8 27B",
        "gemini-3.7-flash" => "Gemini 3.7 Flash",
        "claude-3-7-sonnet" => "Claude 3.7 Sonnet",
        "deepseek-r1" => "DeepSeek R1",
        other => other,
    }
}

fn draw_ctx_menu_item(
    ui: &mut Ui,
    id: Id,
    icon: &str,
    label: &str,
    right_icon: &str,
    colors: &crate::theme::ThemeColors,
) -> bool {
    let resp = ui.container()
        .id(id)
        .full_width()
        .height(26.0)
        .radius_all(4.0)
        .bg(Color::TRANSPARENT)
        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
        .padding_left(8.0)
        .padding_right(8.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .row()
                .gap(8.0)
                .valign(Align::Center)
                .show(|ui| {
                    ui.container()
                        .width(14.0)
                        .halign(Align::Center)
                        .show(|ui| {
                            ui.text(icon)
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    ui.text(label)
                        .size(11.5)
                        .color(colors.text_primary)
                        .show();
                });

            if !right_icon.is_empty() {
                ui.text(right_icon)
                    .size(9.0)
                    .color(colors.text_dim)
                    .show();
            }
        });

    resp.clicked
}

