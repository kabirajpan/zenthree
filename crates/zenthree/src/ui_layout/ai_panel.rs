use crate::state::{ZenthreeState, DiffLineState, EditedFileSummary, TaskStatus, AiInputDrawer};
use crate::session::format_relative_time;
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    if state.poll_ai_responses() {
        ui.request_redraw();
    }
    let colors = zenthra::use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let agent_ctx = zenthra::use_context::<crate::state::AgentDomainState>();
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
    let mut model_btn_just_clicked = false;
    let mut add_ctx_btn_just_clicked = false;

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
                    // Left side: Left arrow + text heading "AI Assistant" (no emoji or icon)
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            let back_btn = ui.container()
                                .width(20.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f053}")
                                        .size(10.0)
                                        .weight(FontWeight::Bold)
                                        .color(if state.ai_showing_session_list { colors.accent } else { colors.text_muted })
                                        .show();
                                });
                            if back_btn.clicked {
                                if !state.ai_showing_session_list {
                                    state.session_list = crate::session::list_saved_sessions(&state.current_dir);
                                    state.ai_showing_session_list = true;
                                } else {
                                    state.ai_showing_session_list = false;
                                }
                                ui.needs_redraw = true;
                            }

                            let backend_btn = ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .radius_all(4.0)
                                .hover_bg(colors.highlight)
                                .show(|ui| {
                                    let (agent_name, badge) = match state.ai_backend {
                                        crate::agent::AgentBackend::Native => ("ZENE", "Native"),
                                        crate::agent::AgentBackend::GooseAcp => ("Goose", "ACP"),
                                    };

                                    ui.text(agent_name)
                                        .size(12.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.text_primary)
                                        .show();

                                    ui.text(&format!("({})", badge))
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if backend_btn.clicked {
                                state.ai_backend = match state.ai_backend {
                                    crate::agent::AgentBackend::Native => crate::agent::AgentBackend::GooseAcp,
                                    crate::agent::AgentBackend::GooseAcp => crate::agent::AgentBackend::Native,
                                };
                                ui.needs_redraw = true;
                            }

                            // Direct manual model switcher pill in header
                            let hdr_model_name = if state.model_auto_mode {
                                "Auto"
                            } else if !state.selected_model.is_empty() {
                                friendly_model_name(&state.selected_model)
                            } else {
                                "Model"
                            };
                            let (hdr_dot_c, _) = if state.last_provider_notice.is_some() {
                                (Color::rgb(0.95, 0.55, 0.20), "●")
                            } else if state.selected_model.contains("gemini") {
                                (Color::rgb(0.95, 0.75, 0.35), "●")
                            } else {
                                (Color::rgb(0.35, 0.85, 0.45), "●")
                            };
                            let hdr_model_btn = ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(3.0)
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .radius_all(4.0)
                                .bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                .hover_bg(colors.highlight)
                                .show(|ui| {
                                    ui.text("●")
                                        .size(6.0)
                                        .color(hdr_dot_c)
                                        .show();
                                    ui.text(hdr_model_name)
                                        .size(9.5)
                                        .weight(FontWeight::Medium)
                                        .color(colors.text_muted)
                                        .show();
                                    ui.text("\u{f107}")
                                        .size(7.5)
                                        .color(colors.text_dim)
                                        .show();
                                });

                            if hdr_model_btn.clicked {
                                state.ai_model_menu_open = !state.ai_model_menu_open;
                                state.ai_context_menu_open = false;
                                model_btn_just_clicked = true;
                                ui.needs_redraw = true;
                            } else if ui.right_clicked && hdr_model_btn.hovered {
                                let cycle = [
                                    ("qwen/qwen3.8-27b", "Qwen 3.8 27B"),
                                    ("openai/gpt-oss-120b", "GPT-OSS 120B"),
                                    ("gemini-3.6-flash", "Gemini 3.6 Flash"),
                                    ("gemini-2.5-pro", "Gemini 2.5 Pro"),
                                    ("claude-3-7-sonnet", "Claude 3.7 Sonnet"),
                                ];
                                let curr = &state.selected_model;
                                let next_idx = cycle.iter().position(|(id, _)| id == curr)
                                    .map(|idx| (idx + 1) % cycle.len())
                                    .unwrap_or(0);
                                let (next_id, next_name) = cycle[next_idx];
                                state.selected_model = next_id.to_string();
                                state.model_auto_mode = false;
                                state.last_provider_notice = None;
                                state.status_message = format!("Switched model to {}", next_name);
                                ui.needs_redraw = true;
                            }
                        });

                    // Right side: + icon and cross icon to close
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            // + icon to start new session
                            let new_session_btn = ui.container()
                                .width(20.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(colors.highlight)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("+")
                                        .size(12.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if new_session_btn.clicked {
                                state.start_new_session();
                                state.ai_showing_session_list = false;
                                ui.needs_redraw = true;
                            }

                            // Cross icon to close AI panel
                            let close_btn = ui.container()
                                .width(20.0)
                                .height(20.0)
                                .radius_all(3.0)
                                .hover_bg(Color::rgba(0.9, 0.2, 0.2, 0.15))
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("✕")
                                        .size(10.5)
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

            // ─── Bottom Prompt Input Height & Messages Scroll Area ─────
            let ai_input_id = ui.id_from("ai_prompt_text_area");
            let is_input_focused = ui.focused_id == Some(ai_input_id);

            const CONTEXT_BANNER_H: f32 = 0.0;
            const TOOLBAR_H: f32 = 28.0;

            let drawer_h = match state.active_input_drawer {
                AiInputDrawer::None => 0.0,
                AiInputDrawer::PendingModifications | AiInputDrawer::SessionChanges => {
                    let count = state.pending_modifications.len()
                        .max(if state.pending_modification.is_some() { 1 } else { 0 })
                        .max(state.session_modified_files.len());
                    if count > 0 {
                        let file_list_h = (count as f32 * 28.0 + 8.0).min(130.0);
                        file_list_h + 32.0
                    } else {
                        0.0
                    }
                }
                AiInputDrawer::Terminals => {
                    let running_count = state.ai_terminal_commands.iter().filter(|c| c.status == crate::state::AiCommandStatus::Running).count();
                    if running_count > 0 {
                        (running_count as f32 * 26.0 + 8.0).clamp(34.0, 130.0)
                    } else {
                        0.0
                    }
                }
                AiInputDrawer::Artifacts => {
                    let arts = state.current_session.list_artifacts();
                    if !arts.is_empty() {
                        (arts.len() as f32 * 26.0 + 8.0).clamp(34.0, 130.0)
                    } else {
                        0.0
                    }
                }
            };

            let quota_banner_h = if state.last_provider_notice.is_some() { 26.0 } else { 20.0 };
            let msgs_h = (height - HEADER_H - CONTEXT_BANNER_H - bottom_input_total_h - TOOLBAR_H - quota_banner_h - drawer_h - 12.0).max(60.0);
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
                    let mut toggle_accordion_idx: Option<usize> = None;
                    let mut toggle_explored_folder_target: Option<(usize, String)> = None;
                    let mut toggle_thought_key: Option<(usize, usize)> = None;
                    let mut jump_target: Option<(String, Option<String>)> = None;
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
                                        .padding(9.0, 14.0, 9.0, 14.0)
                                        .bg(match state.theme {
                                            crate::theme::ThemeMode::Dark => Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.06),
                                            crate::theme::ThemeMode::Light => Color::rgba(0.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0, 0.05),
                                        })
                                        .border(colors.border, 1.0)
                                        .radius_all(16.0)
                                        .show(|ui| {
                                            ui.text(&msg.content)
                                                .size(11.5)
                                                .line_height(1.48)
                                                .color(colors.text_primary)
                                                .max_width(user_text_w)
                                                .wrap(zenthra_text::prelude::TextWrap::Word)
                                                .selectable(true)
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
                                                    original_code: None,
                                                    proposed_code: None,
                                                });
                                            }
                                        }
                                    }

                                    let mut chat_reject_single = None;

                                    // Top-level Cognitive Work / Exploration Accordion
                                    let has_work_activity = msg.is_working
                                        || !msg.explored_items.is_empty()
                                        || !files_to_show.is_empty()
                                        || msg.thoughts.is_some();

                                    if has_work_activity {
                                        let file_count = msg.explored_items.iter()
                                            .filter(|i| i.action_type == "Analyzed" && (i.line_range.is_some() || i.file_name.contains('.')))
                                            .count();
                                        let folder_count = msg.explored_items.iter()
                                            .filter(|i| i.action_type == "Analyzed" && i.line_range.is_none() && !i.file_name.contains('.') && !i.file_name.is_empty())
                                            .count();
                                        let search_count = msg.explored_items.iter().filter(|i| i.action_type == "Searched").count();

                                        let is_exp_open = if msg.is_working {
                                            !state.expanded_explored_msgs.contains(&msg_idx)
                                        } else {
                                            state.expanded_explored_msgs.contains(&msg_idx)
                                        };
                                        let exp_arrow = if is_exp_open { "\u{f078}" } else { "\u{f054}" };

                                        let verb = if msg.is_working { "Exploring" } else { "Explored" };
                                        let header_title = if file_count > 0 && folder_count > 0 {
                                            format!("{} {} file{}, {} folder{}", verb, file_count, if file_count == 1 { "" } else { "s" }, folder_count, if folder_count == 1 { "" } else { "s" })
                                        } else if file_count > 0 {
                                            format!("{} {} file{}", verb, file_count, if file_count == 1 { "" } else { "s" })
                                        } else if folder_count > 0 {
                                            format!("{} {} folder{}", verb, folder_count, if folder_count == 1 { "" } else { "s" })
                                        } else if search_count > 0 {
                                            format!("{} {} search{}", if msg.is_working { "Searching" } else { "Searched" }, search_count, if search_count == 1 { "" } else { "es" })
                                        } else if msg.is_working {
                                            "Exploring workspace".to_string()
                                        } else if let Some(secs) = msg.elapsed_secs {
                                            format!("Worked for {}s", secs.max(1))
                                        } else {
                                            "Explored workspace".to_string()
                                        };

                                        let exp_btn = ui.container()
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                            .radius_all(3.0)
                                            .row()
                                            .valign(Align::Center)
                                            .gap(5.0)
                                            .show(|ui| {
                                                ui.container()
                                                    .width(10.0)
                                                    .valign(Align::Center)
                                                    .halign(Align::Center)
                                                    .show(|ui| {
                                                        ui.text(exp_arrow)
                                                            .size(7.0)
                                                            .color(colors.text_muted)
                                                            .show();
                                                    });
                                                ui.text(&header_title)
                                                    .size(10.5)
                                                    .weight(FontWeight::Medium)
                                                    .color(colors.text_dim)
                                                    .show();
                                            });

                                        if exp_btn.clicked {
                                            toggle_accordion_idx = Some(msg_idx);
                                        }

                                        if is_exp_open {
                                            ui.container()
                                                .padding_left(14.0)
                                                .column()
                                                .gap(3.0)
                                                .show(|ui| {
                                                    // 1. Chronological event progression (Thoughts and Analyzed items interleaved)
                                                    for (item_idx, item) in msg.explored_items.iter().enumerate() {
                                                        if item.action_type == "Thought" || item.thought_content.is_some() {
                                                            let thought_text = item.thought_content.as_deref().unwrap_or("");
                                                            let thought_dur = item.duration_secs.unwrap_or(1);
                                                            let is_thought_exp = state.expanded_thought_items.contains(&(msg_idx, item_idx));
                                                            let t_arrow = if is_thought_exp { "\u{f078}" } else { "\u{f054}" };

                                                            let t_btn = ui.container()
                                                                .padding(1.5, 6.0, 1.5, 6.0)
                                                                .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.05))
                                                                .radius_all(3.0)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(5.0)
                                                                .show(|ui| {
                                                                    ui.container()
                                                                        .width(8.0)
                                                                        .valign(Align::Center)
                                                                        .halign(Align::Center)
                                                                        .show(|ui| {
                                                                            ui.text(t_arrow)
                                                                                .size(6.5)
                                                                                .color(colors.text_muted)
                                                                                .show();
                                                                        });
                                                                    ui.text(&format!("Thought for {}s", thought_dur))
                                                                        .size(9.5)
                                                                        .weight(FontWeight::Medium)
                                                                        .color(colors.text_muted)
                                                                        .show();
                                                                });

                                                            if t_btn.clicked {
                                                                toggle_thought_key = Some((msg_idx, item_idx));
                                                            }

                                                            if is_thought_exp && !thought_text.is_empty() {
                                                                ui.container()
                                                                    .padding_left(14.0)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .padding(5.0, 9.0, 5.0, 9.0)
                                                                            .bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.03))
                                                                            .radius_all(4.0)
                                                                            .show(|ui| {
                                                                                ui.text(thought_text)
                                                                                    .size(9.5)
                                                                                    .line_height(1.3)
                                                                                    .color(colors.text_dim)
                                                                                    .max_width(assistant_text_w - 32.0)
                                                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                                    .show();
                                                                            });
                                                                    });
                                                            }
                                                        } else if item.action_type == "Analyzed" {
                                                            let is_dir = item.line_range.is_none() && (!item.file_name.contains('.') || item.file_name.ends_with('/'));
                                                            let display_name = std::path::Path::new(&item.file_name)
                                                                .file_name()
                                                                .and_then(|n| n.to_str())
                                                                .unwrap_or(&item.file_name);

                                                            if is_dir {
                                                                let is_folder_expanded = state.is_explored_folder_expanded(msg_idx, &item.file_name);
                                                                let f_arrow = if is_folder_expanded { "\u{f078}" } else { "\u{f054}" };

                                                                let folder_row = ui.container()
                                                                    .padding(1.5, 6.0, 1.5, 6.0)
                                                                    .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                                                    .radius_all(3.0)
                                                                    .row()
                                                                    .valign(Align::Center)
                                                                    .gap(5.0)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .width(8.0)
                                                                            .valign(Align::Center)
                                                                            .halign(Align::Center)
                                                                            .show(|ui| {
                                                                                ui.text(f_arrow)
                                                                                    .size(6.5)
                                                                                    .color(colors.text_muted)
                                                                                    .show();
                                                                            });
                                                                        ui.text("Analyzed")
                                                                            .size(10.0)
                                                                            .color(colors.text_muted)
                                                                            .show();
                                                                        ui.text("\u{f07b}")
                                                                            .size(10.0)
                                                                            .color(Color::rgb(0.35, 0.60, 0.95))
                                                                            .show();
                                                                        ui.text(display_name)
                                                                            .size(10.0)
                                                                            .weight(FontWeight::Medium)
                                                                            .color(colors.text_primary)
                                                                            .show();
                                                                    });

                                                                if folder_row.clicked {
                                                                    toggle_explored_folder_target = Some((msg_idx, item.file_name.clone()));
                                                                }

                                                                if is_folder_expanded && !item.children.is_empty() {
                                                                    ui.container()
                                                                        .padding_left(16.0)
                                                                        .column()
                                                                        .gap(2.0)
                                                                        .show(|ui| {
                                                                            for child in &item.children {
                                                                                let (c_glyph, c_color) = get_file_glyph_and_color(child, &colors);
                                                                                let c_name = child.trim_end_matches('/');
                                                                                ui.container()
                                                                                    .padding(1.0, 4.0, 1.0, 4.0)
                                                                                    .row()
                                                                                    .valign(Align::Center)
                                                                                    .gap(5.0)
                                                                                    .show(|ui| {
                                                                                        ui.text("•")
                                                                                            .size(7.0)
                                                                                            .color(colors.text_muted)
                                                                                            .show();
                                                                                        ui.text(c_glyph)
                                                                                            .size(9.5)
                                                                                            .color(c_color)
                                                                                            .show();
                                                                                        ui.text(c_name)
                                                                                            .size(9.5)
                                                                                            .color(colors.text_dim)
                                                                                            .show();
                                                                                    });
                                                                            }
                                                                        });
                                                                }
                                                            } else {
                                                                let (glyph, icon_color) = get_file_glyph_and_color(&item.file_name, &colors);

                                                                let file_row = ui.container()
                                                                    .padding(1.5, 6.0, 1.5, 6.0)
                                                                    .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                                                    .radius_all(3.0)
                                                                    .row()
                                                                    .valign(Align::Center)
                                                                    .gap(6.0)
                                                                    .show(|ui| {
                                                                        ui.text("Analyzed")
                                                                            .size(10.0)
                                                                            .color(colors.text_muted)
                                                                            .show();

                                                                        ui.text(glyph)
                                                                            .size(10.0)
                                                                            .color(icon_color)
                                                                            .show();

                                                                        ui.text(display_name)
                                                                            .size(10.0)
                                                                            .weight(FontWeight::Medium)
                                                                            .color(colors.text_primary)
                                                                            .show();

                                                                        if let Some(ref range) = item.line_range {
                                                                            ui.text(range)
                                                                                .size(9.5)
                                                                                .color(colors.text_muted)
                                                                                .show();
                                                                        }
                                                                    });

                                                                if file_row.clicked {
                                                                    jump_target = Some((item.file_name.clone(), item.line_range.clone()));
                                                                }
                                                            }
                                                        } else if item.action_type == "Searched" {
                                                            ui.container()
                                                                .padding(1.5, 6.0, 1.5, 6.0)
                                                                .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                                                .radius_all(3.0)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(6.0)
                                                                .show(|ui| {
                                                                    ui.text("Searched")
                                                                        .size(10.0)
                                                                        .color(colors.text_muted)
                                                                        .show();

                                                                    ui.text("\u{f002}")
                                                                        .size(9.5)
                                                                        .color(colors.accent)
                                                                        .show();

                                                                    ui.text(&item.file_name)
                                                                        .size(10.0)
                                                                        .weight(FontWeight::Medium)
                                                                        .color(colors.text_primary)
                                                                        .show();

                                                                    if let Some(ref range) = item.line_range {
                                                                        ui.text(range)
                                                                            .size(9.5)
                                                                            .color(colors.text_muted)
                                                                            .show();
                                                                    }
                                                                });
                                                        }
                                                    }

                                                    // Fallback thought if explored_items has no thoughts but msg.thoughts exists
                                                    if !msg.explored_items.iter().any(|i| i.action_type == "Thought" || i.thought_content.is_some()) {
                                                        if let Some(ref thought_text) = msg.thoughts {
                                                            let is_thought_exp = state.expanded_thought_items.contains(&(msg_idx, 999999));
                                                            let t_arrow = if is_thought_exp { "\u{f078}" } else { "\u{f054}" };

                                                            let t_btn = ui.container()
                                                                .padding(1.5, 6.0, 1.5, 6.0)
                                                                .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.05))
                                                                .radius_all(3.0)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(5.0)
                                                                .show(|ui| {
                                                                    ui.container()
                                                                        .width(8.0)
                                                                        .valign(Align::Center)
                                                                        .halign(Align::Center)
                                                                        .show(|ui| {
                                                                            ui.text(t_arrow)
                                                                                .size(6.5)
                                                                                .color(colors.text_muted)
                                                                                .show();
                                                                        });
                                                                    ui.text("Reasoning")
                                                                        .size(9.5)
                                                                        .weight(FontWeight::Medium)
                                                                        .color(colors.text_muted)
                                                                        .show();
                                                                });

                                                            if t_btn.clicked {
                                                                toggle_thought_key = Some((msg_idx, 999999));
                                                            }

                                                            if is_thought_exp && !thought_text.trim().is_empty() {
                                                                ui.container()
                                                                    .padding_left(14.0)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .padding(5.0, 9.0, 5.0, 9.0)
                                                                            .bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.03))
                                                                            .radius_all(4.0)
                                                                            .show(|ui| {
                                                                                ui.text(thought_text)
                                                                                    .size(9.5)
                                                                                    .line_height(1.3)
                                                                                    .color(colors.text_dim)
                                                                                    .max_width(assistant_text_w - 32.0)
                                                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                                    .show();
                                                                            });
                                                                    });
                                                            }
                                                        }
                                                    }

                                                    // 2. Edited files badges
                                                    for edited in &files_to_show {
                                                        let glyph = get_file_glyph(&edited.file_name);
                                                        let target_name = edited.file_name.clone();

                                                        let p = std::path::Path::new(&edited.file_name);
                                                        let file_name = p.file_name().and_then(|n| n.to_str()).unwrap_or(&edited.file_name).to_string();

                                                        let pending_match = state.pending_modifications.iter().find(|p| {
                                                            p.file_path.ends_with(&edited.file_name)
                                                                || p.file_path.file_name().and_then(|n| n.to_str()) == Some(&edited.file_name)
                                                        }).or_else(|| {
                                                            state.pending_modification.as_ref().filter(|p| {
                                                                p.file_path.ends_with(&edited.file_name)
                                                                    || p.file_path.file_name().and_then(|n| n.to_str()) == Some(&edited.file_name)
                                                            })
                                                        });

                                                        let edit_btn = ui.container()
                                                            .padding(2.0, 6.0, 2.0, 6.0)
                                                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
                                                            .radius_all(3.0)
                                                            .row()
                                                            .valign(Align::Center)
                                                            .gap(6.0)
                                                            .show(|ui| {
                                                                ui.text("Edited")
                                                                    .size(10.0)
                                                                    .color(colors.text_muted)
                                                                    .show();

                                                                ui.text(glyph)
                                                                    .size(10.0)
                                                                    .color(colors.accent)
                                                                    .show();

                                                                ui.text(&file_name)
                                                                    .size(10.0)
                                                                    .weight(FontWeight::Medium)
                                                                    .color(colors.text_primary)
                                                                    .show();

                                                                ui.container()
                                                                    .row()
                                                                    .gap(4.0)
                                                                    .valign(Align::Center)
                                                                    .show(|ui| {
                                                                        if edited.lines_added > 0 {
                                                                            ui.text(&format!("+{}", edited.lines_added))
                                                                                .size(9.0)
                                                                                .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                                                .show();
                                                                        }
                                                                        if edited.lines_removed > 0 {
                                                                            ui.text(&format!("-{}", edited.lines_removed))
                                                                                .size(9.0)
                                                                                .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                                                                .show();
                                                                        }
                                                                    });

                                                                if let Some(pending) = pending_match {
                                                                    let rej_btn = ui.container()
                                                                        .padding(1.0, 4.0, 1.0, 4.0)
                                                                        .radius_all(2.0)
                                                                        .hover_bg(Color::rgba(0.9, 0.2, 0.2, 0.15))
                                                                        .show(|ui| {
                                                                            ui.text("✕")
                                                                                .size(9.5)
                                                                                .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                                                                .show();
                                                                        });
                                                                    if rej_btn.clicked {
                                                                        chat_reject_single = Some(pending.file_path.clone());
                                                                    }
                                                                }
                                                            });

                                                        if edit_btn.clicked && chat_reject_single.is_none() {
                                                            open_file_target = Some(target_name);
                                                        }
                                                    }

                                                    // 3. Active Decision Sub-State Indicator (Working.. / Compacting. / Searching.. / Reading..)
                                                    if msg.is_working {
                                                        let now_ms = std::time::SystemTime::now()
                                                            .duration_since(std::time::UNIX_EPOCH)
                                                            .map(|d| d.as_millis())
                                                            .unwrap_or(0);
                                                        let dots_count = ((now_ms / 350) % 3) as usize;
                                                        let dots = match dots_count {
                                                            0 => ".",
                                                            1 => "..",
                                                            _ => "...",
                                                        };

                                                        let signal_status = agent_ctx.as_ref()
                                                            .and_then(|a| a.active_status.clone())
                                                            .or_else(|| state.ai_active_status.with(|s| s.clone()));
                                                        let active_status_str = signal_status.as_deref().or(msg.status_text.as_deref());
                                                        let base_status = match active_status_str {
                                                            Some("Searching") => "Searching",
                                                            Some("Compacting") => "Compacting",
                                                            Some("Reading") => "Reading",
                                                            Some("Exploring") => "Exploring",
                                                            Some("Editing") => "Editing",
                                                            Some(s) => s,
                                                            None => "Working",
                                                        };
                                                        let status_label = format!("{}{}", base_status, dots);

                                                        ui.container()
                                                            .padding(2.0, 6.0, 2.0, 6.0)
                                                            .row()
                                                            .valign(Align::Center)
                                                            .show(|ui| {
                                                                ui.text(&status_label)
                                                                    .size(10.0)
                                                                    .color(colors.text_dim)
                                                                    .show();
                                                            });
                                                        ui.needs_redraw = true;
                                                    }
                                                });
                                        }
                                    }

                                    if let Some(path) = chat_reject_single {
                                        state.reject_single_modification(&path);
                                        ui.needs_redraw = true;
                                    }

                                    // 3. Markdown formatted response content
                                    if !msg.content.trim().is_empty() {
                                        let is_quota = msg.content.contains("quota limit")
                                            || msg.content.contains("RESOURCE_EXHAUSTED")
                                            || msg.content.contains("429")
                                            || msg.content.contains("quota exceeded")
                                            || msg.content.contains("No capacity available");
                                        let is_goose_err = msg.content.contains("Goose ACP") || (msg.content.contains("Internal error") && state.ai_backend == crate::agent::AgentBackend::GooseAcp);
                                        let is_err_or_quota = msg.content.trim().starts_with("⚠️")
                                            || msg.content.contains("UNAVAILABLE")
                                            || is_quota
                                            || is_goose_err;

                                        if is_err_or_quota {
                                            let (title_text, sub_text) = if is_quota {
                                                ("AI Engine Service Notice", "Quota limit or rate limit reached. Auto-routed request to active provider.")
                                            } else if is_goose_err {
                                                ("Goose ACP Notice", "Local Goose engine reported an error. Click 'Goose (ACP)' in header to switch to Native (ZENE).")
                                            } else {
                                                ("AI Engine Notice", "Provider or engine reported an issue. Check model status or network.")
                                            };

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
                                                            ui.text(title_text)
                                                                .size(10.0)
                                                                .weight(FontWeight::Bold)
                                                                .color(Color::rgb(0.95, 0.65, 0.3))
                                                                .show();
                                                            ui.text(sub_text)
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
                                            &mut state.expanded_code_blocks,
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
                                                &mut state.expanded_code_blocks,
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

                                        // Action bar at bottom of assistant message (Copy Response)
                                        if !msg.is_working && !msg.content.trim().is_empty() {
                                            let copy_text = msg.content.clone();
                                            let is_copied = state.status_message.starts_with("Response copied") && state.status_message.contains(&format!("#{}", msg_idx));

                                            ui.container()
                                                .row()
                                                .gap(6.0)
                                                .valign(Align::Center)
                                                .padding(2.0, 0.0, 4.0, 0.0)
                                                .show(|ui| {
                                                    let copy_resp_btn = ui.container()
                                                        .padding(2.0, 6.0, 2.0, 6.0)
                                                        .radius_all(3.0)
                                                        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.05))
                                                        .row()
                                                        .gap(4.0)
                                                        .valign(Align::Center)
                                                        .show(|ui| {
                                                            if is_copied {
                                                                ui.text("\u{f00c}") // check icon
                                                                    .size(9.0)
                                                                    .color(Color::rgb(0.35, 0.85, 0.45))
                                                                    .show();
                                                                ui.text("Copied")
                                                                    .size(9.0)
                                                                    .color(Color::rgb(0.35, 0.85, 0.45))
                                                                    .show();
                                                            } else {
                                                                ui.text("\u{f0c5}") // copy icon
                                                                    .size(9.0)
                                                                    .color(colors.text_muted)
                                                                    .show();
                                                                ui.text("Copy")
                                                                    .size(9.0)
                                                                    .color(colors.text_muted)
                                                                    .show();
                                                            }
                                                        });

                                                    if copy_resp_btn.clicked {
                                                        state.set_clipboard(copy_text);
                                                        state.status_message = format!("Response copied #{}", msg_idx);
                                                        ui.needs_redraw = true;
                                                    }
                                                });
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
                    if let Some((msg_idx, folder_path)) = toggle_explored_folder_target {
                        state.toggle_explored_folder(msg_idx, &folder_path);
                        ui.needs_redraw = true;
                    }
                    if let Some((msg_idx, item_idx)) = toggle_thought_key {
                        state.toggle_thought_accordion(msg_idx, item_idx);
                        ui.needs_redraw = true;
                    }
                    if let Some((file, range)) = jump_target {
                        state.jump_to_file_and_line(&file, range.as_deref());
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

            // ─── Interactive Permission Request Card ───────────────────────────
            if let Some(perm) = state.ai_pending_permissions.first().cloned() {
                let perm_card_w = ai_panel_w - 16.0;
                let mut allow_once = false;
                let mut allow_always = false;
                let mut deny = false;

                ui.container()
                    .width(perm_card_w)
                    .padding(8.0, 10.0, 8.0, 10.0)
                    .bg(Color::rgba(245.0 / 255.0, 158.0 / 255.0, 11.0 / 255.0, 0.08))
                    .border(Color::rgba(245.0 / 255.0, 158.0 / 255.0, 11.0 / 255.0, 0.4), 1.0)
                    .radius_all(6.0)
                    .column()
                    .gap(6.0)
                    .show(|ui| {
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text("\u{f3ed}")
                                    .size(11.0)
                                    .color(Color::rgb(245.0 / 255.0, 158.0 / 255.0, 11.0 / 255.0))
                                    .show();
                                ui.text(&format!("Permission Request: {}", perm.tool_name))
                                    .size(10.5)
                                    .weight(FontWeight::SemiBold)
                                    .color(colors.text_primary)
                                    .show();
                            });

                        ui.text(&perm.reason)
                            .size(9.5)
                            .color(colors.text_muted)
                            .show();

                        if let Some(ref action) = perm.suggested_action {
                            ui.text(&format!("Action: {}", action))
                                .size(9.0)
                                .color(colors.text_muted)
                                .show();
                        }

                        ui.container()
                            .row()
                            .gap(6.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                let allow_btn = ui.container()
                                    .padding(2.5, 8.0, 2.5, 8.0)
                                    .bg(colors.accent)
                                    .radius_all(3.0)
                                    .show(|ui| {
                                        ui.text("Allow Once")
                                            .size(9.0)
                                            .weight(FontWeight::SemiBold)
                                            .color(Color::WHITE)
                                            .show();
                                    });
                                if allow_btn.clicked {
                                    allow_once = true;
                                }

                                let always_btn = ui.container()
                                    .padding(2.5, 8.0, 2.5, 8.0)
                                    .bg(colors.bg_base)
                                    .border(colors.border, 1.0)
                                    .radius_all(3.0)
                                    .show(|ui| {
                                        ui.text("Always Allow")
                                            .size(9.0)
                                            .color(colors.text_primary)
                                            .show();
                                    });
                                if always_btn.clicked {
                                    allow_always = true;
                                }

                                let deny_btn = ui.container()
                                    .padding(2.5, 8.0, 2.5, 8.0)
                                    .bg(Color::rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.15))
                                    .border(Color::rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.4), 1.0)
                                    .radius_all(3.0)
                                    .show(|ui| {
                                        ui.text("Deny")
                                            .size(9.0)
                                            .weight(FontWeight::Medium)
                                            .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                            .show();
                                    });
                                if deny_btn.clicked {
                                    deny = true;
                                }
                            });
                    });

                if allow_once {
                    state.respond_to_permission(&perm.call_id, true, false);
                    ui.needs_redraw = true;
                } else if allow_always {
                    state.respond_to_permission(&perm.call_id, true, true);
                    ui.needs_redraw = true;
                } else if deny {
                    state.respond_to_permission(&perm.call_id, false, false);
                    ui.needs_redraw = true;
                }
            }

            // ─── Active Drawer: Popover above Toolbar ──────────────────────
            let mut file_to_open = None;

            match state.active_input_drawer {
                AiInputDrawer::PendingModifications | AiInputDrawer::SessionChanges => {
                    struct DrawerFileItem {
                        file_path: std::path::PathBuf,
                        file_name: String,
                        rel_path: String,
                        lines_added: usize,
                        lines_removed: usize,
                    }

                    let mut file_items: Vec<DrawerFileItem> = Vec::new();

                    for p in &state.pending_modifications {
                        let added = p.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                        let removed = p.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                        let file_name = p.file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                        let rel_path = p.file_path.strip_prefix(&state.current_dir).unwrap_or(&p.file_path).to_string_lossy().to_string();
                        file_items.push(DrawerFileItem {
                            file_path: p.file_path.clone(),
                            file_name,
                            rel_path,
                            lines_added: added,
                            lines_removed: removed,
                        });
                    }

                    if file_items.is_empty() {
                        if let Some(ref p) = state.pending_modification {
                            let added = p.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Added).count();
                            let removed = p.diff_lines.iter().filter(|(_, s)| *s == DiffLineState::Removed).count();
                            let file_name = p.file_path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                            let rel_path = p.file_path.strip_prefix(&state.current_dir).unwrap_or(&p.file_path).to_string_lossy().to_string();
                            file_items.push(DrawerFileItem {
                                file_path: p.file_path.clone(),
                                file_name,
                                rel_path,
                                lines_added: added,
                                lines_removed: removed,
                            });
                        }
                    }

                    for s in &state.session_modified_files {
                        if !file_items.iter().any(|f| f.file_path == s.file_path) {
                            let file_name = s.file_path.file_name().and_then(|n| n.to_str()).unwrap_or(&s.file_name).to_string();
                            let rel_path = s.file_path.strip_prefix(&state.current_dir).unwrap_or(&s.file_path).to_string_lossy().to_string();
                            file_items.push(DrawerFileItem {
                                file_path: s.file_path.clone(),
                                file_name,
                                rel_path,
                                lines_added: s.lines_added,
                                lines_removed: s.lines_removed,
                            });
                        }
                    }

                    if file_items.is_empty() {
                        state.active_input_drawer = AiInputDrawer::None;
                    } else {
                        let total_files = file_items.len();
                        let file_list_h = (total_files as f32 * 28.0 + 8.0).min(130.0);
                        let mut accept_all_pressed = false;
                        let mut reject_all_pressed = false;
                        let mut reject_single = None;

                        ui.container()
                            .width(ai_panel_w)
                            .bg(colors.bg_sidebar)
                            .border(colors.border, 1.0)
                            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, -4.0, 14.0)
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
                                        for item in &file_items {
                                            let file_row = ui.container()
                                                .width(ai_panel_w - 18.0)
                                                .height(26.0)
                                                .padding(2.0, 8.0, 2.0, 6.0)
                                                .radius_all(3.0)
                                                .hover_bg(colors.highlight)
                                                .row()
                                                .valign(Align::Center)
                                                .halign(Align::SpaceBetween)
                                                .show(|ui| {
                                                    // Left: File name + relative path
                                                    ui.container()
                                                        .row()
                                                        .gap(6.0)
                                                        .valign(Align::Center)
                                                        .show(|ui| {
                                                            ui.text(&item.file_name)
                                                                .size(11.0)
                                                                .weight(FontWeight::SemiBold)
                                                                .color(colors.text_primary)
                                                                .show();

                                                            ui.text(&item.rel_path)
                                                                .size(9.5)
                                                                .weight(FontWeight::Regular)
                                                                .color(colors.text_muted)
                                                                .show();
                                                        });

                                                    // Right: +added -removed + Reject single button
                                                    ui.container()
                                                        .row()
                                                        .gap(6.0)
                                                        .valign(Align::Center)
                                                        .show(|ui| {
                                                            if item.lines_added > 0 || item.lines_removed == 0 {
                                                                ui.text(&format!("+{}", item.lines_added))
                                                                    .size(10.0)
                                                                    .weight(FontWeight::Medium)
                                                                    .color(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0))
                                                                    .show();
                                                            }
                                                            if item.lines_removed > 0 {
                                                                ui.text(&format!("-{}", item.lines_removed))
                                                                    .size(10.0)
                                                                    .weight(FontWeight::Medium)
                                                                    .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                                                    .show();
                                                            }

                                                            // Single reject button ✕
                                                            let rej_btn = ui.container()
                                                                .padding(1.0, 4.0, 1.0, 4.0)
                                                                .radius_all(2.0)
                                                                .hover_bg(Color::rgba(0.9, 0.2, 0.2, 0.15))
                                                                .show(|ui| {
                                                                    ui.text("✕")
                                                                        .size(9.5)
                                                                        .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                                                        .show();
                                                                });
                                                            if rej_btn.clicked {
                                                                reject_single = Some(item.file_path.clone());
                                                            }
                                                        });
                                                });

                                            if file_row.clicked && reject_single.is_none() {
                                                file_to_open = Some(item.file_path.clone());
                                            }
                                        }
                                    });

                                // Clean bottom action bar with Reject all & Accept all
                                ui.container()
                                    .width(ai_panel_w - 2.0)
                                    .height(30.0)
                                    .padding(0.0, 8.0, 0.0, 8.0)
                                    .border(colors.border, 1.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        let count_label = format!("{} modified file{}", total_files, if total_files == 1 { "" } else { "s" });
                                        ui.text(&count_label)
                                            .size(9.5)
                                            .weight(FontWeight::Medium)
                                            .color(colors.text_muted)
                                            .show();

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
                                                            .size(9.5)
                                                            .weight(FontWeight::Medium)
                                                            .color(colors.text_muted)
                                                            .show();
                                                    });
                                                if reject_btn.clicked {
                                                    reject_all_pressed = true;
                                                }

                                                let accept_btn = ui.container()
                                                    .padding(2.0, 8.0, 2.0, 8.0)
                                                    .bg(colors.accent)
                                                    .hover_bg(colors.accent.with_alpha(0.85))
                                                    .radius_all(3.0)
                                                    .show(|ui| {
                                                        ui.text("Accept all")
                                                            .size(9.5)
                                                            .weight(FontWeight::SemiBold)
                                                            .color(Color::WHITE)
                                                            .show();
                                                    });
                                                if accept_btn.clicked {
                                                    accept_all_pressed = true;
                                                }
                                            });
                                    });
                            });

                        if accept_all_pressed {
                            state.accept_all_modifications();
                            state.session_modified_files.clear();
                            state.active_input_drawer = AiInputDrawer::None;
                            ui.needs_redraw = true;
                        } else if reject_all_pressed {
                            state.reject_all_modifications();
                            state.session_modified_files.clear();
                            state.active_input_drawer = AiInputDrawer::None;
                            ui.needs_redraw = true;
                        } else if let Some(path) = reject_single {
                            state.reject_single_modification(&path);
                            state.session_modified_files.retain(|f| f.file_path != path);
                            if file_items.len() <= 1 {
                                state.active_input_drawer = AiInputDrawer::None;
                            }
                            ui.needs_redraw = true;
                        }
                    }
                }
                AiInputDrawer::Terminals => {
                    let running_cmds: Vec<_> = state.ai_terminal_commands.iter()
                        .filter(|c| c.status == crate::state::AiCommandStatus::Running)
                        .cloned()
                        .collect();

                    if running_cmds.is_empty() {
                        state.active_input_drawer = AiInputDrawer::None;
                    } else {
                        let mut cancel_requested_id = None;
                        let drawer_h = (running_cmds.len() as f32 * 26.0 + 8.0).clamp(34.0, 130.0);

                        ui.container()
                            .width(ai_panel_w)
                            .height(drawer_h)
                            .bg(colors.bg_sidebar)
                            .border(colors.border, 1.0)
                            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, -4.0, 14.0)
                            .radius(4.0, 4.0, 0.0, 0.0)
                            .padding(4.0, 8.0, 4.0, 8.0)
                            .column()
                            .gap(2.0)
                            .scroll_y(true)
                            .show(|ui| {
                                for cmd in &running_cmds {
                                    let display_cmd = if cmd.command.chars().count() > 38 {
                                        format!("{}...", cmd.command.chars().take(38).collect::<String>())
                                    } else {
                                        cmd.command.clone()
                                    };

                                    ui.container()
                                        .width(ai_panel_w - 18.0)
                                        .height(24.0)
                                        .padding(2.0, 8.0, 2.0, 6.0)
                                        .radius_all(3.0)
                                        .hover_bg(colors.highlight)
                                        .row()
                                        .valign(Align::Center)
                                        .halign(Align::SpaceBetween)
                                        .show(|ui| {
                                            ui.text(&display_cmd)
                                                .size(11.0)
                                                .weight(FontWeight::Medium)
                                                .color(colors.text_primary)
                                                .show();

                                            // Cross icon (✕) to cancel/kill active command
                                            let cross_btn = ui.container()
                                                .padding(1.0, 4.0, 1.0, 4.0)
                                                .radius_all(2.0)
                                                .hover_bg(Color::rgba(0.9, 0.2, 0.2, 0.15))
                                                .show(|ui| {
                                                    ui.text("✕")
                                                        .size(10.0)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });
                                            if cross_btn.clicked {
                                                cancel_requested_id = Some(cmd.id.clone());
                                            }
                                        });
                                }
                            });

                        if let Some(cmd_id) = cancel_requested_id {
                            state.cancel_ai_terminal_command(&cmd_id);
                            if state.ai_terminal_commands.iter().filter(|c| c.status == crate::state::AiCommandStatus::Running).count() == 0 {
                                state.active_input_drawer = AiInputDrawer::None;
                            }
                            ui.needs_redraw = true;
                        }
                    }
                }
                AiInputDrawer::Artifacts => {
                    let arts = state.current_session.list_artifacts();
                    if arts.is_empty() {
                        state.active_input_drawer = AiInputDrawer::None;
                    } else {
                        let arts_h = (arts.len() as f32 * 26.0 + 8.0).clamp(34.0, 130.0);

                        ui.container()
                            .width(ai_panel_w)
                            .height(arts_h)
                            .bg(colors.bg_sidebar)
                            .border(colors.border, 1.0)
                            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, -4.0, 14.0)
                            .radius(4.0, 4.0, 0.0, 0.0)
                            .padding(4.0, 8.0, 4.0, 8.0)
                            .column()
                            .gap(2.0)
                            .scroll_y(true)
                            .show(|ui| {
                                for art in &arts {
                                    let art_row = ui.container()
                                        .width(ai_panel_w - 18.0)
                                        .height(24.0)
                                        .padding(2.0, 8.0, 2.0, 8.0)
                                        .radius_all(3.0)
                                        .hover_bg(colors.highlight)
                                        .row()
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text(&art.name)
                                                .size(11.0)
                                                .weight(FontWeight::Medium)
                                                .color(colors.text_primary)
                                                .show();
                                        });

                                    if art_row.clicked {
                                        file_to_open = Some(art.path.clone());
                                    }
                                }
                            });
                    }
                }
                AiInputDrawer::None => {}
            }

            if let Some(target) = file_to_open {
                state.open_file(&target);
                ui.needs_redraw = true;
            }

            // ─── Top of Input Prompt Area: Toolbar (Unified Framework Badges) ───
            let has_pending = !state.pending_modifications.is_empty()
                || state.pending_modification.is_some()
                || !state.session_modified_files.is_empty();
            let has_running_term = state.ai_terminal_commands.iter().any(|c| c.status == crate::state::AiCommandStatus::Running);
            let has_artifacts = state.current_session.has_plan || state.current_session.has_mission || state.current_session.has_walkthrough;
            let is_pending_open = state.active_input_drawer == AiInputDrawer::PendingModifications
                || state.active_input_drawer == AiInputDrawer::SessionChanges;

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
                    let review_btn = ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .bg(if is_pending_open { colors.highlight } else { colors.bg_base })
                        .border(if is_pending_open { colors.accent } else { colors.border }, 1.0)
                        .hover_bg(colors.highlight)
                        .radius_all(3.0)
                        .row()
                        .gap(4.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{ea71}")
                                .size(12.0)
                                .color(if is_pending_open { colors.text_primary } else { colors.text_muted })
                                .show();
                            ui.text("Review Changes")
                                .size(9.0)
                                .weight(FontWeight::Medium)
                                .color(if is_pending_open { colors.text_primary } else { colors.text_muted })
                                .show();
                        });
                    if review_btn.clicked {
                        state.active_input_drawer = if is_pending_open { AiInputDrawer::None } else { AiInputDrawer::PendingModifications };
                        ui.needs_redraw = true;
                    }
                });

            // ─── AI Quota & Engine Status Bar ─────────────────────────────────
            let quota_banner_w = ai_panel_w;
            let active_notice = state.last_provider_notice.clone();
            let mut dismiss_notice = false;
            let mut switch_to_groq = false;
            let mut switch_to_native = false;

            if let Some(ref notice) = active_notice {
                let is_goose = state.ai_backend == crate::agent::AgentBackend::GooseAcp;
                let is_quota = notice.contains("429")
                    || notice.contains("quota")
                    || notice.contains("Quota")
                    || notice.contains("RESOURCE_EXHAUSTED");

                ui.container()
                    .width(quota_banner_w)
                    .bg(Color::rgba(0.95, 0.45, 0.2, 0.14))
                    .border(Color::rgba(0.95, 0.45, 0.2, 0.45), 1.0)
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
                                let display_msg = if is_goose {
                                    "Goose ACP Error · Switch to ZENE Native"
                                } else if is_quota {
                                    "Gemini Quota Exceeded (429) · Switch to Groq (Free & Fast)"
                                } else {
                                    let cleaned = notice.strip_prefix("AI Notice: ").unwrap_or(notice);
                                    if cleaned.len() > 55 {
                                        &cleaned[..52]
                                    } else {
                                        cleaned
                                    }
                                };
                                ui.text(display_msg)
                                    .size(9.0)
                                    .weight(FontWeight::Medium)
                                    .color(Color::rgb(0.98, 0.70, 0.35))
                                    .show();
                            });

                        ui.container()
                            .row()
                            .gap(4.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                if is_goose {
                                    let native_btn = ui.container()
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .bg(Color::rgba(0.2, 0.6, 0.95, 0.25))
                                        .hover_bg(Color::rgba(0.2, 0.6, 0.95, 0.50))
                                        .radius_all(3.0)
                                        .show(|ui| {
                                            ui.text("Switch to ZENE")
                                                .size(8.5)
                                                .weight(FontWeight::SemiBold)
                                                .color(Color::rgb(0.85, 0.95, 1.0))
                                                .show();
                                        });
                                    if native_btn.clicked {
                                        switch_to_native = true;
                                    }
                                } else if is_quota || state.selected_model.contains("gemini") {
                                    let groq_btn = ui.container()
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .bg(Color::rgba(0.95, 0.55, 0.2, 0.25))
                                        .hover_bg(Color::rgba(0.95, 0.55, 0.2, 0.50))
                                        .radius_all(3.0)
                                        .show(|ui| {
                                            ui.text("Switch to Groq")
                                                .size(8.5)
                                                .weight(FontWeight::SemiBold)
                                                .color(Color::rgb(1.0, 0.92, 0.75))
                                                .show();
                                        });
                                    if groq_btn.clicked {
                                        switch_to_groq = true;
                                    }
                                }

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
                    });

                if switch_to_native {
                    state.ai_backend = crate::agent::AgentBackend::Native;
                    state.last_provider_notice = None;
                    state.status_message = "Switched to ZENE (Native) engine".to_string();
                    ui.needs_redraw = true;
                } else if switch_to_groq {
                    state.selected_model = "qwen/qwen3.8-27b".to_string();
                    state.model_auto_mode = false;
                    state.last_provider_notice = None;
                    state.status_message = "Switched AI model to Qwen 3.8 27B (Groq)".to_string();
                    ui.needs_redraw = true;
                } else if dismiss_notice {
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
                            .gap(5.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                let (engine_dot, engine_label) = if state.model_auto_mode {
                                    (Color::rgb(0.35, 0.85, 0.45), "Auto")
                                } else if state.selected_model.contains("qwen") || state.selected_model.contains("gpt") {
                                    (Color::rgb(0.35, 0.85, 0.45), "Groq")
                                } else if state.selected_model.contains("gemini") {
                                    (Color::rgb(0.95, 0.75, 0.35), "Google")
                                } else {
                                    (Color::rgb(0.45, 0.65, 0.95), "AI")
                                };
                                ui.text("●")
                                    .size(6.0)
                                    .color(engine_dot)
                                    .show();
                                ui.text(engine_label)
                                    .size(8.5)
                                    .weight(FontWeight::Medium)
                                    .color(colors.text_dim)
                                    .show();

                                ui.text("·")
                                    .size(8.5)
                                    .color(colors.border)
                                    .show();

                                // Manual Quick Model Switch Chips
                                let is_qwen = !state.model_auto_mode && state.selected_model == "qwen/qwen3.8-27b";
                                let qwen_chip = ui.container()
                                    .padding(1.0, 4.0, 1.0, 4.0)
                                    .radius_all(3.0)
                                    .bg(if is_qwen { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
                                    .border(if is_qwen { colors.accent } else { Color::TRANSPARENT }, 0.5)
                                    .show(|ui| {
                                        ui.text("Qwen 3.8")
                                            .size(8.5)
                                            .weight(if is_qwen { FontWeight::Bold } else { FontWeight::Regular })
                                            .color(if is_qwen { colors.accent } else { colors.text_muted })
                                            .show();
                                    });
                                if qwen_chip.clicked {
                                    state.selected_model = "qwen/qwen3.8-27b".to_string();
                                    state.model_auto_mode = false;
                                    state.last_provider_notice = None;
                                    state.status_message = "Active AI model set to Qwen 3.8 27B (Groq)".to_string();
                                    ui.needs_redraw = true;
                                }

                                let is_gem = !state.model_auto_mode && state.selected_model.contains("gemini");
                                let gem_chip = ui.container()
                                    .padding(1.0, 4.0, 1.0, 4.0)
                                    .radius_all(3.0)
                                    .bg(if is_gem { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
                                    .border(if is_gem { colors.accent } else { Color::TRANSPARENT }, 0.5)
                                    .show(|ui| {
                                        ui.text("Gemini 3.6")
                                            .size(8.5)
                                            .weight(if is_gem { FontWeight::Bold } else { FontWeight::Regular })
                                            .color(if is_gem { colors.accent } else { colors.text_muted })
                                            .show();
                                    });
                                if gem_chip.clicked {
                                    state.selected_model = "gemini-3.6-flash".to_string();
                                    state.model_auto_mode = false;
                                    state.last_provider_notice = None;
                                    state.status_message = "Active AI model set to Gemini 3.6 Flash".to_string();
                                    ui.needs_redraw = true;
                                }

                                let is_gpt = !state.model_auto_mode && state.selected_model.contains("gpt-oss");
                                let gpt_chip = ui.container()
                                    .padding(1.0, 4.0, 1.0, 4.0)
                                    .radius_all(3.0)
                                    .bg(if is_gpt { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
                                    .border(if is_gpt { colors.accent } else { Color::TRANSPARENT }, 0.5)
                                    .show(|ui| {
                                        ui.text("GPT-OSS")
                                            .size(8.5)
                                            .weight(if is_gpt { FontWeight::Bold } else { FontWeight::Regular })
                                            .color(if is_gpt { colors.accent } else { colors.text_muted })
                                            .show();
                                    });
                                if gpt_chip.clicked {
                                    state.selected_model = "openai/gpt-oss-120b".to_string();
                                    state.model_auto_mode = false;
                                    state.last_provider_notice = None;
                                    state.status_message = "Active AI model set to GPT-OSS 120B (Groq)".to_string();
                                    ui.needs_redraw = true;
                                }
                            });
                        ui.container()
                            .row()
                            .gap(4.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                let (status_dot, status_label) = if is_input_focused || state.is_agent_working() {
                                    (Color::rgb(0.35, 0.70, 0.95), "Working")
                                } else {
                                    (Color::rgb(0.35, 0.85, 0.45), "Ready")
                                };
                                ui.text("●")
                                    .size(6.0)
                                    .color(status_dot)
                                    .show();
                                ui.text(status_label)
                                    .size(8.5)
                                    .weight(FontWeight::SemiBold)
                                    .color(status_dot)
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
                                        add_ctx_btn_just_clicked = true;
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
                                    let is_working = state.is_agent_working();
                                    let model_display = if state.model_auto_mode {
                                        "Auto"
                                    } else if !state.selected_model.is_empty() {
                                        friendly_model_name(&state.selected_model)
                                    } else {
                                        "Select a Model"
                                    };

                                    let (status_dot_color, status_dot_char) = if state.last_provider_notice.is_some() {
                                        (Color::rgb(0.95, 0.55, 0.20), "●")
                                    } else if is_working {
                                        (Color::rgb(0.35, 0.70, 0.95), "●")
                                    } else {
                                        (Color::rgb(0.35, 0.85, 0.45), "●")
                                    };

                                    let model_btn = ui.container()
                                        .id(Id::from_u64(999998603))
                                        .height(24.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .radius_all(4.0)
                                        .bg(if state.ai_model_menu_open { Color::rgba(1.0, 1.0, 1.0, 0.14) } else { Color::TRANSPARENT })
                                        .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.12))
                                        .row()
                                        .gap(4.0)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text(status_dot_char)
                                                .size(6.5)
                                                .color(status_dot_color)
                                                .show();
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
                                        model_btn_just_clicked = true;
                                        ui.needs_redraw = true;
                                    } else if ui.right_clicked && model_btn.hovered {
                                        let cycle = [
                                            ("qwen/qwen3.8-27b", "Qwen 3.8 27B"),
                                            ("openai/gpt-oss-120b", "GPT-OSS 120B"),
                                            ("gemini-3.6-flash", "Gemini 3.6 Flash"),
                                            ("gemini-2.5-pro", "Gemini 2.5 Pro"),
                                            ("claude-3-7-sonnet", "Claude 3.7 Sonnet"),
                                        ];
                                        let curr = &state.selected_model;
                                        let next_idx = cycle.iter().position(|(id, _)| id == curr)
                                            .map(|idx| (idx + 1) % cycle.len())
                                            .unwrap_or(0);
                                        let (next_id, next_name) = cycle[next_idx];
                                        state.selected_model = next_id.to_string();
                                        state.model_auto_mode = false;
                                        state.last_provider_notice = None;
                                        state.status_message = format!("Switched model to {}", next_name);
                                        ui.needs_redraw = true;
                                    }

                                    // Send & submit button with Lucide ArrowRight and Lucide Pause
                                    let is_paused = state.current_mission.as_ref().map(|m| m.is_paused).unwrap_or(false);
                                    let (icon, icon_color, icon_size, btn_bg, btn_hover, btn_border) = if is_working {
                                        (
                                            "\u{f04c}", // Lucide Pause (||)
                                            Color::rgb(245.0 / 255.0, 75.0 / 255.0, 75.0 / 255.0), // Bright red icon
                                            18.0,
                                            Color::rgba(42.0 / 255.0, 42.0 / 255.0, 48.0 / 255.0, 1.0), // Dark gray circle dark bg
                                            Color::rgba(58.0 / 255.0, 58.0 / 255.0, 66.0 / 255.0, 1.0),
                                            Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.15),
                                        )
                                    } else if is_paused && state.ai_input.trim().is_empty() {
                                        (
                                            "\u{f04b}", // Resume green play
                                            Color::WHITE,
                                            14.5,
                                            Color::rgb(0.35, 0.85, 0.45),
                                            Color::rgb(0.45, 0.95, 0.55),
                                            Color::TRANSPARENT,
                                        )
                                    } else {
                                        (
                                            "\u{ea9c}", // Lucide ArrowRight (-->)
                                            Color::WHITE,
                                            17.5,
                                            colors.accent,
                                            colors.accent.with_alpha(0.85),
                                            Color::TRANSPARENT,
                                        )
                                    };

                                    let send_circle = ui.container()
                                        .id(Id::from_u64(999998604))
                                        .width(26.0)
                                        .height(26.0)
                                        .radius_all(13.0)
                                        .bg(btn_bg)
                                        .hover_bg(btn_hover)
                                        .border(btn_border, 1.0)
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text(icon)
                                                .size(icon_size)
                                                .weight(FontWeight::Bold)
                                                .color(icon_color)
                                                .show();
                                        });

                                    if send_circle.clicked {
                                        if is_working {
                                            state.stop_ai_generation();
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
        let (menu_x, menu_y) = if let Some((btn_rect, _)) = ui.get_recorded_layout(Id::from_u64(999998601)) {
            let x = btn_rect.origin.x.clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
            let y = (btn_rect.origin.y - menu_h - 4.0).max(splitter_y + 36.0);
            (x, y)
        } else {
            let x = (splitter_x + 8.0).clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
            let y = (splitter_y + height - bottom_input_total_h - menu_h - 4.0).max(splitter_y + 40.0);
            (x, y)
        };

        let mouse_in_menu = ui.mouse_in_rect(menu_x - 6.0, menu_y - 6.0, menu_w + 12.0, menu_h + 12.0);
        let mouse_in_add_btn = if let Some((btn_rect, _)) = ui.get_recorded_layout(Id::from_u64(999998601)) {
            ui.mouse_in_rect(btn_rect.origin.x - 4.0, btn_rect.origin.y - 4.0, btn_rect.size.width + 8.0, btn_rect.size.height + 8.0)
        } else {
            ui.mouse_in_rect(splitter_x + 8.0, splitter_y + height - 36.0, 32.0, 32.0)
        };
        let mut close_menu = false;
        if (ui.clicked || ui.right_clicked) && !add_ctx_btn_just_clicked && !mouse_in_menu && !mouse_in_add_btn {
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
        let menu_w = 236.0;
        let menu_h = if state.adding_custom_model { 360.0 } else { 310.0 };

        let (menu_x, menu_y) = if let Some((btn_rect, _)) = ui.get_recorded_layout(Id::from_u64(999998603)) {
            let x = (btn_rect.origin.x + btn_rect.size.width - menu_w).clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
            let y = (btn_rect.origin.y - menu_h - 4.0).max(splitter_y + 36.0);
            (x, y)
        } else {
            let x = (splitter_x + ai_panel_w - menu_w - 8.0).clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
            let y = (splitter_y + height - bottom_input_total_h - menu_h - 4.0).max(splitter_y + 40.0);
            (x, y)
        };

        let mouse_in_menu = ui.mouse_in_rect(menu_x - 6.0, menu_y - 6.0, menu_w + 12.0, menu_h + 12.0);
        let mouse_in_model_btn = if let Some((btn_rect, _)) = ui.get_recorded_layout(Id::from_u64(999998603)) {
            ui.mouse_in_rect(btn_rect.origin.x - 4.0, btn_rect.origin.y - 4.0, btn_rect.size.width + 8.0, btn_rect.size.height + 8.0)
        } else {
            ui.mouse_in_rect(splitter_x + ai_panel_w - 140.0, splitter_y + height - 36.0, 140.0, 32.0)
        };

        let mut close_menu = false;
        if (ui.clicked || ui.right_clicked) && !model_btn_just_clicked && !mouse_in_menu && !mouse_in_model_btn {
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
                            state.last_provider_notice = None;
                            state.status_message = "Auto model failover enabled".to_string();
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
                        ("qwen/qwen3.8-27b", "Qwen 3.8 27B", "Groq (Fast)"),
                        ("openai/gpt-oss-120b", "GPT-OSS 120B", "Groq"),
                        ("openai/gpt-oss-20b", "GPT-OSS 20B", "Groq"),
                        ("gemini-3.6-flash", "Gemini 3.6 Flash", "Google"),
                        ("gemini-2.5-pro", "Gemini 2.5 Pro", "Google"),
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
                            state.last_provider_notice = None;
                            state.status_message = format!("Active AI model set to {}", m_name);
                            close_menu = true;
                            ui.needs_redraw = true;
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
                            state.last_provider_notice = None;
                            state.status_message = format!("Active AI model set to {}", c_model);
                            close_menu = true;
                            ui.needs_redraw = true;
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
    } else if !name.contains('.') || name == "." || name.ends_with('/') {
        "\u{f07b}"
    } else {
        "\u{ea7f}"
    }
}

fn get_file_glyph_and_color(name: &str, colors: &crate::theme::ThemeColors) -> (&'static str, Color) {
    if name.ends_with(".rs") {
        ("\u{e7a8}", Color::rgb(0.95, 0.42, 0.20))
    } else if name.to_lowercase().contains("media") || name.ends_with(".png") || name.ends_with(".jpg") || name.ends_with(".jpeg") || name.ends_with(".webp") || name.ends_with(".svg") {
        ("\u{f03e}", Color::rgb(0.70, 0.70, 0.75))
    } else if name.ends_with(".ts") || name.ends_with(".js") || name.ends_with(".tsx") || name.ends_with(".jsx") {
        ("\u{e628}", Color::rgb(0.35, 0.65, 0.95))
    } else if name.ends_with(".toml") || name.ends_with(".json") || name.ends_with(".yaml") || name.ends_with(".yml") {
        ("\u{e60b}", Color::rgb(0.40, 0.80, 0.75))
    } else if name.ends_with(".md") || name.ends_with(".txt") {
        ("\u{f15c}", Color::rgb(0.45, 0.65, 0.95))
    } else if name.ends_with(".py") {
        ("\u{e73c}", Color::rgb(0.95, 0.80, 0.30))
    } else if name.ends_with(".html") || name.ends_with(".css") {
        ("\u{e736}", Color::rgb(0.90, 0.45, 0.30))
    } else if !name.contains('.') || name == "." || name.ends_with('/') {
        ("\u{f07b}", Color::rgb(0.35, 0.60, 0.95))
    } else {
        ("\u{ea7f}", colors.accent)
    }
}

fn friendly_model_name(id: &str) -> &str {
    match id {
        "gemini-3.6-flash" => "Gemini 3.6 Flash",
        "gemini-2.5-pro" => "Gemini 2.5 Pro",
        "gemini-3.5-flash" | "gemini-3.7-flash" | "gemini-2.5-flash" => "Gemini 3.6 Flash",
        "gemini-3.1-flash-lite" | "gemini-flash-lite-latest" => "Gemini Flash Lite",
        "openai/gpt-oss-120b" => "GPT-OSS 120B",
        "openai/gpt-oss-20b" => "GPT-OSS 20B",
        "qwen/qwen3.8-27b" => "Qwen 3.8 27B",
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

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExploredTreeNode {
    pub name: String,
    pub full_path: String,
    pub is_dir: bool,
    pub action_type: String,
    pub line_range: Option<String>,
    pub children: Vec<ExploredTreeNode>,
}

#[allow(dead_code)]
fn build_explored_tree(items: &[&crate::state::ExploredItem]) -> Vec<ExploredTreeNode> {
    let mut root_nodes: Vec<ExploredTreeNode> = Vec::new();

    for item in items {
        if item.action_type == "Searched" {
            root_nodes.push(ExploredTreeNode {
                name: item.file_name.clone(),
                full_path: item.file_name.clone(),
                is_dir: false,
                action_type: "Searched".to_string(),
                line_range: item.line_range.clone(),
                children: Vec::new(),
            });
            continue;
        }

        if item.file_name == "." || item.file_name.is_empty() {
            for child in &item.children {
                let is_dir = child.ends_with('/') || !child.contains('.');
                let clean_child = child.trim_end_matches('/').to_string();
                if !root_nodes.iter().any(|n| n.name == clean_child) {
                    root_nodes.push(ExploredTreeNode {
                        name: clean_child.clone(),
                        full_path: clean_child,
                        is_dir,
                        action_type: "Analyzed".to_string(),
                        line_range: None,
                        children: Vec::new(),
                    });
                }
            }
            continue;
        }

        let clean_path = item.file_name.trim_start_matches("./").trim_end_matches('/').to_string();
        let parts: Vec<&str> = clean_path.split('/').filter(|s| !s.is_empty()).collect();
        if parts.is_empty() {
            continue;
        }

        let is_dir_item = !item.children.is_empty()
            || (item.line_range.is_none() && (!parts.last().unwrap_or(&"").contains('.') || item.file_name.ends_with('/')));

        let mut current_children = &mut root_nodes;
        let mut path_so_far = String::new();

        for (idx, part) in parts.iter().enumerate() {
            let is_last = idx + 1 == parts.len();
            if !path_so_far.is_empty() {
                path_so_far.push('/');
            }
            path_so_far.push_str(part);

            if !is_last {
                let found_idx = current_children.iter().position(|n| n.name == *part);
                let node_idx = match found_idx {
                    Some(i) => i,
                    None => {
                        current_children.push(ExploredTreeNode {
                            name: part.to_string(),
                            full_path: path_so_far.clone(),
                            is_dir: true,
                            action_type: "Analyzed".to_string(),
                            line_range: None,
                            children: Vec::new(),
                        });
                        current_children.len() - 1
                    }
                };
                current_children = &mut current_children[node_idx].children;
            } else {
                let found_idx = current_children.iter().position(|n| n.name == *part);
                let node_idx = match found_idx {
                    Some(i) => {
                        let node = &mut current_children[i];
                        if is_dir_item {
                            node.is_dir = true;
                        } else {
                            node.is_dir = false;
                            if item.line_range.is_some() {
                                node.line_range = item.line_range.clone();
                            }
                        }
                        i
                    }
                    None => {
                        current_children.push(ExploredTreeNode {
                            name: part.to_string(),
                            full_path: path_so_far.clone(),
                            is_dir: is_dir_item,
                            action_type: "Analyzed".to_string(),
                            line_range: if is_dir_item { None } else { item.line_range.clone() },
                            children: Vec::new(),
                        });
                        current_children.len() - 1
                    }
                };

                if is_dir_item && !item.children.is_empty() {
                    let parent_path = path_so_far.clone();
                    let node = &mut current_children[node_idx];
                    for child in &item.children {
                        let is_child_dir = child.ends_with('/') || !child.contains('.');
                        let clean_child = child.trim_end_matches('/').to_string();
                        if !node.children.iter().any(|c| c.name == clean_child) {
                            let child_full_path = format!("{}/{}", parent_path, clean_child);
                            node.children.push(ExploredTreeNode {
                                name: clean_child,
                                full_path: child_full_path,
                                is_dir: is_child_dir,
                                action_type: "Analyzed".to_string(),
                                line_range: None,
                                children: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    sort_tree(&mut root_nodes);
    root_nodes
}

#[allow(dead_code)]
fn sort_tree(nodes: &mut Vec<ExploredTreeNode>) {
    nodes.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.cmp(&b.name),
        }
    });
    for node in nodes.iter_mut() {
        if !node.children.is_empty() {
            sort_tree(&mut node.children);
        }
    }
}

#[allow(dead_code)]
fn count_tree_nodes(nodes: &[ExploredTreeNode]) -> (usize, usize) {
    let mut files = 0;
    let mut folders = 0;
    for node in nodes {
        if node.action_type == "Searched" {
            // query search
        } else if node.is_dir {
            folders += 1;
            let (sub_files, sub_folders) = count_tree_nodes(&node.children);
            files += sub_files;
            folders += sub_folders;
        } else {
            files += 1;
        }
    }
    (files, folders)
}

#[allow(dead_code)]
fn render_explored_tree_node(
    ui: &mut Ui,
    state: &ZenthreeState,
    colors: &crate::theme::ThemeColors,
    msg_idx: usize,
    node: &ExploredTreeNode,
    depth: usize,
    jump_target: &mut Option<(String, Option<String>)>,
    toggle_folder_target: &mut Option<(usize, String)>,
) {
    let indent = (depth as f32) * 14.0;
    if node.action_type == "Searched" {
        ui.container()
            .padding_left(indent)
            .row()
            .gap(5.0)
            .valign(Align::Center)
            .show(|ui| {
                ui.text("•")
                    .size(9.0)
                    .color(colors.text_muted)
                    .show();
                ui.text("Searched")
                    .size(10.0)
                    .color(colors.text_dim)
                    .show();
                ui.text(&format!("`{}`", node.name))
                    .size(10.0)
                    .color(colors.accent)
                    .show();
                if let Some(ref range) = node.line_range {
                    ui.text(range)
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                }
            });
    } else if node.is_dir {
        let is_expanded = state.is_explored_folder_expanded(msg_idx, &node.full_path);
        let arrow = if is_expanded { "\u{f078}" } else { "\u{f054}" };

        let folder_btn = ui.container()
            .padding_left(indent)
            .padding(1.0, 3.0, 1.0, 3.0)
            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
            .radius_all(3.0)
            .row()
            .gap(4.0)
            .valign(Align::Center)
            .show(|ui| {
                ui.container()
                    .width(10.0)
                    .valign(Align::Center)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text(arrow)
                            .size(6.5)
                            .color(colors.text_muted)
                            .show();
                    });
                ui.text("Analyzed")
                    .size(10.0)
                    .color(colors.text_dim)
                    .show();
                ui.text("\u{f07b}")
                    .size(10.0)
                    .color(colors.accent)
                    .show();
                ui.text(&node.name)
                    .size(10.0)
                    .weight(FontWeight::Medium)
                    .color(colors.text_primary)
                    .show();
            });

        if folder_btn.clicked {
            *toggle_folder_target = Some((msg_idx, node.full_path.clone()));
        }

        if is_expanded {
            for child in &node.children {
                render_explored_tree_node(
                    ui,
                    state,
                    colors,
                    msg_idx,
                    child,
                    depth + 1,
                    jump_target,
                    toggle_folder_target,
                );
            }
        }
    } else {
        // Leaf file
        let glyph = get_file_glyph(&node.name);
        let file_btn = ui.container()
            .padding_left(indent + 14.0)
            .padding(1.0, 3.0, 1.0, 3.0)
            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.04))
            .radius_all(3.0)
            .row()
            .gap(4.0)
            .valign(Align::Center)
            .show(|ui| {
                ui.text("•")
                    .size(9.0)
                    .color(colors.text_muted)
                    .show();
                ui.text("Analyzed")
                    .size(10.0)
                    .color(colors.text_dim)
                    .show();
                ui.text(glyph)
                    .size(10.0)
                    .color(colors.accent)
                    .show();
                ui.text(&node.name)
                    .size(10.0)
                    .weight(FontWeight::Medium)
                    .color(colors.text_primary)
                    .show();
                if let Some(ref range) = node.line_range {
                    ui.container()
                        .padding(0.0, 3.0, 0.0, 3.0)
                        .radius_all(3.0)
                        .bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                        .show(|ui| {
                            ui.text(range)
                                .size(9.5)
                                .color(colors.accent)
                                .show();
                        });
                }
            });

        if file_btn.clicked {
            *jump_target = Some((node.full_path.clone(), node.line_range.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ExploredItem;

    #[test]
    fn test_build_explored_tree_hierarchy_and_counting() {
        let root_dir = ExploredItem {
            file_name: ".".to_string(),
            line_range: None,
            action_type: "Analyzed".to_string(),
            thought_content: None,
            duration_secs: None,
            children: vec!["Cargo.toml".to_string(), "README.md".to_string(), "crates/".to_string()],
        };
        let cargo_toml = ExploredItem {
            file_name: "Cargo.toml".to_string(),
            line_range: Some("#L1-25".to_string()),
            action_type: "Analyzed".to_string(),
            thought_content: None,
            duration_secs: None,
            children: Vec::new(),
        };
        let crates_dir = ExploredItem {
            file_name: "crates".to_string(),
            line_range: None,
            action_type: "Analyzed".to_string(),
            thought_content: None,
            duration_secs: None,
            children: vec!["ai/".to_string(), "editor/".to_string()],
        };
        let editor_lib = ExploredItem {
            file_name: "crates/editor/src/lib.rs".to_string(),
            line_range: Some("#L1-50".to_string()),
            action_type: "Analyzed".to_string(),
            thought_content: None,
            duration_secs: None,
            children: Vec::new(),
        };

        let items = vec![&root_dir, &cargo_toml, &crates_dir, &editor_lib];
        let tree = build_explored_tree(&items);

        // Verify top-level: crates/ (folder first), Cargo.toml, README.md
        assert_eq!(tree[0].name, "crates");
        assert!(tree[0].is_dir);

        // Find crates node and its children
        let crates_node = &tree[0];
        let editor_node = crates_node.children.iter().find(|c| c.name == "editor").expect("editor node");
        assert!(editor_node.is_dir);

        let src_node = editor_node.children.iter().find(|c| c.name == "src").expect("src node");
        assert!(src_node.is_dir);

        let lib_node = src_node.children.iter().find(|c| c.name == "lib.rs").expect("lib.rs node");
        assert!(!lib_node.is_dir);
        assert_eq!(lib_node.line_range, Some("#L1-50".to_string()));

        let (files, folders) = count_tree_nodes(&tree);
        assert!(files >= 2);
        assert!(folders >= 4); // crates, ai, editor, src
    }
}

