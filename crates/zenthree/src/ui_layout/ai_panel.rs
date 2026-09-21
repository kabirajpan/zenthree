use crate::state::{ZenthreeState, AiModel, AiProvider};
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
                    // Left: dot + "Assistant" label + model badge
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

                            // Model badge (clickable dropdown trigger)
                            let model_badge_id = ui.id_from("ai_model_badge");
                            let model_badge = ui.container()
                                .id(model_badge_id)
                                .padding(2.0, 6.0, 2.0, 6.0)
                                .radius_all(4.0)
                                .bg(colors.highlight.with_alpha(0.6))
                                .hover_bg(colors.highlight)
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .show(|ui| {
                                    ui.text(state.selected_ai_model.display_name())
                                        .size(9.5)
                                        .color(colors.text_muted)
                                        .show();
                                    ui.text(if state.ai_model_menu_open { "\u{f0d8}" } else { "\u{f0d7}" })
                                        .size(8.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            if model_badge.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if model_badge.clicked {
                                state.ai_model_menu_open = !state.ai_model_menu_open;
                                ui.needs_redraw = true;
                            }
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
                                state.ai_session_json = None;
                                state.ai_block_cache.clear();
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

            // ─── Model selector dropdown (shown below hairline when open) ────
            if state.ai_model_menu_open {
                let mut select_model: Option<AiModel> = None;
                ui.container()
                    .width(ai_panel_w)
                    .padding(4.0, 10.0, 4.0, 10.0)
                    .bg(colors.bg_sidebar)
                    .border(colors.border.with_alpha(0.4), 0.5)
                    .column()
                    .gap(2.0)
                    .show(|ui| {
                        let mut last_provider: Option<AiProvider> = None;
                        for model in AiModel::ALL {
                            if last_provider != Some(model.provider()) {
                                let label = match model.provider() {
                                    AiProvider::Groq => "GROQ",
                                    AiProvider::Gemini => "GEMINI",
                                };
                                // Section gap before second group
                                if last_provider.is_some() {
                                    ui.container().height(4.0).show(|_| {});
                                }
                                ui.text(label)
                                    .size(8.5)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_muted.with_alpha(0.6))
                                    .show();
                                last_provider = Some(model.provider());
                            }

                            let is_selected = state.selected_ai_model == model;
                            let row_id = ui.id_from(&format!("ai_model_opt_{}", model.model_id()));
                            let row = ui.container()
                                .id(row_id)
                                .full_width()
                                .padding(4.0, 6.0, 4.0, 6.0)
                                .radius_all(4.0)
                                .bg(if is_selected { colors.accent.with_alpha(0.08) } else { colors.bg_sidebar })
                                .hover_bg(colors.highlight)
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    ui.text(model.display_name())
                                        .size(10.5)
                                        .color(if is_selected { colors.accent } else { colors.text_primary })
                                        .show();
                                    if is_selected {
                                        ui.text("\u{f00c}")
                                            .size(9.0)
                                            .color(colors.accent)
                                            .show();
                                    }
                                });

                            if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                            if row.clicked { select_model = Some(model); }
                        }
                    });

                if let Some(model) = select_model {
                    state.selected_ai_model = model;
                    state.ai_model_menu_open = false;
                    // Reset conversation so the new model gets a fresh session
                    state.ai_session_json = None;
                    state.save_persistent_state();
                    ui.needs_redraw = true;
                }
            }

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
                        let mut copy_code_snippet: Option<(u64, String)> = None;

                        for (msg_idx, msg) in state.ai_messages.iter().enumerate() {
                            let is_copied = state.copied_ai_msg
                                .as_ref()
                                .map(|(idx, t)| *idx == msg_idx && t.elapsed().as_secs_f32() < 2.0)
                                .unwrap_or(false);

                            let bubble_id = Id::from_u64(999990000 + (msg_idx as u64) * 2);
                            let copy_btn_id = Id::from_u64(999990000 + (msg_idx as u64) * 2 + 1);
                            let mut copy_clicked = false;

                            let full_text: &str = &msg.content;
                            let is_thinking = !msg.is_user && (full_text == "Thinking..." || full_text.starts_with("🔧"));

                            if msg.is_user {
                                // ─── User Message (Right-aligned, accent tint) ───
                                let bubble_max_w = (ai_panel_w - 36.0).max(80.0);
                                let user_text_w = (bubble_max_w - 20.0).max(60.0);

                                let newline_count = full_text.as_bytes().iter().filter(|&&b| b == b'\n').count();
                                let is_long = newline_count >= 5 || full_text.len() > 300;
                                let is_expanded = msg.expanded;

                                let display_text: &str = if is_long && !is_expanded {
                                    let mut end = full_text.len().min(240);
                                    while !full_text.is_char_boundary(end) {
                                        end -= 1;
                                    }
                                    &full_text[..end]
                                } else {
                                    full_text
                                };

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
                                // ─── Assistant Message (full-width, native background, no bubble) ───
                                let ai_text_w = (ai_panel_w - 24.0).max(60.0);
                                let code_box_w = ai_text_w.max(50.0);

                                ui.container()
                                    .full_width()
                                    .id(bubble_id)
                                    .padding(6.0, 12.0, 4.0, 12.0)
                                    .column()
                                    .gap(5.0)
                                    .show(|ui| {
                                                if is_thinking {
                                                    let status_label = if full_text.starts_with("🔧") {
                                                        full_text.trim_start_matches("🔧").trim()
                                                    } else {
                                                        "Thinking..."
                                                    };
                                                    let icon = if full_text.starts_with("🔧") { "\u{f0ad}" } else { "\u{f110}" };
                                                    ui.container()
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(6.0)
                                                        .padding(3.0, 8.0, 3.0, 8.0)
                                                        .radius_all(6.0)
                                                        .bg(colors.accent.with_alpha(0.08))
                                                        .border(colors.accent.with_alpha(0.2), 0.5)
                                                        .show(|ui| {
                                                            ui.text(icon)
                                                                .size(10.0)
                                                                .color(colors.accent)
                                                                .show();
                                                            ui.text(status_label)
                                                                .size(10.5)
                                                                .italic()
                                                                .color(colors.accent)
                                                                .show();
                                                        });
                                                } else {
                                                    // ── Cache: parse once, reuse every frame ──────────────
                                                    let content_hash = {
                                                        use std::hash::{Hash, Hasher};
                                                        let mut h = std::collections::hash_map::DefaultHasher::new();
                                                        full_text.hash(&mut h);
                                                        h.finish()
                                                    };
                                                    if !state.ai_block_cache.contains_key(&content_hash) {
                                                        let parsed = parse_markdown_blocks(full_text);
                                                        state.ai_block_cache.insert(content_hash, Box::new(parsed));
                                                    }
                                                    let blocks = state.ai_block_cache
                                                        .get(&content_hash)
                                                        .and_then(|b| b.downcast_ref::<Vec<MarkdownBlock>>())
                                                        .map(|v| v.as_slice())
                                                        .unwrap_or(&[]);
                                                    for (block_idx, block) in blocks.iter().enumerate() {
                                                        match block {
                                                            MarkdownBlock::Heading { level, text } => {
                                                                let (font_size, pad_t) = match level {
                                                                    1 => (13.0, 4.0),
                                                                    2 => (12.0, 3.0),
                                                                    _ => (11.5, 2.0),
                                                                };
                                                                let clean_text = strip_markdown_stars(text);
                                                                ui.container()
                                                                    .padding_top(pad_t)
                                                                    .padding_bottom(1.0)
                                                                    .show(|ui| {
                                                                        ui.text(&clean_text)
                                                                            .size(font_size)
                                                                            .weight(FontWeight::Bold)
                                                                            .color(colors.text_primary)
                                                                            .max_width(ai_text_w)
                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                            .show();
                                                                    });
                                                            }
                                                            MarkdownBlock::BulletItem { text } => {
                                                                let clean_text = strip_markdown_stars(text);
                                                                ui.container()
                                                                    .full_width()
                                                                    .row()
                                                                    .valign(Align::Top)
                                                                    .gap(6.0)
                                                                    .padding_left(2.0)
                                                                    .show(|ui| {
                                                                        ui.text("•")
                                                                            .size(11.0)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                        ui.text(&clean_text)
                                                                            .size(11.0)
                                                                            .line_height(1.35)
                                                                            .color(colors.text_primary)
                                                                            .max_width(ai_text_w - 14.0)
                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                            .show();
                                                                    });
                                                            }
                                                            MarkdownBlock::NumberedItem { num, text } => {
                                                                let clean_text = strip_markdown_stars(text);
                                                                ui.container()
                                                                    .full_width()
                                                                    .row()
                                                                    .valign(Align::Top)
                                                                    .gap(6.0)
                                                                    .padding_left(2.0)
                                                                    .show(|ui| {
                                                                        ui.text(&format!("{}.", num))
                                                                            .size(10.5)
                                                                            .weight(FontWeight::Medium)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                        ui.text(&clean_text)
                                                                            .size(11.0)
                                                                            .line_height(1.35)
                                                                            .color(colors.text_primary)
                                                                            .max_width(ai_text_w - 18.0)
                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                            .show();
                                                                    });
                                                            }
                                                            MarkdownBlock::CodeBlock { lang, code } => {
                                                                let code_btn_id = Id::from_u64(999990000 + (msg_idx as u64) * 1000 + (block_idx as u64) * 2 + 1);
                                                                let is_code_copied = state.copied_ai_code_id.as_ref()
                                                                    .map(|(id, t)| *id == code_btn_id.raw() && t.elapsed().as_secs_f32() < 2.0)
                                                                    .unwrap_or(false);

                                                                let lang_display = lang.as_deref().unwrap_or("code").to_uppercase();

                                                                ui.container()
                                                                    .width(code_box_w)
                                                                    .radius_all(6.0)
                                                                    .bg(colors.bg_base.with_alpha(0.85))
                                                                    .border(colors.border.with_alpha(0.5), 0.5)
                                                                    .column()
                                                                    .show(|ui| {
                                                                        // Code card header: language badge + Copy Code button
                                                                        ui.container()
                                                                            .full_width()
                                                                            .height(22.0)
                                                                            .padding(0.0, 8.0, 0.0, 8.0)
                                                                            .bg(colors.bg_sidebar.with_alpha(0.6))
                                                                            .radius_top(6.0)
                                                                            .row()
                                                                            .halign(Align::SpaceBetween)
                                                                            .valign(Align::Center)
                                                                            .show(|ui| {
                                                                                ui.text(&lang_display)
                                                                                    .size(9.0)
                                                                                    .weight(FontWeight::Bold)
                                                                                    .color(colors.text_muted)
                                                                                    .show();

                                                                                let copy_code_btn = ui.container()
                                                                                    .id(code_btn_id)
                                                                                    .padding(1.0, 5.0, 1.0, 5.0)
                                                                                    .radius_all(3.0)
                                                                                    .hover_bg(colors.highlight)
                                                                                    .row()
                                                                                    .valign(Align::Center)
                                                                                    .gap(4.0)
                                                                                    .show(|ui| {
                                                                                        ui.text(if is_code_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                                            .size(9.0)
                                                                                            .color(if is_code_copied { colors.accent } else { colors.text_muted })
                                                                                            .show();
                                                                                        ui.text(if is_code_copied { "Copied" } else { "Copy" })
                                                                                            .size(9.0)
                                                                                            .color(if is_code_copied { colors.accent } else { colors.text_muted })
                                                                                            .show();
                                                                                    });

                                                                                if copy_code_btn.hovered {
                                                                                    ui.cursor_icon = CursorIcon::Pointer;
                                                                                }
                                                                                if copy_code_btn.clicked {
                                                                                    copy_code_snippet = Some((code_btn_id.raw(), code.clone()));
                                                                                }
                                                                            });

                                                                        // Code content area
                                                                        ui.container()
                                                                            .padding(6.0, 8.0, 6.0, 8.0)
                                                                            .full_width()
                                                                            .show(|ui| {
                                                                                ui.text(code)
                                                                                    .size(10.0)
                                                                                    .monospace()
                                                                                    .line_height(1.35)
                                                                                    .color(colors.text_primary)
                                                                                    .max_width(code_box_w - 16.0)
                                                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                                    .show();
                                                                            });
                                                                    });
                                                            }
                                                            MarkdownBlock::Paragraph(text) => {
                                                                let clean_text = strip_markdown_stars(text);
                                                                ui.text(&clean_text)
                                                                    .size(11.0)
                                                                    .line_height(1.35)
                                                                    .color(colors.text_primary)
                                                                    .max_width(ai_text_w)
                                                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                    .show();
                                                            }
                                                        }
                                                    }

                                                    // ── Footer: copy button pinned right ──────────────
                                                    ui.container()
                                                        .full_width()
                                                        .row()
                                                        .halign(Align::Right)
                                                        .valign(Align::Center)
                                                        .padding_top(2.0)
                                                        .show(|ui| {
                                                            let copy_btn = ui.container()
                                                                .id(copy_btn_id)
                                                                .padding(2.0, 5.0, 2.0, 5.0)
                                                                .radius_all(4.0)
                                                                .hover_bg(colors.highlight)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(3.0)
                                                                .show(|ui| {
                                                                    ui.text(if is_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                        .size(9.0)
                                                                        .color(if is_copied { colors.accent } else { colors.text_muted.with_alpha(0.6) })
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
                                }
                            }
                        }

                        if let Some(idx) = toggle_expand_idx {
                            if let Some(m) = state.ai_messages.get_mut(idx) {
                                m.expanded = !m.expanded;
                                ui.needs_redraw = true;
                            }
                        }

                        if let Some((btn_id, snippet)) = copy_code_snippet {
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(snippet);
                            }
                            state.copied_ai_code_id = Some((btn_id, std::time::Instant::now()));
                            state.status_message = "Code snippet copied to clipboard".to_string();
                            ui.needs_redraw = true;
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
                                                if last.content == "Thinking..." || last.content.starts_with("🔧") {
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

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

#[derive(Debug, Clone, PartialEq)]
enum MarkdownBlock {
    Heading { level: u8, text: String },
    CodeBlock { lang: Option<String>, code: String },
    BulletItem { text: String },
    NumberedItem { num: String, text: String },
    Paragraph(String),
}

/// Strip bold/italic markdown markers so we can show clean text in non-rich text widgets.
fn strip_markdown_stars(s: &str) -> String {
    s.replace("**", "").replace("__", "")
}

/// Parse a markdown string into a flat list of [`MarkdownBlock`]s using pulldown-cmark.
fn parse_markdown_blocks(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();

    let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
    let parser = Parser::new_ext(content, opts);

    // We accumulate inline text here while inside a block-level element.
    let mut text_buf = String::new();

    // Ordered list tracking: when Some(n) we are inside an ordered list and n is the next number.
    let mut ordered_start: Option<u64> = None;
    // Counter for ordered items actually emitted so far in the current ordered list.
    let mut ordered_counter: u64 = 0;

    // Current heading level (0 = not in a heading)
    let mut heading_level: u8 = 0;

    // Are we inside an unordered list item?
    let mut in_bullet: bool = false;

    // Current code fence language
    let mut code_lang: Option<String> = None;

    for event in parser {
        match event {
            // ── Block-open ──────────────────────────────────────────────────
            Event::Start(Tag::Heading { level, .. }) => {
                text_buf.clear();
                heading_level = match level {
                    HeadingLevel::H1 => 1,
                    HeadingLevel::H2 => 2,
                    HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4,
                    HeadingLevel::H5 => 5,
                    HeadingLevel::H6 => 6,
                };
            }
            Event::End(TagEnd::Heading(_)) => {
                let text = std::mem::take(&mut text_buf).trim().to_string();
                if !text.is_empty() {
                    blocks.push(MarkdownBlock::Heading { level: heading_level, text });
                }
                heading_level = 0;
            }

            Event::Start(Tag::Paragraph) => {
                text_buf.clear();
            }
            Event::End(TagEnd::Paragraph) => {
                let text = std::mem::take(&mut text_buf).trim().to_string();
                if !text.is_empty() {
                    blocks.push(MarkdownBlock::Paragraph(text));
                }
            }

            // ── Lists ───────────────────────────────────────────────────────
            Event::Start(Tag::List(start_num)) => {
                ordered_start = start_num;
                ordered_counter = start_num.unwrap_or(0);
            }
            Event::End(TagEnd::List(_)) => {
                ordered_start = None;
                ordered_counter = 0;
            }
            Event::Start(Tag::Item) => {
                text_buf.clear();
                in_bullet = ordered_start.is_none();
            }
            Event::End(TagEnd::Item) => {
                let text = std::mem::take(&mut text_buf).trim().to_string();
                if !text.is_empty() {
                    if in_bullet {
                        blocks.push(MarkdownBlock::BulletItem { text });
                    } else {
                        blocks.push(MarkdownBlock::NumberedItem {
                            num: ordered_counter.to_string(),
                            text,
                        });
                        ordered_counter += 1;
                    }
                }
                in_bullet = false;
            }

            // ── Fenced code blocks ──────────────────────────────────────────
            Event::Start(Tag::CodeBlock(kind)) => {
                text_buf.clear();
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        let l = lang.trim().to_string();
                        if l.is_empty() { None } else { Some(l) }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
            }
            Event::End(TagEnd::CodeBlock) => {
                let code = std::mem::take(&mut text_buf);
                // pulldown-cmark appends a trailing newline; trim it.
                let code = code.trim_end_matches('\n').to_string();
                blocks.push(MarkdownBlock::CodeBlock { lang: code_lang.take(), code });
            }

            // ── Inline text & code ──────────────────────────────────────────
            Event::Text(t) | Event::Code(t) => {
                text_buf.push_str(&t);
            }
            Event::SoftBreak => {
                text_buf.push(' ');
            }
            Event::HardBreak => {
                text_buf.push('\n');
            }

            _ => {}
        }
    }

    blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_blocks_comprehensive() {
        let md = "I have inspected the project.\n\n## Diagnostics Summary\n- Errors: 0\n- Warnings: 0\n\n```rust\nfn main() {\n    println!(\"Hello Zenthra!\");\n}\n```\n\n1. First step\n2. Second step\n\nAll systems operational.";
        let blocks = parse_markdown_blocks(md);

        // paragraph, heading, 2 bullets, code block, 2 numbered, paragraph
        assert_eq!(blocks.len(), 8, "got: {:#?}", blocks);
        assert_eq!(blocks[0], MarkdownBlock::Paragraph("I have inspected the project.".into()));
        assert_eq!(blocks[1], MarkdownBlock::Heading { level: 2, text: "Diagnostics Summary".into() });
        assert_eq!(blocks[2], MarkdownBlock::BulletItem { text: "Errors: 0".into() });
        assert_eq!(blocks[3], MarkdownBlock::BulletItem { text: "Warnings: 0".into() });
        match &blocks[4] {
            MarkdownBlock::CodeBlock { lang, code } => {
                assert_eq!(lang.as_deref(), Some("rust"));
                assert!(code.contains("fn main()"));
            }
            other => panic!("Expected CodeBlock, got {:?}", other),
        }
        assert_eq!(blocks[5], MarkdownBlock::NumberedItem { num: "1".into(), text: "First step".into() });
        assert_eq!(blocks[6], MarkdownBlock::NumberedItem { num: "2".into(), text: "Second step".into() });
        assert_eq!(blocks[7], MarkdownBlock::Paragraph("All systems operational.".into()));
    }
}
