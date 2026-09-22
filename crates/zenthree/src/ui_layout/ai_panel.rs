use crate::state::{ZenthreeState, AiModel, AiProvider, TrajectoryStep, ExplorationItem, path_matches};
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context};
use crate::theme::ThemeColors;

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
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

            struct ModifiedFileItem {
                file_path: std::path::PathBuf,
                file_name: String,
                rel_path: String,
                lines_added: usize,
                lines_removed: usize,
            }

            let mut modified_files: Vec<ModifiedFileItem> = Vec::new();
            for rev in &state.pending_reviews {
                let file_name = rev.path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                let rel_path = rev.path.strip_prefix(&state.current_dir).unwrap_or(&rev.path).to_string_lossy().to_string();
                modified_files.push(ModifiedFileItem {
                    file_path: rev.path.clone(),
                    file_name,
                    rel_path,
                    lines_added: rev.total_added_lines(),
                    lines_removed: rev.total_removed_lines(),
                });
            }
            for tab in &state.tabs {
                if tab.lines_added > 0 || tab.lines_removed > 0 {
                    if let Some(ref tp) = tab.path {
                        if !modified_files.iter().any(|m| path_matches(&m.file_path, tp)) {
                            let file_name = tab.name.clone();
                            let rel_path = tp.strip_prefix(&state.current_dir).unwrap_or(tp).to_string_lossy().to_string();
                            modified_files.push(ModifiedFileItem {
                                file_path: tp.clone(),
                                file_name,
                                rel_path,
                                lines_added: tab.lines_added,
                                lines_removed: tab.lines_removed,
                            });
                        }
                    }
                }
            }
            for msg in state.ai_messages.iter().rev() {
                for step in &msg.trajectory {
                    if let TrajectoryStep::FileEdit { path, lines_added, lines_removed } = step {
                        let pb = std::path::PathBuf::from(path);
                        let full_p = if pb.is_absolute() { pb.clone() } else { state.current_dir.join(&pb) };
                        if !modified_files.iter().any(|item| path_matches(&item.file_path, &full_p) || item.file_name == *path) {
                            let fname = full_p.file_name().and_then(|n| n.to_str()).unwrap_or(path).to_string();
                            let rel = full_p.strip_prefix(&state.current_dir).unwrap_or(&full_p).to_string_lossy().to_string();
                            modified_files.push(ModifiedFileItem {
                                file_path: full_p,
                                file_name: fname,
                                rel_path: rel,
                                lines_added: *lines_added,
                                lines_removed: *lines_removed,
                            });
                        }
                    }
                }
            }
            let has_modified_files = !modified_files.is_empty();

            let modified_section_h = if has_modified_files {
                let cards_h = if state.show_modified_files_popup {
                    (modified_files.len().max(1) as f32 * 26.0 + 12.0).min(180.0) + 4.0
                } else {
                    0.0
                };
                cards_h + 28.0
            } else {
                24.0
            };

            // Dock height: prompt box + bottom row/padding + toolbar/modified section
            let dock_total_h = prompt_box_h + 52.0 + modified_section_h;
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
                .gap(20.0)
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
                        let mut toggle_trajectory_group: Option<(usize, usize)> = None;
                        let mut copy_target_idx: Option<usize> = None;
                        let mut copy_code_snippet: Option<(u64, String)> = None;

                        for (msg_idx, msg) in state.ai_messages.iter().enumerate() {
                            let is_copied = state.copied_ai_msg
                                .as_ref()
                                .map(|(idx, t)| *idx == msg_idx && t.elapsed().as_secs_f32() < 2.0)
                                .unwrap_or(false);

                            let bubble_id = Id::from_u64(999_100_000 + (msg_idx as u64) * 10);
                            let copy_btn_id = Id::from_u64(999_100_000 + (msg_idx as u64) * 10 + 1);
                            let mut copy_clicked = false;

                            let full_text: &str = &msg.content;

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
                                                    .line_height(1.65)
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
                                    .gap(10.0)
                                    .show(|ui| {
                                        // ── Sibling 1: Trajectory Steps ─────────────────────
                                        for (step_idx, step) in msg.trajectory.iter().enumerate() {
                                            let step_id = Id::from_u64(999_200_000 + (msg_idx as u64) * 1000 + (step_idx as u64));
                                            match step {
                                                TrajectoryStep::Exploration(grp) => {
                                                    let chevron = if grp.expanded { "\u{f078}" } else { "\u{f054}" };
                                                    let summary = grp.summary_label();

                                                    let header_row = ui.container()
                                                        .id(step_id)
                                                        .full_width()
                                                        .padding(3.0, 4.0, 3.0, 4.0)
                                                        .radius_all(4.0)
                                                        .hover_bg(colors.highlight.with_alpha(0.5))
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(6.0)
                                                        .show(|ui| {
                                                            if grp.is_active {
                                                                ui.container()
                                                                    .width(5.0)
                                                                    .height(5.0)
                                                                    .radius_all(2.5)
                                                                    .bg(colors.accent)
                                                                    .show(|_| {});
                                                            }
                                                            ui.text(&summary)
                                                                .size(10.5)
                                                                .color(if grp.is_active { colors.text_primary } else { colors.text_muted })
                                                                .show();

                                                            ui.text(chevron)
                                                                .size(7.5)
                                                                .color(colors.text_muted.with_alpha(0.7))
                                                                .show();
                                                        });

                                                    if header_row.hovered {
                                                        ui.cursor_icon = CursorIcon::Pointer;
                                                    }
                                                    if header_row.clicked {
                                                        toggle_trajectory_group = Some((msg_idx, step_idx));
                                                    }

                                                    if grp.expanded {
                                                        ui.container()
                                                            .full_width()
                                                            .padding_left(14.0)
                                                            .column()
                                                            .gap(4.0)
                                                            .show(|ui| {
                                                                for item in &grp.items {
                                                                    match item {
                                                                        ExplorationItem::AnalyzedFile { path, start_line, end_line, symbol, is_peek } => {
                                                                            let path_obj = std::path::Path::new(path);
                                                                            let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());
                                                                            let ext = path_obj.extension().and_then(|s| s.to_str()).unwrap_or("");
                                                                            let (icon_glyph, icon_color) = crate::mini_icons::get_mini_file_icon(&fname.to_lowercase(), &ext.to_lowercase());

                                                                            let action_verb = match (*is_peek, grp.is_active) {
                                                                                (true, true) => "Peeking",
                                                                                (true, false) => "Peeked",
                                                                                (false, true) => "Reading",
                                                                                (false, false) => "Read",
                                                                            };
                                                                            let badge = if *is_peek {
                                                                                " [outline]".to_string()
                                                                            } else if let Some(ref sym) = symbol {
                                                                                format!(" fn {}()", sym)
                                                                            } else {
                                                                                match (start_line, end_line) {
                                                                                    (Some(s), Some(e)) => format!(" #L{}-{}", s, e),
                                                                                    (Some(s), None) => format!(" #L{}", s),
                                                                                    _ => String::new(),
                                                                                }
                                                                            };

                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text(icon_glyph)
                                                                                        .size(10.0)
                                                                                        .color(icon_color)
                                                                                        .show();
                                                                                    ui.text(fname)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();
                                                                                    if !badge.is_empty() {
                                                                                        ui.text(&badge)
                                                                                            .size(9.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                });
                                                                        }
                                                                        ExplorationItem::BatchRead { paths } => {
                                                                            let action_verb = if grp.is_active { "Reading batch" } else { "Read batch" };
                                                                            let list_str = paths
                                                                                .iter()
                                                                                .filter_map(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()))
                                                                                .collect::<Vec<_>>()
                                                                                .join(", ");
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text(&format!("[{}]", list_str))
                                                                                        .size(10.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        ExplorationItem::Searched { query, scope } => {
                                                                            let search_prefix = if grp.is_active { "Searching" } else { "Searched" };
                                                                            let action_verb = if let Some(ref sc) = scope {
                                                                                format!("{} {}", search_prefix, sc)
                                                                            } else {
                                                                                search_prefix.to_string()
                                                                            };
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(&action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text(&format!("\"{}\"", query))
                                                                                        .size(10.0)
                                                                                        .color(colors.text_primary)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        ExplorationItem::ExploredDirectory { path } => {
                                                                            let action_verb = if grp.is_active { "Exploring" } else { "Explored" };
                                                                            let path_obj = std::path::Path::new(path);
                                                                            let dname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text(dname)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        ExplorationItem::ActivatedSkill { name } => {
                                                                            let action_verb = if grp.is_active { "Activating skill" } else { "Activated skill" };
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text(name)
                                                                                        .size(10.5)
                                                                                        .color(colors.accent)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        ExplorationItem::ExecutedCommand { command } => {
                                                                            let action_verb = if grp.is_active { "Running" } else { "Ran" };
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text(command)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_primary)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        ExplorationItem::CheckedDiagnostics => {
                                                                            let action_verb = if grp.is_active { "Checking diagnostics" } else { "Checked diagnostics" };
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        ExplorationItem::Thought { summary, duration_secs } => {
                                                                            let thought_label = match (duration_secs, grp.is_active) {
                                                                                (Some(secs), _) => format!("Thought for {}s", secs),
                                                                                (None, true) => if summary.is_empty() { "Thinking...".to_string() } else { summary.clone() },
                                                                                (None, false) => if summary.is_empty() { "Thought".to_string() } else { summary.clone() },
                                                                            };
                                                                            ui.container()
                                                                                .full_width()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(4.0)
                                                                                .show(|ui| {
                                                                                    ui.text(&thought_label)
                                                                                        .size(10.0)
                                                                                        .italic()
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                    }
                                                                }
                                                            });
                                                    }
                                                }
                                                TrajectoryStep::FileEdit { path, lines_added, lines_removed } => {
                                                    let path_obj = std::path::Path::new(path);
                                                    let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());
                                                    let ext = path_obj.extension().and_then(|s| s.to_str()).unwrap_or("");
                                                    let (icon_glyph, icon_color) = crate::mini_icons::get_mini_file_icon(&fname.to_lowercase(), &ext.to_lowercase());

                                                    ui.container()
                                                        .id(step_id)
                                                        .full_width()
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(5.0)
                                                        .show(|ui| {
                                                            ui.text("Edited")
                                                                .size(10.5)
                                                                .color(colors.text_muted)
                                                                .show();
                                                            ui.text(icon_glyph)
                                                                .size(10.5)
                                                                .color(icon_color)
                                                                .show();
                                                            ui.text(fname)
                                                                .size(10.5)
                                                                .color(colors.text_primary)
                                                                .show();
                                                            if *lines_added > 0 {
                                                                ui.text(&format!("+{}", lines_added))
                                                                    .size(9.5)
                                                                    .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                    .show();
                                                            }
                                                            if *lines_removed > 0 {
                                                                ui.text(&format!("-{}", lines_removed))
                                                                    .size(9.5)
                                                                    .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                                    .show();
                                                            }
                                                        });
                                                }
                                                TrajectoryStep::Command { command } => {
                                                    ui.container()
                                                        .id(step_id)
                                                        .full_width()
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(5.0)
                                                        .show(|ui| {
                                                            ui.text("Ran")
                                                                .size(10.5)
                                                                .color(colors.text_muted)
                                                                .show();
                                                            ui.text("\u{f120}")
                                                                .size(9.5)
                                                                .color(colors.accent)
                                                                .show();
                                                            ui.text(&format!("$ {}", command))
                                                                .size(10.0)
                                                                .monospace()
                                                                .color(colors.text_primary)
                                                                .show();
                                                        });
                                                }
                                                TrajectoryStep::Plan { title, steps, expanded } => {
                                                    let chevron = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                    let plan_header = ui.container()
                                                        .id(step_id)
                                                        .full_width()
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .radius_all(4.0)
                                                        .hover_bg(colors.highlight.with_alpha(0.5))
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(6.0)
                                                        .show(|ui| {
                                                            ui.text("\u{f0ca}")
                                                                .size(9.5)
                                                                .color(colors.accent)
                                                                .show();
                                                            ui.text(title)
                                                                .size(10.5)
                                                                .weight(FontWeight::Medium)
                                                                .color(colors.text_primary)
                                                                .show();
                                                            ui.text(chevron)
                                                                .size(8.0)
                                                                .color(colors.text_muted)
                                                                .show();
                                                        });

                                                    if plan_header.hovered {
                                                        ui.cursor_icon = CursorIcon::Pointer;
                                                    }
                                                    if plan_header.clicked {
                                                        toggle_trajectory_group = Some((msg_idx, step_idx));
                                                    }

                                                    if *expanded {
                                                        ui.container()
                                                            .full_width()
                                                            .padding_left(16.0)
                                                            .column()
                                                            .gap(3.0)
                                                            .show(|ui| {
                                                                for (p_idx, p_step) in steps.iter().enumerate() {
                                                                    ui.container()
                                                                        .row()
                                                                        .valign(Align::Center)
                                                                        .gap(4.0)
                                                                        .show(|ui| {
                                                                            ui.text(&format!("{}.", p_idx + 1))
                                                                                .size(9.5)
                                                                                .color(colors.accent)
                                                                                .show();
                                                                            ui.text(p_step)
                                                                                .size(10.0)
                                                                                .color(colors.text_muted)
                                                                                .show();
                                                                        });
                                                                }
                                                            });
                                                    }
                                                }
                                            }
                                        }

                                        // "Working..." indicator — shown only when no tool steps have started yet
                                        if msg.is_working && full_text.is_empty() && msg.trajectory.is_empty() {
                                            ui.container()
                                                .row()
                                                .valign(Align::Center)
                                                .gap(6.0)
                                                .padding(3.0, 8.0, 3.0, 8.0)
                                                .radius_all(6.0)
                                                .bg(colors.accent.with_alpha(0.08))
                                                .border(colors.accent.with_alpha(0.2), 0.5)
                                                .show(|ui| {
                                                    ui.text("\u{f110}")
                                                        .size(10.0)
                                                        .color(colors.accent)
                                                        .show();
                                                    ui.text("Working...")
                                                        .size(10.5)
                                                        .italic()
                                                        .color(colors.accent)
                                                        .show();
                                                });
                                        }

                                        // ── Sibling 2: Final Summary / Markdown Answer ──────
                                        if !full_text.is_empty() {
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
                                                                            .line_height(1.65)
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
                                                                            .line_height(1.65)
                                                                            .color(colors.text_primary)
                                                                            .max_width(ai_text_w - 18.0)
                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                            .show();
                                                                    });
                                                            }
                                                            MarkdownBlock::CodeBlock { lang, code } => {
                                                                let code_btn_id = Id::from_u64(999_500_000 + (msg_idx as u64) * 1000 + (block_idx as u64));
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
                                                                                    .line_height(1.65)
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
                                                                    .line_height(1.65)
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
                        if let Some((m_idx, s_idx)) = toggle_trajectory_group {
                            if let Some(m) = state.ai_messages.get_mut(m_idx) {
                                if let Some(step) = m.trajectory.get_mut(s_idx) {
                                    match step {
                                        TrajectoryStep::Exploration(ref mut grp) => grp.expanded = !grp.expanded,
                                        TrajectoryStep::Plan { ref mut expanded, .. } => *expanded = !*expanded,
                                        _ => {}
                                    }
                                    ui.needs_redraw = true;
                                }
                            }
                        }

                        if let Some((btn_id, snippet)) = copy_code_snippet {
                            if let Ok(mut cb) = arboard::Clipboard::new() {
                                let _ = cb.set_text(snippet);
                            }
                            state.copied_ai_code_id = Some((btn_id, std::time::Instant::now()));
                            state.set_status("Code snippet copied to clipboard");
                            ui.needs_redraw = true;
                        }

                        if let Some(idx) = copy_target_idx {
                            if let Some(m) = state.ai_messages.get(idx) {
                                if let Ok(mut cb) = arboard::Clipboard::new() {
                                    let _ = cb.set_text(m.content.clone());
                                }
                            }
                            state.copied_ai_msg = Some((idx, std::time::Instant::now()));
                            state.set_status("Copied to clipboard");
                            ui.needs_redraw = true;
                        }
                    }
                });

                // Input box container (docked at the bottom)
                ui.container()
                    .width(ai_panel_w)
                    .height(dock_total_h)
                    .max_height(dock_total_h)
                    .padding(6.0, 10.0, 8.0, 10.0)
                    .column()
                    .gap(6.0)
                    .show(|ui| {
                    let dock_border_color = if is_input_focused {
                        colors.accent.with_alpha(0.7)
                    } else {
                        colors.border.with_alpha(0.4)
                    };

                    let mut target_open_file: Option<std::path::PathBuf> = None;
                    let mut target_reject_file: Option<std::path::PathBuf> = None;
                    let mut action_accept_all = false;
                    let mut action_reject_all = false;

                    // ─── Modified Files Cards & Review Bar (or Standard Quick Toolbar) ───
                    if has_modified_files {
                        // 1. Modified Files Card Container (Single space-saving unified card holding all files)
                        if state.show_modified_files_popup {
                            let card_container_h = (modified_files.len().max(1) as f32 * 24.0 + 8.0).min(180.0);
                            ui.container()
                                .full_width()
                                .height(card_container_h)
                                .radius_all(6.0)
                                .bg(colors.bg_sidebar)
                                .padding(3.0, 6.0, 3.0, 6.0)
                                .scroll_y(true)
                                .column()
                                .gap(2.0)
                                .show(|ui| {
                                    for (idx, item) in modified_files.iter().enumerate() {
                                        let ext = item.file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                                        let (_glyph, icon_color) = crate::mini_icons::get_mini_file_icon(&item.file_name.to_lowercase(), &ext.to_lowercase());

                                        let is_active = state.tabs.get(state.active_tab.unwrap_or(0))
                                            .and_then(|t| t.path.as_ref())
                                            .map(|p| path_matches(p, &item.file_path))
                                            .unwrap_or(false);

                                        let mut reject_this_file = false;
                                        let row_id = ui.id_from(&format!("mod_file_row_{}_{}", idx, item.file_name));
                                        let row = ui.container()
                                            .id(row_id)
                                            .full_width()
                                            .height(22.0)
                                            .radius_all(3.0)
                                            .bg(if is_active { colors.highlight } else { Color::TRANSPARENT })
                                            .hover_bg(colors.highlight)
                                            .padding(0.0, 4.0, 0.0, 4.0)
                                            .clickable(true)
                                            .row()
                                            .valign(Align::Center)
                                            .halign(Align::SpaceBetween)
                                            .show(|ui| {
                                                // Left side: file icon, file name, file path
                                                ui.container()
                                                    .row()
                                                    .valign(Align::Center)
                                                    .gap(7.0)
                                                    .show(|ui| {
                                                        // 1. Language outline box with centered colored dot
                                                        ui.container()
                                                            .width(14.0)
                                                            .height(14.0)
                                                            .radius_all(3.0)
                                                            .border(icon_color, 1.0)
                                                            .halign(Align::Center)
                                                            .valign(Align::Center)
                                                            .show(|ui| {
                                                                ui.container()
                                                                    .width(3.5)
                                                                    .height(3.5)
                                                                    .radius_all(1.75)
                                                                    .bg(icon_color)
                                                                    .show(|_| {});
                                                            });

                                                        // 2. File name
                                                        ui.text(&item.file_name)
                                                            .size(11.5)
                                                            .weight(FontWeight::Medium)
                                                            .color(colors.text_primary)
                                                            .show();

                                                        // 3. File path
                                                        ui.text(&item.rel_path)
                                                            .size(10.5)
                                                            .color(colors.text_dim)
                                                            .show();
                                                    });

                                                // Right side: + and - , cross
                                                ui.container()
                                                    .row()
                                                    .valign(Align::Center)
                                                    .gap(6.0)
                                                    .show(|ui| {
                                                        // +added diff counter (always formatted +N)
                                                        ui.text(&format!("+{}", item.lines_added))
                                                            .size(10.5)
                                                            .weight(FontWeight::Medium)
                                                            .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                            .show();

                                                        // -removed diff counter (always formatted -M)
                                                        ui.text(&format!("-{}", item.lines_removed))
                                                            .size(10.5)
                                                            .weight(FontWeight::Medium)
                                                            .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                            .show();

                                                        // cross
                                                        let cross_id = ui.id_from(&format!("mod_file_cross_{}_{}", idx, item.file_name));
                                                        let cross_btn = ui.container()
                                                            .id(cross_id)
                                                            .clickable(true)
                                                            .padding(1.0, 3.0, 1.0, 3.0)
                                                            .radius_all(2.5)
                                                            .hover_bg(colors.highlight)
                                                            .show(|ui| {
                                                                ui.text("✕")
                                                                    .size(8.5)
                                                                    .color(colors.text_dim)
                                                                    .show();
                                                            });
                                                        if cross_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                                        if cross_btn.clicked {
                                                            reject_this_file = true;
                                                        }
                                                    });
                                            });

                                        if row.hovered && !reject_this_file { ui.cursor_icon = CursorIcon::Pointer; }
                                        if row.clicked && !reject_this_file {
                                            target_open_file = Some(item.file_path.clone());
                                        }
                                        if reject_this_file {
                                            target_reject_file = Some(item.file_path.clone());
                                        }
                                    }
                                });
                        }

                        // 2. Review Bar: ← 📄 N File(s) With Changes               Reject all [Accept all] ⌵
                        ui.container()
                            .full_width()
                            .height(24.0)
                            .padding(0.0, 2.0, 0.0, 2.0)
                            .row()
                            .halign(Align::SpaceBetween)
                            .valign(Align::Center)
                            .show(|ui| {
                                // Left side
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(6.0)
                                    .show(|ui| {
                                        let arrow_btn = ui.container()
                                            .id(Id::from_u64(999999865))
                                            .clickable(true)
                                            .show(|ui| {
                                                ui.text("←")
                                                    .size(13.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                        if arrow_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if arrow_btn.clicked {
                                            state.show_modified_files_popup = !state.show_modified_files_popup;
                                            ui.needs_redraw = true;
                                        }

                                        let file_icon_btn = ui.container()
                                            .clickable(true)
                                            .show(|ui| {
                                                ui.text("\u{f15c}")
                                                    .size(12.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                        if file_icon_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if file_icon_btn.clicked {
                                            state.show_modified_files_popup = !state.show_modified_files_popup;
                                            ui.needs_redraw = true;
                                        }

                                        let count = modified_files.len();
                                        let label = format!("{} {} With Changes", count, if count == 1 { "File" } else { "Files" });
                                        let label_btn = ui.container()
                                            .clickable(true)
                                            .show(|ui| {
                                                ui.text(&label)
                                                    .size(12.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                        if label_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if label_btn.clicked {
                                            state.show_modified_files_popup = !state.show_modified_files_popup;
                                            ui.needs_redraw = true;
                                        }
                                    });

                                // Right side
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(8.0)
                                    .show(|ui| {
                                        let rej_btn = ui.container()
                                            .id(Id::from_u64(999999866))
                                            .clickable(true)
                                            .padding(2.0, 6.0, 2.0, 6.0)
                                            .radius_all(4.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("Reject all")
                                                    .size(12.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                        if rej_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if rej_btn.clicked {
                                            action_reject_all = true;
                                        }

                                        let acc_btn = ui.container()
                                            .id(Id::from_u64(999999867))
                                            .clickable(true)
                                            .padding(3.0, 9.0, 3.0, 9.0)
                                            .radius_all(5.0)
                                            .bg(Color { r: 0.0, g: 122.0 / 255.0, b: 255.0 / 255.0, a: 1.0 })
                                            .hover_bg(Color { r: 24.0 / 255.0, g: 138.0 / 255.0, b: 255.0 / 255.0, a: 1.0 })
                                            .show(|ui| {
                                                ui.text("Accept all")
                                                    .size(12.0)
                                                    .weight(FontWeight::SemiBold)
                                                    .color(Color::WHITE)
                                                    .show();
                                            });
                                        if acc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if acc_btn.clicked {
                                            action_accept_all = true;
                                        }

                                        let chevron_btn = ui.container()
                                            .clickable(true)
                                            .show(|ui| {
                                                ui.text(if state.show_modified_files_popup { "⌵" } else { "⌃" })
                                                    .size(11.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                        if chevron_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if chevron_btn.clicked {
                                            state.show_modified_files_popup = !state.show_modified_files_popup;
                                            ui.needs_redraw = true;
                                        }
                                    });
                            });
                    } else {
                        // Standard quick toolbar when no modified files
                        ui.container()
                            .full_width()
                            .height(20.0)
                            .padding(0.0, 2.0, 4.0, 2.0)
                            .row()
                            .halign(Align::SpaceBetween)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(8.0)
                                    .show(|ui| {
                                        // 1. File icon
                                        let file_id = Id::from_u64(999999860);
                                        let file_btn = ui.container()
                                            .id(file_id)
                                            .clickable(true)
                                            .padding(2.0, 3.0, 2.0, 3.0)
                                            .radius_all(3.0)
                                            .bg(if state.show_modified_files_popup { colors.highlight } else { Color::TRANSPARENT })
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("\u{f15c}")
                                                    .size(12.5)
                                                    .color(if state.show_modified_files_popup { colors.accent } else { colors.text_muted })
                                                    .show();
                                            });
                                        if file_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if file_btn.clicked {
                                            state.show_modified_files_popup = !state.show_modified_files_popup;
                                            ui.needs_redraw = true;
                                        }

                                        // 2. Terminal icon
                                        let term_id = Id::from_u64(999999861);
                                        ui.container()
                                            .id(term_id)
                                            .clickable(true)
                                            .padding(2.0, 3.0, 2.0, 3.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("\u{f120}")
                                                    .size(11.5)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });

                                        // 3. Inbox icon
                                        let inbox_id = Id::from_u64(999999862);
                                        ui.container()
                                            .id(inbox_id)
                                            .clickable(true)
                                            .padding(2.0, 3.0, 2.0, 3.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .row()
                                            .valign(Align::Center)
                                            .gap(1.0)
                                            .show(|ui| {
                                                ui.text("\u{f2d0}")
                                                    .size(12.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                                ui.container()
                                                    .width(4.5)
                                                    .height(4.5)
                                                    .radius_all(2.25)
                                                    .bg(Color { r: 59.0 / 255.0, g: 130.0 / 255.0, b: 246.0 / 255.0, a: 1.0 })
                                                    .show(|_| {});
                                            });

                                        // 4. Chrome icon
                                        let chrome_id = Id::from_u64(999999863);
                                        ui.container()
                                            .id(chrome_id)
                                            .clickable(true)
                                            .padding(2.0, 3.0, 2.0, 3.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("\u{f268}")
                                                    .size(12.5)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                    });

                            });
                    }

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
                            let placeholder = if ai_panel_w >= 240.0 {
                                "Ask anything, @ to mention, / for actions"
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
                                    let is_processing = state.ai_messages.last().map(|m| m.is_working).unwrap_or(false);
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
                                            // Cancel: clear the signal queue so drain_ai_updates stops processing
                                            state.ai_update_signal.set(Vec::new());
                                            // Also send a synthetic Done update so is_working is reset on next frame
                                            state.ai_update_signal.with_mut(|q| q.push(
                                                crate::state::types::AiUiUpdate::Done {
                                                    reply: "Request stopped.".to_string(),
                                                    session_json: None,
                                                }
                                            ));
                                            if let Some(last) = state.ai_messages.iter_mut().rev().find(|m| !m.is_user) {
                                                last.is_working = false;
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

                    if action_accept_all {
                        state.accept_all_reviews();
                        state.show_modified_files_popup = false;
                        ui.needs_redraw = true;
                    }
                    if action_reject_all {
                        state.reject_all_reviews();
                        state.show_modified_files_popup = false;
                        ui.needs_redraw = true;
                    }
                    if let Some(path) = target_reject_file {
                        state.reject_review_for_path(&path);
                        ui.needs_redraw = true;
                    }
                    if let Some(path) = target_open_file {
                        state.open_file(&path);
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
