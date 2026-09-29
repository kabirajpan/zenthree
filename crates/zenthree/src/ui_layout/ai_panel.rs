use crate::state::{ZenthreeState, AiModel, AiProvider, AgentMode, TrajectoryStep, ExplorationItem, path_matches, SessionSummary, AiSettingsTab, AiMessageBg};
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context, Placement, PopoverAlign, Wrap};
use crate::theme::ThemeColors;

const AGENT_MODE_BTN_ID: Id = Id::from_u64(999999885);
const MODEL_PICKER_BTN_ID: Id = Id::from_u64(999999886);
const AI_MORE_BTN_ID: Id = Id::from_u64(999999887);



fn draw_ai_chat_header(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    header_w: f32,
    colors: &crate::theme::ThemeColors,
    _panel_bg: Color,
) {
    const HEADER_H: f32 = 36.0;
    let header_bg = if state.glassmorphism {
        Color::TRANSPARENT
    } else {
        colors.bg_sidebar
    };
    ui.container()
        .width(header_w)
        .height(HEADER_H)
        .bg(header_bg)
        .padding_left(8.0)
        .padding_right(10.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .no_wrap()
        .show(|ui| {
            // Left: Expand/Collapse Arrow + Title
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    let is_expanded = state.ai_panel_expanded;
                    let btn_w = 22.0;
                    let btn_h = 22.0;
                    let expand_btn = ui.container()
                        .clickable(true)
                        .width(btn_w)
                        .height(btn_h)
                        .radius_all(4.0)
                        .bg(if is_expanded { colors.highlight } else { Color::TRANSPARENT })
                        .hover_bg(colors.highlight)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let icon = if is_expanded {
                                "\u{f514}" // collapse
                            } else {
                                "\u{f515}" // expand
                            };
                            let icon_color = if is_expanded {
                                colors.accent
                            } else {
                                colors.text_muted
                            };
                            ui.text(icon)
                                .size(12.0)
                                .color(icon_color)
                                .show();
                        });
                    if expand_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if expand_btn.clicked {
                        state.ai_panel_expanded = !state.ai_panel_expanded;
                        state.save_persistent_state();
                        ui.needs_redraw = true;
                    }

                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(7.0)
                        .show(|ui| {
                            ui.container()
                                .width(7.0)
                                .height(7.0)
                                .radius_all(3.5)
                                .bg(colors.accent)
                                .show(|_| {});

                            ui.text("AI Assistant")
                                .size(12.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });
                });

            // Right Actions (New, History, 3-dot Menu, Close)
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(3.0)
                .show(|ui| {
                    let new_btn = ui.container()
                        .clickable(true)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(4.0)
                        .hover_bg(colors.highlight)
                        .row()
                        .valign(Align::Center)
                        .gap(3.0)
                        .show(|ui| {
                            ui.text("\u{f067}") // fa-plus
                                .size(9.0)
                                .color(colors.text_muted)
                                .show();
                            ui.text("New")
                                .size(10.5)
                                .color(colors.text_muted)
                                .show();
                        });
                    if new_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if new_btn.clicked {
                        state.start_new_ai_session();
                        ui.needs_redraw = true;
                    }

                    let history_btn = ui.container()
                        .clickable(true)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(4.0)
                        .bg(if state.ai_sessions_visible { colors.highlight } else { Color::TRANSPARENT })
                        .hover_bg(colors.highlight)
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            ui.text("\u{f1da}") // fa-clock-rotate-left
                                .size(10.0)
                                .color(if state.ai_sessions_visible { colors.accent } else { colors.text_muted })
                                .show();
                            ui.text("History")
                                .size(10.5)
                                .color(if state.ai_sessions_visible { colors.text_primary } else { colors.text_muted })
                                .show();
                        });
                    if history_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if history_btn.clicked {
                        state.ai_sessions_visible = !state.ai_sessions_visible;
                        if state.ai_sessions_visible {
                            state.refresh_ai_sessions();
                        }
                        ui.needs_redraw = true;
                    }

                    // Floating Thinking Orb Window Toggle Button
                    let is_orb_active = state.thinking_orbs_demo_open;
                    let orb_btn = ui.container()
                        .clickable(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .bg(if is_orb_active { colors.highlight } else { Color::TRANSPARENT })
                        .hover_bg(colors.highlight)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f111}") // fa-circle
                                .size(9.0)
                                .color(if is_orb_active {
                                    colors.accent
                                } else {
                                    colors.text_muted
                                })
                                .show();
                        });
                    if orb_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if orb_btn.clicked {
                        state.thinking_orbs_demo_open = !state.thinking_orbs_demo_open;
                        if state.thinking_orbs_demo_open {
                            state.set_status("Thinking Orb Showcase Window activated");
                        }
                        ui.needs_redraw = true;
                    }

                    let more_btn = ui.container()
                        .raw_id(AI_MORE_BTN_ID)
                        .clickable(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .bg(if state.ai_menu_open { colors.highlight } else { Color::TRANSPARENT })
                        .hover_bg(colors.highlight)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f142}") // fa-ellipsis-vertical
                                .size(11.0)
                                .color(if state.ai_menu_open { colors.accent } else { colors.text_muted })
                                .show();
                        });
                    if more_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if more_btn.clicked {
                        state.ai_menu_open = !state.ai_menu_open;
                        ui.needs_redraw = true;
                    }

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
                    if close_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if close_btn.clicked {
                        state.cancel_active_ai();
                        state.ai_panel_visible = false;
                        state.save_persistent_state();
                        ui.needs_redraw = true;
                    }
                });
        });
}

pub fn draw_ai_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let ai_panel_w = width;
    let ai_id = Id::from_u64(999999801);
    let ai_input_id = ui.id_from("ai_prompt_text_area");

    // ─── Resizing Splitter Handle ─────────────────────────────────────────
    let splitter_x = ui.cursor_x;
    let splitter_y = ui.cursor_y;

    if !state.ai_panel_expanded {
        let handle_w = 6.0;
        let handle_id = Id::from_u64(999998899);
        let is_blocked = ui.modal_active || state.is_modal_or_dialog_open() || ui.is_occluded(handle_id, ui.mouse_x, ui.mouse_y);
        let is_splitter_hovered = !is_blocked && ui.mouse_in_rect(
            splitter_x - handle_w / 2.0,
            splitter_y,
            handle_w,
            height,
        );

        if is_blocked {
            state.active_resize_ai_panel = false;
        }

        if is_splitter_hovered || state.active_resize_ai_panel {
            ui.cursor_icon = CursorIcon::ColResize;
            ui.resize_active = true;
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
            colors.accent.with_alpha(0.8)
        } else {
            colors.border
        };

        let edge_thickness = if state.active_resize_ai_panel || is_splitter_hovered { 2.0 } else { 1.0 };
        let edge_draw_x = if edge_thickness == 1.0 { splitter_x.round() } else { (splitter_x - 1.0).round() };

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
    }

    // ─── Main AI Agent Panel ──────────────────────────────────────────────
    let panel_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 0.90).clamp(0.30, 0.90))
    } else {
        match state.ai_settings.effective_message_bg() {
            AiMessageBg::DeepDark => colors.bg_base,
            AiMessageBg::Sidebar => colors.bg_sidebar,
        }
    };

    const HEADER_H: f32 = 36.0;

    if state.ai_panel_expanded && state.ai_sessions_visible {
        // Expanded Mode with History OPEN: Side-by-Side with full-height native drawer!
        const HISTORY_SIDEBAR_W: f32 = 280.0;
        let hist_w = HISTORY_SIDEBAR_W.min(ai_panel_w * 0.45);
        let chat_w = (ai_panel_w - hist_w).max(200.0);

        let panel_container = ui.container()
            .id(ai_id)
            .width(ai_panel_w)
            .height(height)
            .max_height(height)
            .clip(true)
            .bg(panel_bg);

        panel_container
            .row()
            .show(|ui| {
                // Left: Active Chat Area
                ui.container()
                    .width(chat_w)
                    .height(height)
                    .max_height(height)
                    .column()
                    .clip(true)
                    .show(|ui| {
                        draw_ai_chat_header(ui, state, chat_w, &colors, panel_bg);

                        ui.container()
                            .width(chat_w)
                            .height(1.0)
                            .bg(if state.glassmorphism { colors.border.with_alpha(0.25) } else { colors.border })
                            .show(|_| {});

                        let chat_content_h = height - HEADER_H - 1.0;
                        draw_ai_chat_content(ui, state, chat_w, chat_content_h, &colors, ai_input_id);
                    });

                // Full-height vertical divider line between Chat and History sidebar
                ui.container()
                    .width(1.0)
                    .height(height)
                    .bg(colors.border)
                    .show(|_| {});

                // Right: Session History Sidebar Panel (Native Full Height!)
                draw_session_history_panel(ui, state, hist_w - 1.0, height, &colors);
            });
    } else if state.ai_sessions_visible {
        // Docked Mode with History OPEN: In-place full-height replacement
        draw_session_history_panel(ui, state, ai_panel_w, height, &colors);
    } else {
        // History CLOSED (Docked or Expanded): Chat takes full panel width
        let panel_container = ui.container()
            .id(ai_id)
            .width(ai_panel_w)
            .height(height)
            .max_height(height)
            .clip(true)
            .bg(panel_bg);

        panel_container
            .column()
            .show(|ui| {
                draw_ai_chat_header(ui, state, ai_panel_w, &colors, panel_bg);

                ui.container()
                    .width(ai_panel_w)
                    .height(1.0)
                    .bg(if state.glassmorphism { colors.border.with_alpha(0.25) } else { colors.border })
                    .show(|_| {});

                let content_h = height - HEADER_H - 1.0;
                draw_ai_chat_content(ui, state, ai_panel_w, content_h, &colors, ai_input_id);
            });
    }

    // ─── Floating Menu Panel: AI 3-Dot More Menu ──────────────────────────
    if state.ai_menu_open {
        let menu_w = 145.0;
        let menu_h = 92.0;

        if let Some(anchor) = ui.next_screen_layout_cache.get(&AI_MORE_BTN_ID)
            .or_else(|| ui.screen_layout_cache.get(&AI_MORE_BTN_ID)).copied()
        {
            let (mx, my) = zenthra_widgets::compute_placement_pos(
                anchor,
                menu_w,
                menu_h,
                Placement::Bottom,
                PopoverAlign::End,
                4.0,
                false,
                ui.width,
                ui.height,
            );
            let mouse_in_menu = ui.mouse_in_rect(mx, my, menu_w, menu_h);
            let mouse_in_trigger = ui.mouse_in_rect(anchor.origin.x, anchor.origin.y, anchor.size.width, anchor.size.height);
            if ui.clicked && !mouse_in_menu && !mouse_in_trigger {
                state.ai_menu_open = false;
                ui.needs_redraw = true;
            }
        }

        if state.ai_menu_open {
            let popover_bg = if state.glassmorphism {
                colors.bg_sidebar.with_alpha((state.glass_opacity * 0.90).clamp(0.40, 0.95))
            } else {
                colors.bg_sidebar
            };
            let mut popover_builder = ui.popover(AI_MORE_BTN_ID)
                .placement(Placement::Bottom)
                .align(PopoverAlign::End)
                .gap(4.0)
                .auto_flip(false)
                .width(menu_w)
                .height(menu_h)
                .bg(popover_bg)
                .border(colors.border.with_alpha(0.8), 1.0)
                .radius_all(6.0)
                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, 4.0, 16.0)
                .padding(3.0, 3.0, 3.0, 3.0);

            if state.glassmorphism {
                popover_builder = popover_builder.backdrop_blur(state.blur_strength.max(4.0));
            }

            popover_builder.show(|ui| {
                    // Option 1: MCP Servers (UI only)
                    let mcp_btn = ui.container()
                        .full_width()
                        .padding(4.0, 7.0, 4.0, 7.0)
                        .radius_all(3.5)
                        .hover_bg(colors.highlight)
                        .clickable(true)
                        .row()
                        .valign(Align::Center)
                        .gap(7.0)
                        .show(|ui| {
                            ui.text("\u{f1e6}") // plug icon
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();
                            ui.text("MCP")
                                .size(11.0)
                                .color(colors.text_primary)
                                .show();
                        });
                    if mcp_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if mcp_btn.clicked {
                        state.ai_menu_open = false;
                        state.set_status("MCP Servers (UI placeholder)");
                        ui.needs_redraw = true;
                    }

                    // Option 2: Providers (UI only)
                    let prov_btn = ui.container()
                        .full_width()
                        .padding(4.0, 7.0, 4.0, 7.0)
                        .radius_all(3.5)
                        .hover_bg(colors.highlight)
                        .clickable(true)
                        .row()
                        .valign(Align::Center)
                        .gap(7.0)
                        .show(|ui| {
                            ui.text("\u{f233}") // server icon
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();
                            ui.text("Provider")
                                .size(11.0)
                                .color(colors.text_primary)
                                .show();
                        });
                    if prov_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if prov_btn.clicked {
                        state.ai_menu_open = false;
                        state.set_status("AI Providers (UI placeholder)");
                        ui.needs_redraw = true;
                    }

                    // Option 3: Settings (opens AI & Agent Settings modal!)
                    let set_btn = ui.container()
                        .full_width()
                        .padding(4.0, 7.0, 4.0, 7.0)
                        .radius_all(3.5)
                        .hover_bg(colors.highlight)
                        .clickable(true)
                        .row()
                        .valign(Align::Center)
                        .gap(7.0)
                        .show(|ui| {
                            ui.text("\u{f013}") // gear icon
                                .size(10.0)
                                .color(colors.accent)
                                .show();
                            ui.text("Settings")
                                .size(11.0)
                                .weight(FontWeight::Medium)
                                .color(colors.text_primary)
                                .show();
                        });
                    if set_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if set_btn.clicked {
                        ui.clicked = false;
                        state.ai_menu_open = false;
                        state.ai_settings.open = true;
                        state.ai_settings.just_opened = true;
                        state.ai_settings.active_tab = AiSettingsTab::Appearance;
                        ui.needs_redraw = true;
                    }
                });
        }
    }

    // ─── Floating Menu Panel: Agent Mode ─────────────────────────────────
    if state.agent_mode_menu_open {
        let menu_w = 138.0;
        let menu_h = 62.0;

        if let Some(anchor) = ui.next_screen_layout_cache.get(&AGENT_MODE_BTN_ID)
            .or_else(|| ui.screen_layout_cache.get(&AGENT_MODE_BTN_ID)).copied()
        {
            let (mx, my) = zenthra_widgets::compute_placement_pos(
                anchor,
                menu_w,
                menu_h,
                Placement::Top,
                PopoverAlign::Start,
                3.0,
                false,
                ui.width,
                ui.height,
            );
            let mouse_in_menu = ui.mouse_in_rect(mx, my, menu_w, menu_h);
            let mouse_in_trigger = ui.mouse_in_rect(anchor.origin.x, anchor.origin.y, anchor.size.width, anchor.size.height);
            if ui.clicked && !mouse_in_menu && !mouse_in_trigger {
                state.agent_mode_menu_open = false;
                ui.needs_redraw = true;
            }
        }

        if state.agent_mode_menu_open {
            ui.popover(AGENT_MODE_BTN_ID)
                .placement(Placement::Top)
                .align(PopoverAlign::Start)
                .gap(3.0)
                .auto_flip(false)
                .width(menu_w)
                .height(menu_h)
                .bg(if colors.bg_base.r > 0.5 {
                    colors.bg_editor
                } else {
                    Color::rgb(22.0 / 255.0, 22.0 / 255.0, 25.0 / 255.0)
                })
                .border(if colors.bg_base.r > 0.5 {
                    colors.border
                } else {
                    Color::rgb(45.0 / 255.0, 45.0 / 255.0, 54.0 / 255.0)
                }, 1.0)
                .radius_all(6.0)
                .shadow(
                    Color::rgba(0.0, 0.0, 0.0, if colors.bg_base.r > 0.5 { 0.18 } else { 0.65 }),
                    0.0,
                    4.0,
                    16.0,
                )
                .padding(3.0, 3.0, 3.0, 3.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
                        .column()
                        .gap(1.5)
                        .show(|ui| {
                            for mode in AgentMode::ALL {
                                let is_sel = state.agent_mode == mode;
                                let row_id = ui.id_from(&format!("pop_agent_mode_{:?}", mode));
                                let row = ui.container()
                                    .id(row_id)
                                    .full_width()
                                    .height(26.0)
                                    .padding(2.0, 6.0, 2.0, 6.0)
                                    .radius_all(3.5)
                                    .bg(if is_sel { colors.accent.with_alpha(0.12) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.container().row().gap(5.0).valign(Align::Center).show(|ui| {
                                            ui.text(mode.icon())
                                                .size(10.0)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.text(mode.display_name())
                                                .size(9.5)
                                                .weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                                .color(if is_sel { colors.accent } else { colors.text_primary })
                                                .show();
                                        });
                                        if is_sel {
                                            ui.text("\u{f00c}").size(8.5).color(colors.accent).show();
                                        }
                                    });
                                if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if row.clicked {
                                    ui.clicked = false;
                                    state.agent_mode = mode;
                                    state.agent_mode_menu_open = false;
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });
        }
    }

    // ─── Floating Menu Panel: Model Selector ──────────────────────────────
    if state.ai_model_menu_open {
        let menu_w = 180.0;
        let total_item_h = AiModel::ALL.len() as f32 * 25.5 + 40.0;
        let max_allowed_h = (ui.height - 110.0).max(250.0);
        let menu_h = total_item_h.min(340.0).min(max_allowed_h);
        let is_light = colors.bg_base.r > 0.5;
        let menu_bg = if is_light {
            colors.bg_editor
        } else {
            Color::rgb(22.0 / 255.0, 22.0 / 255.0, 25.0 / 255.0)
        };
        let menu_border = if is_light {
            colors.border
        } else {
            Color::rgb(45.0 / 255.0, 45.0 / 255.0, 54.0 / 255.0)
        };
        let menu_shadow = if is_light {
            Color::rgba(0.0, 0.0, 0.0, 0.18)
        } else {
            Color::rgba(0.0, 0.0, 0.0, 0.65)
        };

        if let Some(anchor) = ui.next_screen_layout_cache.get(&MODEL_PICKER_BTN_ID)
            .or_else(|| ui.screen_layout_cache.get(&MODEL_PICKER_BTN_ID)).copied()
        {
            let (mx, my) = zenthra_widgets::compute_placement_pos(
                anchor,
                menu_w,
                menu_h,
                Placement::Top,
                PopoverAlign::Start,
                3.0,
                false,
                ui.width,
                ui.height,
            );
            let mouse_in_menu = ui.mouse_in_rect(mx, my, menu_w, menu_h);
            let mouse_in_trigger = ui.mouse_in_rect(anchor.origin.x, anchor.origin.y, anchor.size.width, anchor.size.height);
            if ui.clicked && !mouse_in_menu && !mouse_in_trigger {
                state.ai_model_menu_open = false;
                ui.needs_redraw = true;
            }
        }

        if state.ai_model_menu_open {
            let mut select_model: Option<AiModel> = None;
            ui.popover(MODEL_PICKER_BTN_ID)
                .placement(Placement::Top)
                .align(PopoverAlign::Start)
                .gap(3.0)
                .auto_flip(false)
                .width(menu_w)
                .height(menu_h)
                .bg(menu_bg)
                .border(menu_border, 1.0)
                .radius_all(6.0)
                .shadow(menu_shadow, 0.0, 4.0, 16.0)
                .padding(3.0, 3.0, 3.0, 3.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
                        .height(menu_h - 6.0)
                        .column()
                        .gap(1.2)
                        .scroll_y(true)
                        .show(|ui| {
                            let mut last_provider: Option<AiProvider> = None;
                            for model in AiModel::ALL {
                                if last_provider != Some(model.provider()) {
                                    let label = match model.provider() {
                                        AiProvider::Antigravity => "GOOGLE ANTIGRAVITY (EXTERNAL AGENT)",
                                        AiProvider::Groq => "GROQ",
                                        AiProvider::Gemini => "GEMINI",
                                    };
                                    if last_provider.is_some() {
                                        ui.container().height(2.5).show(|_| {});
                                    }
                                    ui.container().padding(2.0, 5.0, 1.0, 5.0).show(|ui| {
                                        ui.text(label)
                                            .size(7.5)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_muted.with_alpha(0.65))
                                            .show();
                                    });
                                    last_provider = Some(model.provider());
                                }

                                let is_selected = state.selected_ai_model == model;
                                let row_id = ui.id_from(&format!("pop_model_opt_{}", model.model_id()));
                                let row = ui.container()
                                    .id(row_id)
                                    .full_width()
                                    .height(24.0)
                                    .padding(2.0, 6.0, 2.0, 6.0)
                                    .radius_all(3.5)
                                    .bg(if is_selected { colors.accent.with_alpha(0.10) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        ui.text(model.display_name())
                                            .size(9.5)
                                            .weight(if is_selected { FontWeight::Medium } else { FontWeight::Regular })
                                            .color(if is_selected { colors.accent } else { colors.text_primary })
                                            .show();
                                        if is_selected {
                                            ui.text("\u{f00c}")
                                                .size(8.5)
                                                .color(colors.accent)
                                                .show();
                                        }
                                    });

                                if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if row.clicked {
                                    ui.clicked = false;
                                    select_model = Some(model);
                                }
                            }
                        });
                });

            if let Some(model) = select_model {
                state.selected_ai_model = model;
                state.ai_model_menu_open = false;
                state.ai_session_json = None;
                state.save_persistent_state();
                ui.needs_redraw = true;
            }
        }
    }
}

fn draw_ai_chat_content(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    chat_w: f32,
    content_h: f32,
    colors: &crate::theme::ThemeColors,
    ai_input_id: Id,
) {
    // ─── Dynamic Height & Width Calculations ───────────────────────
    let is_input_focused = ui.focused_id == Some(ai_input_id);
    const MIN_PROMPT_H: f32 = 32.0;
    const MAX_PROMPT_H: f32 = 120.0;
    const MAX_INPUT_W: f32 = 684.0; // 90% of previous 760px cap
    let input_dock_w = (chat_w - 20.0).min(MAX_INPUT_W).max(120.0);

    // Immediately track visual wrapped lines while typing continuously without newline
    let chars_per_line = ((input_dock_w - 24.0).max(60.0) / 6.6).max(8.0);
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

            {
                let mut upsert_file = |path: std::path::PathBuf, fname: String, rel: String, added: usize, removed: usize| {
                    if let Some(existing) = modified_files.iter_mut().find(|item| {
                        path_matches(&item.file_path, &path) || item.file_name == fname
                    }) {
                        if added > 0 {
                            existing.lines_added = existing.lines_added.max(added);
                        }
                        if removed > 0 {
                            existing.lines_removed = existing.lines_removed.max(removed);
                        }
                        if existing.rel_path.is_empty() && !rel.is_empty() {
                            existing.rel_path = rel;
                        }
                    } else {
                        modified_files.push(ModifiedFileItem {
                            file_path: path,
                            file_name: fname,
                            rel_path: rel,
                            lines_added: added,
                            lines_removed: removed,
                        });
                    }
                };

                // 1. From pending reviews
                for rev in &state.pending_reviews {
                    let fname = rev.path.file_name().and_then(|n| n.to_str()).unwrap_or("file").to_string();
                    let rel = rev.path.strip_prefix(&state.current_dir).unwrap_or(&rev.path).to_string_lossy().to_string();
                    let mut added = rev.total_added_lines();
                    let removed = rev.total_removed_lines();
                    if added == 0 && (rev.is_new_file || removed == 0) && !rev.proposed_content.is_empty() {
                        added = rev.proposed_content.lines().count().max(1);
                    }
                    upsert_file(rev.path.clone(), fname, rel, added, removed);
                }

                // 2. From tabs
                for tab in &state.tabs {
                    if tab.is_dirty || tab.lines_added > 0 || tab.lines_removed > 0 {
                        if let Some(ref tp) = tab.path {
                            let fname = tab.name.clone();
                            let rel = tp.strip_prefix(&state.current_dir).unwrap_or(tp).to_string_lossy().to_string();
                            upsert_file(tp.clone(), fname, rel, tab.lines_added, tab.lines_removed);
                        }
                    }
                }

                // 3. From AI trajectory steps
                for msg in state.ai_messages.iter().rev() {
                    for step in &msg.trajectory {
                        if let TrajectoryStep::FileEdit { path, lines_added, lines_removed } = step {
                            let pb = std::path::PathBuf::from(path);
                            let full_p = if pb.is_absolute() { pb.clone() } else { state.current_dir.join(&pb) };
                            let fname = full_p.file_name().and_then(|n| n.to_str()).unwrap_or(path).to_string();
                            let rel = full_p.strip_prefix(&state.current_dir).unwrap_or(&full_p).to_string_lossy().to_string();
                            let mut added = *lines_added;
                            let removed = *lines_removed;
                            if added == 0 && removed == 0 {
                                if let Ok(content) = std::fs::read_to_string(&full_p) {
                                    let count = content.lines().count();
                                    if count > 0 {
                                        added = count;
                                    }
                                }
                            }
                            upsert_file(full_p, fname, rel, added, removed);
                        }
                    }
                }

                // 4. Guarantee line counts for any modified file still at 0
                for item in &mut modified_files {
                    if item.lines_added == 0 && item.lines_removed == 0 {
                        if let Some(rev) = state.pending_reviews.iter().find(|r| path_matches(&r.path, &item.file_path)) {
                            let a = rev.total_added_lines();
                            let r = rev.total_removed_lines();
                            if a > 0 || r > 0 {
                                item.lines_added = a;
                                item.lines_removed = r;
                            } else if !rev.proposed_content.is_empty() {
                                item.lines_added = rev.proposed_content.lines().count().max(1);
                            }
                        }
                    }
                    if item.lines_added == 0 && item.lines_removed == 0 {
                        if let Ok(content) = std::fs::read_to_string(&item.file_path) {
                            let count = content.lines().count();
                            if count > 0 {
                                item.lines_added = count;
                            }
                        }
                    }
                    if item.lines_added == 0 && item.lines_removed == 0 {
                        item.lines_added = 1;
                    }
                }
            }
            let has_modified_files = !modified_files.is_empty();

            let modified_section_h = if state.show_modified_files_popup && has_modified_files {
                let file_list_h = (modified_files.len().max(1) as f32 * 22.0 + 8.0).min(140.0);
                file_list_h + 6.0 + 20.0
            } else {
                20.0
            };

            // Dock height: prompt box + bottom row/padding + toolbar/modified section + generous floating bottom gap
            const DOCK_TOP_PAD: f32 = 6.0;
            const BOTTOM_DOCK_GAP: f32 = 24.0;
            let dock_inner_h = prompt_box_h + modified_section_h + 6.0 + 48.0;
            let dock_total_h = dock_inner_h + DOCK_TOP_PAD + BOTTOM_DOCK_GAP + 6.0;
            let msgs_h = (content_h - dock_total_h).max(30.0);

            // ─── Messages Scroll Area ────────────────────────────────────
            let panel_bg = match state.ai_settings.effective_message_bg() {
                AiMessageBg::DeepDark => colors.bg_base,
                AiMessageBg::Sidebar => colors.bg_sidebar,
            };
            let msgs_scroll_id = Id::from_u64(999999802);
            ui.container()
                .id(msgs_scroll_id)
                .width(chat_w)
                .height(msgs_h)
                .max_height(msgs_h)
                .clip(true)
                .bg(if state.glassmorphism { Color::TRANSPARENT } else { panel_bg })
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
                        let mut toggle_thoughts_idx: Option<usize> = None;
                        let mut toggle_trajectory_group: Option<(usize, usize)> = None;
                        let mut copy_target_idx: Option<usize> = None;
                        let mut copy_code_snippet: Option<(u64, String)> = None;
                        let mut chat_open_file: Option<std::path::PathBuf> = None;
                        let mut open_implementation_plan = false;
                        let mut proceed_with_plan = false;

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
                                let bubble_max_w = (chat_w - 48.0).min(MAX_INPUT_W).max(80.0);
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

                                let is_short_single_line = newline_count == 0
                                    && full_text.len() <= 45
                                    && (full_text.len() as f32 * 6.5 + 40.0) <= bubble_max_w;

                                ui.container()
                                    .full_width()
                                    .row()
                                    .halign(Align::Right)
                                    .padding(2.0, 0.0, 2.0, 0.0)
                                    .show(|ui| {
                                        let user_bg = Color::rgb(28.0 / 255.0, 28.0 / 255.0, 32.0 / 255.0); // #1c1c20
                                        let user_hover_bg = Color::rgb(32.0 / 255.0, 32.0 / 255.0, 38.0 / 255.0); // #202026
                                        let user_border = Color::rgb(42.0 / 255.0, 42.0 / 255.0, 48.0 / 255.0); // #2a2a30
                                        let user_text_col = Color::rgb(222.0 / 255.0, 222.0 / 255.0, 227.0 / 255.0); // #dedee3

                                        let bubble_res = if is_short_single_line {
                                            ui.container()
                                                .id(bubble_id)
                                                .max_width(bubble_max_w)
                                                .padding(5.0, 12.0, 5.0, 12.0)
                                                .bg(user_bg)
                                                .hover_bg(user_hover_bg)
                                                .border(user_border, 1.0)
                                                .radius_all(14.0)
                                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.35), 0.0, 2.0, 6.0)
                                                .row()
                                                .valign(Align::Center)
                                                .gap(8.0)
                                                .show(|ui| {
                                                    ui.text(display_text)
                                                        .size(11.5)
                                                        .line_height(1.35)
                                                        .color(user_text_col)
                                                        .show();

                                                    let copy_btn = ui.container()
                                                        .id(copy_btn_id)
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .radius_all(4.0)
                                                        .hover_bg(user_hover_bg)
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(3.0)
                                                        .show(|ui| {
                                                             ui.text(if is_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                .size(9.0)
                                                                .color(if is_copied { colors.accent } else { Color::rgb(107.0 / 255.0, 109.0 / 255.0, 118.0 / 255.0) })
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
                                                })
                                        } else {
                                            ui.container()
                                                .id(bubble_id)
                                                .max_width(bubble_max_w)
                                                .padding(7.0, 12.0, 7.0, 12.0)
                                                .bg(user_bg)
                                                .hover_bg(user_hover_bg)
                                                .border(user_border, 1.0)
                                                .radius_all(14.0)
                                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.35), 0.0, 2.0, 6.0)
                                                .column()
                                                .halign(Align::Right)
                                                .gap(4.0)
                                                .show(|ui| {
                                                    ui.text(display_text)
                                                        .size(11.5)
                                                        .line_height(1.65)
                                                        .color(user_text_col)
                                                        .max_width(user_text_w)
                                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                                        .show();

                                                    ui.container()
                                                        .row()
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
                                                                .hover_bg(user_hover_bg)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(3.0)
                                                                .show(|ui| {
                                                                    ui.text(if is_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                        .size(9.0)
                                                                        .color(if is_copied { colors.accent } else { Color::rgb(107.0 / 255.0, 109.0 / 255.0, 118.0 / 255.0) })
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
                                                })
                                        };

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
                                // ─── Assistant Message (full-width, clean container) ───
                                let ai_text_w = (chat_w - 44.0).min(MAX_INPUT_W + 40.0).max(60.0);
                                let code_box_w = ai_text_w.max(50.0);

                                ui.container()
                                    .full_width()
                                    .id(bubble_id)
                                    .padding(4.0, 4.0, 6.0, 4.0)
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

                                                    let mut exp_card = ui.container()
                                                        .full_width()
                                                        .column()
                                                        .gap(4.0);

                                                    if grp.is_active {
                                                        exp_card = exp_card
                                                            .bg(Color::rgb(22.0 / 255.0, 22.0 / 255.0, 25.0 / 255.0)) // #161619
                                                            .border(Color::rgb(38.0 / 255.0, 38.0 / 255.0, 44.0 / 255.0), 1.0) // #26262c
                                                            .radius_all(10.0)
                                                            .padding(6.0, 8.0, 6.0, 8.0)
                                                            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.35), 0.0, 2.0, 8.0);
                                                    }

                                                    exp_card.show(|ui| {
                                                        let header_row = ui.container()
                                                            .id(step_id)
                                                            .full_width()
                                                            .padding(2.0, 4.0, 2.0, 4.0)
                                                            .radius_all(4.0)
                                                            .hover_bg(colors.highlight)
                                                            .row()
                                                            .valign(Align::Center)
                                                            .gap(6.0)
                                                            .show(|ui| {
                                                                if grp.is_active {
                                                                    ui.container()
                                                                        .width(6.0)
                                                                        .height(6.0)
                                                                        .radius_all(3.0)
                                                                        .bg(colors.accent)
                                                                        .shadow(colors.accent.with_alpha(0.8), 0.0, 0.0, 6.0)
                                                                        .show(|_| {});
                                                                }
                                                                ui.text(&summary)
                                                                    .size(11.0)
                                                                    .color(if grp.is_active { Color::rgb(228.0 / 255.0, 228.0 / 255.0, 233.0 / 255.0) } else { Color::rgb(139.0 / 255.0, 141.0 / 255.0, 152.0 / 255.0) })
                                                                    .show();

                                                                ui.text(chevron)
                                                                    .size(7.5)
                                                                    .color(Color::rgb(113.0 / 255.0, 113.0 / 255.0, 122.0 / 255.0))
                                                                    .show();
                                                            });

                                                        if header_row.hovered {
                                                            ui.cursor_icon = CursorIcon::Pointer;
                                                        }
                                                        if header_row.clicked {
                                                            toggle_trajectory_group = Some((msg_idx, step_idx));
                                                        }

                                                        if grp.expanded {
                                                            let line_x = ui.cursor_x + 5.0;
                                                            let line_y = ui.cursor_y;
                                                            ui.container()
                                                                .full_width()
                                                                .padding_left(16.0)
                                                                .column()
                                                                .gap(3.0)
                                                                .show(|ui| {
                                                                for (item_idx, item) in grp.items.iter().enumerate() {
                                                                    match item {
                                                                        ExplorationItem::AnalyzedFile { path, start_line, end_line, symbol, is_peek } => {
                                                                            let path_obj = std::path::Path::new(path);
                                                                            let resolved_path = if path_obj.is_absolute() {
                                                                                path_obj.to_path_buf()
                                                                            } else {
                                                                                state.current_dir.join(path_obj)
                                                                            };
                                                                            let is_dir = resolved_path.is_dir();
                                                                            let file_exists = resolved_path.is_file() || is_dir;

                                                                            let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());
                                                                            let ext = path_obj.extension().and_then(|s| s.to_str()).unwrap_or("");
                                                                            let (icon_glyph, icon_color) = if is_dir {
                                                                                ("\u{f07b}", colors.accent)
                                                                            } else if file_exists {
                                                                                crate::mini_icons::get_mini_file_icon(&fname.to_lowercase(), &ext.to_lowercase())
                                                                            } else {
                                                                                ("\u{f00d}", Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 0.9 })
                                                                            };

                                                                            let (action_verb, verb_color, badge) = if is_dir {
                                                                                ("Explored", colors.text_muted, String::new())
                                                                            } else if !file_exists {
                                                                                (
                                                                                    "Not found",
                                                                                    Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 0.9 },
                                                                                    " [not found]".to_string(),
                                                                                )
                                                                            } else {
                                                                                let verb = match (*is_peek, grp.is_active) {
                                                                                    (true, true) => "Peeking",
                                                                                    (true, false) => "Peeked",
                                                                                    (false, true) => "Reading",
                                                                                    (false, false) => "Read",
                                                                                };
                                                                                let b = if *is_peek {
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
                                                                                (verb, colors.text_muted, b)
                                                                            };

                                                                            // Format clean relative path from workspace root
                                                                            let display_path = if let Ok(rel) = resolved_path.strip_prefix(&state.current_dir) {
                                                                                let s = rel.to_string_lossy().to_string();
                                                                                if s.is_empty() || s == "." {
                                                                                    fname.to_string()
                                                                                } else {
                                                                                    s
                                                                                }
                                                                            } else {
                                                                                fname.to_string()
                                                                            };

                                                                            let item_id = Id::from_u64(999_300_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let file_row = ui.container()
                                                                                .id(item_id)
                                                                                .full_width()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover_bg(if file_exists { colors.highlight } else { colors.highlight.with_alpha(0.3) })
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(action_verb)
                                                                                        .size(10.0)
                                                                                        .color(verb_color)
                                                                                        .show();
                                                                                    ui.text(icon_glyph)
                                                                                        .size(10.0)
                                                                                        .color(icon_color)
                                                                                        .show();
                                                                                    ui.text(&display_path)
                                                                                        .size(10.5)
                                                                                        .color(if file_exists { colors.text_primary } else { colors.text_muted })
                                                                                        .show();
                                                                                    if !badge.is_empty() {
                                                                                        ui.text(&badge)
                                                                                            .size(9.5)
                                                                                            .color(if file_exists { colors.accent } else { Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 0.8 } })
                                                                                            .show();
                                                                                    }
                                                                                });

                                                                            if file_exists && file_row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if file_exists && file_row.clicked {
                                                                                chat_open_file = Some(resolved_path);
                                                                            }
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
                                                                            let resolved = if path_obj.is_absolute() {
                                                                                path_obj.to_path_buf()
                                                                            } else {
                                                                                state.current_dir.join(path_obj)
                                                                            };
                                                                            let display_dir = if let Ok(rel) = resolved.strip_prefix(&state.current_dir) {
                                                                                let s = rel.to_string_lossy().to_string();
                                                                                if s.is_empty() || s == "." {
                                                                                    state.current_dir.file_name().and_then(|s| s.to_str()).unwrap_or(".").to_string()
                                                                                } else {
                                                                                    s
                                                                                }
                                                                            } else {
                                                                                path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str()).to_string()
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
                                                                                    ui.text(&display_dir)
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
                                                        let line_h = (ui.cursor_y - line_y).max(4.0);
                                                        let line_col = if grp.is_active {
                                                            [46.0 / 255.0, 46.0 / 255.0, 56.0 / 255.0, 1.0] // #2e2e38
                                                        } else {
                                                            [42.0 / 255.0, 42.0 / 255.0, 48.0 / 255.0, 1.0] // #2a2a30
                                                        };
                                                        ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
                                                            instance: zenthra_render::RectInstance {
                                                                pos: [line_x, line_y + 1.0],
                                                                size: [1.0, (line_h - 2.0).max(2.0)],
                                                                color: line_col,
                                                                radius: [0.5; 4],
                                                                clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
                                                                ..Default::default()
                                                            },
                                                        }));
                                                    }
                                                });
                                            }
                                                TrajectoryStep::FileEdit { path, lines_added, lines_removed } => {
                                                    let path_obj = std::path::Path::new(path);
                                                    let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());
                                                    let ext = path_obj.extension().and_then(|s| s.to_str()).unwrap_or("");
                                                    let (icon_glyph, icon_color) = crate::mini_icons::get_mini_file_icon(&fname.to_lowercase(), &ext.to_lowercase());

                                                    let mut added = *lines_added;
                                                    let mut removed = *lines_removed;
                                                    let pb = std::path::PathBuf::from(path);
                                                    let full_p = if pb.is_absolute() { pb } else { state.current_dir.join(&pb) };
                                                    if added == 0 && removed == 0 {
                                                        if let Some(rev) = state.pending_reviews.iter().find(|r| path_matches(&r.path, &full_p)) {
                                                            added = rev.diff_lines.iter().filter(|(_, s)| *s == crate::state::DiffLineState::Added).count();
                                                            removed = rev.diff_lines.iter().filter(|(_, s)| *s == crate::state::DiffLineState::Removed).count();
                                                            if added == 0 && (rev.is_new_file || removed == 0) {
                                                                added = rev.proposed_content.lines().count().max(1);
                                                            }
                                                        }
                                                        if added == 0 {
                                                            if let Ok(content) = std::fs::read_to_string(&full_p) {
                                                                let count = content.lines().count();
                                                                if count > 0 {
                                                                    added = count;
                                                                }
                                                            }
                                                        }
                                                    }

                                                    let edit_row = ui.container()
                                                        .id(step_id)
                                                        .full_width()
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .radius_all(3.0)
                                                        .clickable(true)
                                                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
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
                                                            if added > 0 {
                                                                ui.text(&format!("+{}", added))
                                                                    .size(9.5)
                                                                    .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                    .show();
                                                            }
                                                            if removed > 0 {
                                                                ui.text(&format!("-{}", removed))
                                                                    .size(9.5)
                                                                    .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                                    .show();
                                                            }
                                                        });
                                                    if edit_row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                                    if edit_row.clicked {
                                                        chat_open_file = Some(full_p);
                                                    }
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
                                                TrajectoryStep::Plan { title: _, steps, expanded: _ } => {
                                                    let plan_badge_id = Id::from_u64(999_400_000 + (msg_idx as u64) * 100 + (step_idx as u64));
                                                    let proceed_btn_id = Id::from_u64(999_400_000 + (msg_idx as u64) * 100 + (step_idx as u64) + 1);

                                                    ui.container()
                                                        .full_width()
                                                        .padding(4.0, 2.0, 4.0, 2.0)
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(8.0)
                                                        .show(|ui| {
                                                            // Implementation Badge / Button
                                                            let badge_resp = ui.container()
                                                                .id(plan_badge_id)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(6.0)
                                                                .padding(4.0, 10.0, 4.0, 10.0)
                                                                .radius_all(6.0)
                                                                .bg(colors.accent.with_alpha(0.12))
                                                                .border(colors.accent.with_alpha(0.35), 1.0)
                                                                .hover_bg(colors.accent.with_alpha(0.24))
                                                                .show(|ui| {
                                                                    ui.text("\u{f15c}")
                                                                        .size(10.0)
                                                                        .color(colors.accent)
                                                                        .show();
                                                                    ui.text("Implementation Plan")
                                                                        .size(10.5)
                                                                        .weight(FontWeight::SemiBold)
                                                                        .color(colors.accent)
                                                                        .show();
                                                                    let step_count = steps.len();
                                                                    if step_count > 0 {
                                                                        ui.container()
                                                                            .padding(1.0, 5.0, 1.0, 5.0)
                                                                            .radius_all(10.0)
                                                                            .bg(colors.accent.with_alpha(0.18))
                                                                            .show(|ui| {
                                                                                ui.text(&format!("{} steps", step_count))
                                                                                    .size(9.0)
                                                                                    .weight(FontWeight::Medium)
                                                                                    .color(colors.text_primary)
                                                                                    .show();
                                                                            });
                                                                    }
                                                                    ui.text("\u{f06e}")
                                                                        .size(9.5)
                                                                        .color(colors.accent.with_alpha(0.85))
                                                                        .show();
                                                                });

                                                            if badge_resp.hovered {
                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                            }
                                                            if badge_resp.clicked {
                                                                open_implementation_plan = true;
                                                            }

                                                            // Proceed with Plan Button
                                                            let proceed_resp = ui.container()
                                                                .id(proceed_btn_id)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(5.0)
                                                                .padding(4.0, 10.0, 4.0, 10.0)
                                                                .radius_all(6.0)
                                                                .bg(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0).with_alpha(0.16))
                                                                .border(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0).with_alpha(0.38), 1.0)
                                                                .hover_bg(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0).with_alpha(0.26))
                                                                .show(|ui| {
                                                                    ui.text("\u{f04b}")
                                                                        .size(9.0)
                                                                        .color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                                                                        .show();
                                                                    ui.text("Proceed")
                                                                        .size(10.5)
                                                                        .weight(FontWeight::SemiBold)
                                                                        .color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                                                                        .show();
                                                                });

                                                            if proceed_resp.hovered {
                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                            }
                                                            if proceed_resp.clicked {
                                                                proceed_with_plan = true;
                                                            }
                                                        });
                                                }
                                            }
                                        }

                                        // ── Sibling 1.5: Collapsible Thought / Reasoning Block ──
                                        if state.ai_settings.show_thoughts {
                                            if let Some(ref thoughts) = msg.thoughts {
                                                if !thoughts.trim().is_empty() {
                                                    let thought_id = Id::from_u64(999_310_000 + (msg_idx as u64) * 10);
                                                    let is_expanded = msg.thoughts_expanded;
                                                    let chevron = if is_expanded { "\u{f078}" } else { "\u{f054}" };

                                                    ui.container()
                                                        .full_width()
                                                        .column()
                                                        .gap(4.0)
                                                        .bg(Color::rgb(20.0 / 255.0, 20.0 / 255.0, 24.0 / 255.0))
                                                        .border(Color::rgb(38.0 / 255.0, 38.0 / 255.0, 44.0 / 255.0), 1.0)
                                                        .radius_all(8.0)
                                                        .padding(6.0, 10.0, 6.0, 10.0)
                                                        .show(|ui| {
                                                            let header_res = ui.container()
                                                                .id(thought_id)
                                                                .full_width()
                                                                .clickable(true)
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(6.0)
                                                                .padding(2.0, 0.0, 2.0, 0.0)
                                                                .show(|ui| {
                                                                    ui.text(chevron)
                                                                        .size(9.0)
                                                                        .color(colors.text_muted)
                                                                        .show();
                                                                    ui.text("\u{f0eb}") // fa-lightbulb
                                                                        .size(10.0)
                                                                        .color(colors.accent)
                                                                        .show();
                                                                    let title = if msg.is_working && full_text.is_empty() {
                                                                        "Thinking..."
                                                                    } else {
                                                                        "Thought Process"
                                                                    };
                                                                    ui.text(title)
                                                                        .size(10.5)
                                                                        .weight(FontWeight::Medium)
                                                                        .color(colors.text_primary)
                                                                        .show();
                                                                });

                                                            if header_res.hovered {
                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                            }
                                                            if header_res.clicked {
                                                                toggle_thoughts_idx = Some(msg_idx);
                                                            }

                                                            if is_expanded {
                                                                ui.container()
                                                                    .full_width()
                                                                    .padding(4.0, 0.0, 4.0, 16.0)
                                                                    .show(|ui| {
                                                                        ui.text(thoughts.trim())
                                                                            .size(10.0)
                                                                            .line_height(1.4)
                                                                            .color(colors.text_muted)
                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                            .show();
                                                                    });
                                                            }
                                                        });
                                                }
                                            }
                                        }

                                        let has_visible_thoughts = msg.thoughts.as_ref().map_or(false, |t| !t.trim().is_empty());

                                        // "Working..." indicator — shown only when no tool steps or thoughts have started yet
                                        if msg.is_working && full_text.is_empty() && msg.trajectory.is_empty() && !has_visible_thoughts {
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
                                        } else if !msg.is_working && full_text.is_empty() && msg.trajectory.is_empty() && !has_visible_thoughts {
                                            ui.text("No response generated. Please check your model or API quota.")
                                                .size(10.5)
                                                .italic()
                                                .color(colors.text_muted)
                                                .show();
                                        }

                                        // ── Sibling 2: Final Summary / Markdown Answer or Error Card ──────
                                        if msg.is_error() {
                                            let (err_title, err_desc, raw_details) = msg.parse_error_details();
                                            let err_details_id = Id::from_u64(999_350_000 + (msg_idx as u64) * 10);
                                            let err_copy_id = Id::from_u64(999_350_000 + (msg_idx as u64) * 10 + 1);
                                            let is_details_expanded = msg.expanded;

                                            ui.container()
                                                .full_width()
                                                .radius_all(8.0)
                                                .bg(Color::rgba(0.95, 0.25, 0.25, 0.08))
                                                .border(Color::rgba(0.95, 0.30, 0.30, 0.28), 1.0)
                                                .padding(10.0, 12.0, 10.0, 12.0)
                                                .column()
                                                .gap(8.0)
                                                .show(|ui| {
                                                    // Header row: Icon + Title + Error Badge
                                                    ui.container()
                                                        .full_width()
                                                        .row()
                                                        .halign(Align::SpaceBetween)
                                                        .valign(Align::Center)
                                                        .show(|ui| {
                                                            ui.container()
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(6.0)
                                                                .show(|ui| {
                                                                    ui.text("\u{f071}")
                                                                        .size(11.5)
                                                                        .color(Color::rgba(0.98, 0.40, 0.40, 1.0))
                                                                        .show();
                                                                    ui.text(&err_title)
                                                                        .size(11.5)
                                                                        .weight(FontWeight::Bold)
                                                                        .color(Color::rgba(0.98, 0.45, 0.45, 1.0))
                                                                        .show();
                                                                });

                                                            ui.container()
                                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                                .radius_all(4.0)
                                                                .bg(Color::rgba(0.95, 0.25, 0.25, 0.18))
                                                                .show(|ui| {
                                                                    ui.text("Error")
                                                                        .size(9.0)
                                                                        .weight(FontWeight::Medium)
                                                                        .color(Color::rgba(0.98, 0.55, 0.55, 0.95))
                                                                        .show();
                                                                });
                                                        });

                                                    // Friendly Description
                                                    ui.text(&err_desc)
                                                        .size(11.0)
                                                        .line_height(1.6)
                                                        .color(colors.text_primary.with_alpha(0.9))
                                                        .max_width((ai_text_w - 24.0).max(60.0))
                                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                                        .show();

                                                    // Collapsible Technical Details (if raw json or full details exist)
                                                    if let Some(ref raw) = raw_details {
                                                        let toggle_btn = ui.container()
                                                            .id(err_details_id)
                                                            .padding(3.0, 6.0, 3.0, 6.0)
                                                            .radius_all(4.0)
                                                            .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.05))
                                                            .row()
                                                            .valign(Align::Center)
                                                            .gap(4.0)
                                                            .show(|ui| {
                                                                let chevron = if is_details_expanded { "\u{f078}" } else { "\u{f054}" };
                                                                let label = if is_details_expanded {
                                                                    "Hide technical details"
                                                                } else {
                                                                    "Show technical details"
                                                                };
                                                                ui.text(chevron)
                                                                    .size(7.5)
                                                                    .color(colors.text_muted)
                                                                    .show();
                                                                ui.text(label)
                                                                    .size(9.5)
                                                                    .color(colors.text_muted)
                                                                    .show();
                                                            });

                                                        if toggle_btn.hovered {
                                                            ui.cursor_icon = CursorIcon::Pointer;
                                                        }
                                                        if toggle_btn.clicked {
                                                            toggle_expand_idx = Some(msg_idx);
                                                        }

                                                        if is_details_expanded {
                                                            let raw_w = (ai_text_w - 24.0).max(50.0);
                                                            ui.container()
                                                                .width(raw_w)
                                                                .padding(8.0, 10.0, 8.0, 10.0)
                                                                .radius_all(6.0)
                                                                .bg(colors.bg_base.with_alpha(0.85))
                                                                .border(Color::rgba(0.95, 0.30, 0.30, 0.2), 0.5)
                                                                .column()
                                                                .gap(4.0)
                                                                .show(|ui| {
                                                                    ui.text(raw)
                                                                        .size(9.5)
                                                                        .monospace()
                                                                        .line_height(1.5)
                                                                        .color(colors.text_muted)
                                                                        .max_width((raw_w - 20.0).max(40.0))
                                                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                        .show();
                                                                });
                                                        }
                                                    }

                                                    // Footer: Copy Button
                                                    ui.container()
                                                        .full_width()
                                                        .row()
                                                        .halign(Align::Right)
                                                        .valign(Align::Center)
                                                        .padding_top(4.0)
                                                        .show(|ui| {
                                                            let copy_btn = ui.container()
                                                                .id(err_copy_id)
                                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                                .radius_all(4.0)
                                                                .hover_bg(Color::rgba(1.0, 1.0, 1.0, 0.08))
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(4.0)
                                                                .show(|ui| {
                                                                    ui.text(if is_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                        .size(9.0)
                                                                        .color(if is_copied { colors.accent } else { colors.text_muted })
                                                                        .show();
                                                                    ui.text(if is_copied { "Copied" } else { "Copy Error" })
                                                                        .size(9.0)
                                                                        .color(if is_copied { colors.accent } else { colors.text_muted })
                                                                        .show();
                                                                });

                                                            if copy_btn.hovered {
                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                            }
                                                            if copy_btn.clicked {
                                                                copy_clicked = true;
                                                            }
                                                        });
                                                });
                                        } else if !full_text.is_empty() {
                                            // ── Cache: parse once, reuse every frame ──────────────
                                            let content_hash = {
                                                use std::hash::{Hash, Hasher};
                                                let mut h = std::collections::hash_map::DefaultHasher::new();
                                                full_text.hash(&mut h);
                                                h.finish()
                                            };
                                            if !state.ai_block_cache.contains_key(&content_hash) {
                                                let parsed = parse_markdown_blocks_impl(full_text);
                                                state.ai_block_cache.insert(content_hash, Box::new(parsed));
                                            }
                                            let blocks = state.ai_block_cache
                                                .get(&content_hash)
                                                .and_then(|b| b.downcast_ref::<Vec<MarkdownBlock>>())
                                                .map(|v| v.as_slice())
                                                .unwrap_or(&[]);
                                            for (block_idx, block) in blocks.iter().enumerate() {
                                                match block {
                                                    MarkdownBlock::Heading { level, spans } => {
                                                        let (font_size, pad_t, pad_b) = match level {
                                                            1 => (14.0_f32, 8.0_f32, 4.0_f32),
                                                            2 => (12.5, 6.0, 3.0),
                                                            3 => (11.5, 5.0, 2.0),
                                                            _ => (11.0, 3.0, 1.0),
                                                        };
                                                        ui.container()
                                                            .padding_top(pad_t)
                                                            .padding_bottom(pad_b)
                                                            .show(|ui| {
                                                                draw_inline_spans(ui, spans, ai_text_w, font_size, FontWeight::Bold, colors.text_primary, colors);
                                                            });
                                                    }
                                                    MarkdownBlock::Rule => {
                                                        ui.container()
                                                            .full_width()
                                                            .height(1.0)
                                                            .bg(colors.border.with_alpha(0.4))
                                                            .padding_top(6.0)
                                                            .padding_bottom(6.0)
                                                            .show(|_| {});
                                                    }
                                                    MarkdownBlock::BlockQuote { kind, spans } => {
                                                        let (accent_col, badge_label, badge_icon) = match kind {
                                                            QuoteKind::Note      => (Color::rgb(59.0/255.0, 130.0/255.0, 246.0/255.0), "NOTE", "\u{f05a}"),
                                                            QuoteKind::Tip       => (Color::rgb(34.0/255.0, 197.0/255.0, 94.0/255.0),  "TIP",  "\u{f0eb}"),
                                                            QuoteKind::Important => (Color::rgb(139.0/255.0, 92.0/255.0, 246.0/255.0), "IMPORTANT", "\u{f06a}"),
                                                            QuoteKind::Warning   => (Color::rgb(245.0/255.0, 158.0/255.0, 11.0/255.0), "WARNING", "\u{f071}"),
                                                            QuoteKind::Caution   => (Color::rgb(239.0/255.0, 68.0/255.0, 68.0/255.0),  "CAUTION", "\u{f057}"),
                                                            QuoteKind::Plain     => (colors.accent, "", "\u{f10d}"),
                                                        };
                                                        let is_keyed = !badge_label.is_empty();
                                                        // Blockquote: row with left accent bar + content column
                                                        ui.container()
                                                            .max_width(ai_text_w)
                                                            .row()
                                                            .gap(0.0)
                                                            .show(|ui| {
                                                                // Left accent bar
                                                                ui.container()
                                                                    .width(3.0)
                                                                    .bg(accent_col)
                                                                    .radius(0.0, 0.0, 2.0, 2.0)
                                                                    .show(|_| {});
                                                                // Content
                                                                ui.container()
                                                                    .bg(accent_col.with_alpha(0.08))
                                                                    .padding(6.0, 10.0, 6.0, 10.0)
                                                                    .column()
                                                                    .gap(4.0)
                                                                    .show(|ui| {
                                                                        if is_keyed {
                                                                            ui.container()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    ui.text(badge_icon)
                                                                                        .size(9.0)
                                                                                        .color(accent_col)
                                                                                        .show();
                                                                                    ui.text(badge_label)
                                                                                        .size(9.0)
                                                                                        .weight(FontWeight::Bold)
                                                                                        .color(accent_col)
                                                                                        .show();
                                                                                });
                                                                        }
                                                                        draw_inline_spans(ui, spans, (ai_text_w - 20.0).max(40.0), 11.0, FontWeight::Regular, colors.text_muted, colors);
                                                                    });
                                                            });
                                                    }
                                                    MarkdownBlock::Table { alignments: _aligns, headers, rows } => {
                                                        let col_count = headers.len().max(1);
                                                        let col_w = ((ai_text_w - 2.0) / col_count as f32).max(40.0);
                                                        ui.container()
                                                            .max_width(ai_text_w)
                                                            .radius_all(6.0)
                                                            .border(colors.border.with_alpha(0.5), 0.5)
                                                            .clip(true)
                                                            .column()
                                                            .show(|ui| {
                                                                // Header row
                                                                ui.container()
                                                                    .full_width()
                                                                    .row()
                                                                    .gap(0.0)
                                                                    .bg(colors.bg_sidebar.with_alpha(0.9))
                                                                    .show(|ui| {
                                                                        for (ci, cell) in headers.iter().enumerate() {
                                                                            let is_last = ci + 1 == col_count;
                                                                            ui.container()
                                                                                .width(col_w)
                                                                                .padding(5.0, 8.0, 5.0, 8.0)
                                                                                .show(|ui| {
                                                                                    draw_inline_spans(ui, cell, col_w - 16.0, 10.0, FontWeight::SemiBold, colors.text_primary, colors);
                                                                                });
                                                                            if !is_last {
                                                                                ui.container()
                                                                                    .width(0.5)
                                                                                    .bg(colors.border.with_alpha(0.4))
                                                                                    .show(|_| {});
                                                                            }
                                                                        }
                                                                    });
                                                                // Divider under header
                                                                ui.container().full_width().height(0.5).bg(colors.border.with_alpha(0.5)).show(|_| {});
                                                                // Data rows
                                                                for (ri, row) in rows.iter().enumerate() {
                                                                    let row_bg = if ri % 2 == 1 {
                                                                        colors.bg_sidebar.with_alpha(0.3)
                                                                    } else {
                                                                        Color::TRANSPARENT
                                                                    };
                                                                    ui.container()
                                                                        .full_width()
                                                                        .row()
                                                                        .gap(0.0)
                                                                        .bg(row_bg)
                                                                        .show(|ui| {
                                                                            for (ci, cell) in row.iter().enumerate() {
                                                                                let is_last = ci + 1 == col_count;
                                                                                ui.container()
                                                                                    .width(col_w)
                                                                                    .padding(4.0, 8.0, 4.0, 8.0)
                                                                                    .show(|ui| {
                                                                                        draw_inline_spans(ui, cell, col_w - 16.0, 10.5, FontWeight::Regular, colors.text_muted, colors);
                                                                                    });
                                                                                if !is_last {
                                                                                    ui.container()
                                                                                        .width(0.5)
                                                                                        .bg(colors.border.with_alpha(0.3))
                                                                                        .show(|_| {});
                                                                                }
                                                                            }
                                                                        });
                                                                    if ri + 1 < rows.len() {
                                                                        ui.container().full_width().height(0.5).bg(colors.border.with_alpha(0.2)).show(|_| {});
                                                                    }
                                                                }
                                                            });
                                                    }

                                                    MarkdownBlock::BulletItem { depth, spans, checked } => {
                                                        let indent = (*depth as f32) * 12.0 + 2.0;
                                                        let avail_w = (ai_text_w - indent - 16.0).max(40.0);
                                                        ui.container()
                                                            .full_width()
                                                            .row()
                                                            .valign(Align::Top)
                                                            .gap(6.0)
                                                            .padding_left(indent)
                                                            .show(|ui| {
                                                                // Bullet or checkbox
                                                                match checked {
                                                                    Some(true) => {
                                                                        ui.text("\u{f14a}")
                                                                            .size(10.5)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                    }
                                                                    Some(false) => {
                                                                        ui.text("\u{f0c8}")
                                                                            .size(10.5)
                                                                            .color(colors.text_muted)
                                                                            .show();
                                                                    }
                                                                    None => {
                                                                        let bullet = if *depth == 0 { "•" } else { "◦" };
                                                                        ui.text(bullet)
                                                                            .size(9.0)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                    }
                                                                }
                                                                draw_inline_spans(ui, spans, avail_w, 11.0, FontWeight::Regular, colors.text_primary, colors);
                                                            });
                                                    }
                                                    MarkdownBlock::NumberedItem { num, depth, spans } => {
                                                        let indent = (*depth as f32) * 12.0 + 2.0;
                                                        let avail_w = (ai_text_w - indent - 22.0).max(40.0);
                                                        ui.container()
                                                            .full_width()
                                                            .row()
                                                            .valign(Align::Top)
                                                            .gap(6.0)
                                                            .padding_left(indent)
                                                            .show(|ui| {
                                                                ui.text(&format!("{}." , num))
                                                                    .size(10.5)
                                                                    .weight(FontWeight::Medium)
                                                                    .color(colors.accent)
                                                                    .show();
                                                                draw_inline_spans(ui, spans, avail_w, 11.0, FontWeight::Regular, colors.text_primary, colors);
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
                                                                            .max_width((code_box_w - 18.0).max(40.0))
                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                            .show();
                                                                    });
                                                            });
                                                    }
                                                    MarkdownBlock::Paragraph(spans) => {
                                                        draw_inline_spans(ui, spans, ai_text_w, 11.0, FontWeight::Regular, colors.text_primary, colors);
                                                    }

                                                }
                                            }

                                            // ── Footer: copy button pinned right ──────────────
                                            ui.container()
                                                .full_width()
                                                .row()
                                                .halign(Align::Right)
                                                .valign(Align::Center)
                                                .padding_top(4.0)
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
                        if let Some(idx) = toggle_thoughts_idx {
                            if let Some(m) = state.ai_messages.get_mut(idx) {
                                m.thoughts_expanded = !m.thoughts_expanded;
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

                        if let Some(path) = chat_open_file {
                            state.open_file(&path);
                            ui.needs_redraw = true;
                        }

                        if open_implementation_plan {
                            let plan_path = state.current_dir.join("implementation_plan.md");
                            state.open_file(&plan_path);
                            for tab in &mut state.tabs {
                                if let Some(ref p) = tab.path {
                                    if path_matches(p, &plan_path) || p.file_name().and_then(|n| n.to_str()) == Some("implementation_plan.md") {
                                        tab.is_markdown_preview = true;
                                    }
                                }
                            }
                            ui.needs_redraw = true;
                        }

                        if proceed_with_plan {
                            state.send_ai_message("Proceed with the implementation plan.");
                            ui.needs_redraw = true;
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

                // Input box container (docked at the bottom, centered with max width cap and floating bottom gap)
                ui.container()
                    .width(chat_w)
                    .height(dock_total_h)
                    .max_height(dock_total_h)
                    .padding(DOCK_TOP_PAD, 10.0, BOTTOM_DOCK_GAP, 10.0)
                    .column()
                    .halign(Align::Center)
                    .show(|ui| {
                    ui.container()
                        .width(input_dock_w)
                        .column()
                        .gap(6.0)
                        .show(|ui| {
                    let is_light = colors.bg_base.r > 0.5;
                    let input_box_bg = if state.glassmorphism {
                        colors.bg_base.with_alpha((state.glass_opacity * 0.40).clamp(0.12, 0.55))
                    } else if is_light {
                        Color::rgb(248.0 / 255.0, 248.0 / 255.0, 250.0 / 255.0)
                    } else {
                        Color::rgb(22.0 / 255.0, 22.0 / 255.0, 25.0 / 255.0) // #161619
                    };
                    let input_border_col = if state.glassmorphism {
                        colors.border.with_alpha(0.35)
                    } else if is_light {
                        colors.border
                    } else {
                        Color::rgb(39.0 / 255.0, 39.0 / 255.0, 46.0 / 255.0) // #27272e
                    };
                    let input_shadow_col = if state.glassmorphism {
                        Color::rgba(0.0, 0.0, 0.0, 0.12)
                    } else if is_light {
                        Color::rgba(0.0, 0.0, 0.0, 0.08)
                    } else {
                        Color::rgba(0.0, 0.0, 0.0, 0.60) // shadow-2xl
                    };
                    let pill_bg = if state.glassmorphism {
                        Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.06)
                    } else if is_light {
                        Color::rgba(0.0, 0.0, 0.0, 0.05)
                    } else {
                        Color::rgb(32.0 / 255.0, 32.0 / 255.0, 37.0 / 255.0) // #202025
                    };
                    let pill_hover_bg = if state.glassmorphism {
                        Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.14)
                    } else if is_light {
                        colors.highlight
                    } else {
                        Color::rgb(39.0 / 255.0, 39.0 / 255.0, 47.0 / 255.0) // #27272f
                    };
                    let pill_border = if state.glassmorphism {
                        colors.border.with_alpha(0.25)
                    } else if is_light {
                        colors.border
                    } else {
                        Color::rgb(45.0 / 255.0, 45.0 / 255.0, 54.0 / 255.0) // #2d2d36
                    };

                    let mut target_open_file: Option<std::path::PathBuf> = None;
                    let mut target_reject_file: Option<std::path::PathBuf> = None;
                    let mut action_accept_all = false;
                    let mut action_reject_all = false;

                    // ─── Modified Files View & Quick Toolbar ────────────────────────
                    if state.show_modified_files_popup {
                        if has_modified_files {
                            // 1. Scrollable file list container (only for the file list) with input background & drop shadow
                            let file_list_h = (modified_files.len().max(1) as f32 * 22.0 + 8.0).min(140.0);
                            let mut mod_files_builder = ui.container()
                                .full_width()
                                .height(file_list_h)
                                .bg(input_box_bg)
                                .radius_all(7.0)
                                .padding(4.0, 6.0, 4.0, 6.0)
                                .scroll_y(true)
                                .column()
                                .gap(2.0);

                            if state.ai_settings.effective_input_shadow() {
                                mod_files_builder = mod_files_builder.shadow(input_shadow_col, 0.0, 2.0, 12.0);
                            }
                            if state.ai_settings.effective_input_outline() {
                                mod_files_builder = mod_files_builder.border(colors.border.with_alpha(0.6), 0.75);
                            }
                            if state.glassmorphism {
                                mod_files_builder = mod_files_builder.backdrop_blur(state.blur_strength.max(4.0));
                            }

                            mod_files_builder.show(|ui| {
                                    for (idx, item) in modified_files.iter().enumerate() {
                                        let ext = item.file_path.extension().and_then(|s| s.to_str()).unwrap_or("");
                                        let (_glyph, icon_color) = crate::mini_icons::get_mini_file_icon(&item.file_name.to_lowercase(), &ext.to_lowercase());

                                        let _is_active = state.tabs.get(state.active_tab.unwrap_or(0))
                                            .and_then(|t| t.path.as_ref())
                                            .map(|p| path_matches(p, &item.file_path))
                                            .unwrap_or(false);

                                        let mut reject_this_file = false;
                                        let row_id = ui.id_from(&format!("mod_file_row_{}_{}", idx, item.file_name));
                                        let row = ui.container()
                                            .id(row_id)
                                            .full_width()
                                            .height(20.0)
                                            .radius_all(3.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                            .padding(0.0, 4.0, 0.0, 4.0)
                                            .clickable(true)
                                            .row()
                                            .valign(Align::Center)
                                            .halign(Align::SpaceBetween)
                                            .show(|ui| {
                                                // Left: File icon + File name + Relative path (if different)
                                                ui.container()
                                                    .row()
                                                    .valign(Align::Center)
                                                    .gap(6.0)
                                                    .show(|ui| {
                                                        ui.container()
                                                            .width(12.0)
                                                            .height(12.0)
                                                            .radius_all(2.5)
                                                            .border(icon_color, 1.0)
                                                            .halign(Align::Center)
                                                            .valign(Align::Center)
                                                            .show(|ui| {
                                                                ui.container()
                                                                    .width(3.0)
                                                                    .height(3.0)
                                                                    .radius_all(1.5)
                                                                    .bg(icon_color)
                                                                    .show(|_| {});
                                                            });

                                                        ui.text(&item.file_name)
                                                            .size(11.0)
                                                            .weight(FontWeight::Medium)
                                                            .color(colors.text_primary)
                                                            .show();

                                                        if item.rel_path != item.file_name && !item.rel_path.is_empty() {
                                                            let display_rel = if item.rel_path.len() > 32 {
                                                                format!("...{}", &item.rel_path[item.rel_path.len() - 29..])
                                                            } else {
                                                                item.rel_path.clone()
                                                            };
                                                            ui.text(&display_rel)
                                                                .size(10.0)
                                                                .color(colors.text_dim)
                                                                .show();
                                                        }
                                                    });

                                                // Right: +added, -removed, cross icon
                                                ui.container()
                                                    .row()
                                                    .valign(Align::Center)
                                                    .gap(5.0)
                                                    .show(|ui| {
                                                        if item.lines_added > 0 {
                                                            ui.text(&format!("+{}", item.lines_added))
                                                                .size(10.0)
                                                                .weight(FontWeight::Medium)
                                                                .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                .show();
                                                        }
                                                        if item.lines_removed > 0 {
                                                            ui.text(&format!("-{}", item.lines_removed))
                                                                .size(10.0)
                                                                .weight(FontWeight::Medium)
                                                                .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                                .show();
                                                        }

                                                        // Cross icon to reject this individual file
                                                        let cross_id = ui.id_from(&format!("mod_file_cross_{}_{}", idx, item.file_name));
                                                        let cross_btn = ui.container()
                                                            .id(cross_id)
                                                            .clickable(true)
                                                            .padding(1.0, 3.0, 1.0, 3.0)
                                                            .radius_all(2.5)
                                                            .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.08))
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

                            // 2. Toolbar row (outside container): [ ‹ 📄 N Files] on left, Reject all / Accept all / ▾ on right
                            ui.container()
                                .full_width()
                                .height(20.0)
                                .padding(0.0, 2.0, 0.0, 2.0)
                                .row()
                                .halign(Align::SpaceBetween)
                                .valign(Align::Center)
                                .show(|ui| {
                                    // Left: [ ‹ 📄 N Files] toggle button (left back icon to close)
                                    let toggle_btn = ui.container()
                                        .clickable(true)
                                        .padding(2.0, 5.0, 2.0, 5.0)
                                        .radius_all(3.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                        .row()
                                        .valign(Align::Center)
                                        .gap(5.0)
                                        .show(|ui| {
                                            ui.text("‹")
                                                .size(13.0)
                                                .weight(FontWeight::Bold)
                                                .color(colors.text_muted)
                                                .show();
                                            ui.text("\u{f15c}")
                                                .size(11.5)
                                                .color(colors.accent)
                                                .show();
                                            let count = modified_files.len();
                                            let label = format!("{} {}", count, if count == 1 { "File" } else { "Files" });
                                            ui.text(&label)
                                                .size(10.5)
                                                .weight(FontWeight::Medium)
                                                .color(colors.accent)
                                                .show();
                                        });
                                    if toggle_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if toggle_btn.clicked {
                                        state.show_modified_files_popup = false;
                                        ui.needs_redraw = true;
                                    }

                                    // Right: Reject all, Accept all, and down arrow ▾ to collapse
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(6.0)
                                        .show(|ui| {
                                            let rej_btn = ui.container()
                                                .id(Id::from_u64(999999866))
                                                .clickable(true)
                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                                .show(|ui| {
                                                    ui.text("Reject all")
                                                        .size(11.0)
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
                                                .padding(2.0, 8.0, 2.0, 8.0)
                                                .radius_all(4.0)
                                                .bg(colors.accent)
                                                .hover_bg(colors.accent.with_alpha(0.85))
                                                .show(|ui| {
                                                    ui.text("Accept all")
                                                        .size(11.0)
                                                        .weight(FontWeight::SemiBold)
                                                        .color(Color::WHITE)
                                                        .show();
                                                });
                                            if acc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if acc_btn.clicked {
                                                action_accept_all = true;
                                            }

                                            // Down arrow to collapse
                                            let arrow_btn = ui.container()
                                                .id(Id::from_u64(999999868))
                                                .clickable(true)
                                                .padding(2.0, 5.0, 2.0, 5.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                                .show(|ui| {
                                                    ui.text("\u{f078}")
                                                        .size(7.5)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });
                                            if arrow_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if arrow_btn.clicked {
                                                state.show_modified_files_popup = false;
                                                ui.needs_redraw = true;
                                            }
                                        });
                                });
                        } else {
                            // Empty state: show inline on toolbar row (no box, other icons hidden)
                            ui.container()
                                .full_width()
                                .height(20.0)
                                .padding(0.0, 2.0, 0.0, 2.0)
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    let toggle_btn = ui.container()
                                        .clickable(true)
                                        .padding(2.0, 4.0, 2.0, 4.0)
                                        .radius_all(3.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                        .show(|ui| {
                                            ui.text("\u{f15c}")
                                                .size(11.5)
                                                .color(colors.accent)
                                                .show();
                                        });
                                    if toggle_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if toggle_btn.clicked {
                                        state.show_modified_files_popup = false;
                                        ui.needs_redraw = true;
                                    }

                                    ui.text("No files modified yet")
                                        .size(10.5)
                                        .color(colors.text_muted.with_alpha(0.7))
                                        .show();
                                });
                        }
                    } else {
                        // Standard quick toolbar when closed (all icons visible)
                        ui.container()
                            .full_width()
                            .height(20.0)
                            .padding(0.0, 2.0, 0.0, 2.0)
                            .row()
                            .halign(Align::SpaceBetween)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(8.0)
                                    .show(|ui| {
                                        // 1. File icon (with count if > 0)
                                        let file_id = Id::from_u64(999999860);
                                        let count = modified_files.len();
                                        let file_btn = ui.container()
                                            .id(file_id)
                                            .clickable(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                            .row()
                                            .valign(Align::Center)
                                            .gap(3.0)
                                            .show(|ui| {
                                                ui.text("\u{f0219}")
                                                    .size(12.5)
                                                    .color(if count > 0 { colors.accent } else { colors.text_muted })
                                                    .show();
                                                if count > 0 {
                                                    ui.text(&format!("{}", count))
                                                        .size(9.5)
                                                        .weight(FontWeight::Medium)
                                                        .color(colors.accent)
                                                        .show();
                                                }
                                            });
                                        if file_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if file_btn.clicked {
                                            state.show_modified_files_popup = true;
                                            ui.needs_redraw = true;
                                        }

                                        // 2. Terminal icon
                                        let term_id = Id::from_u64(999999861);
                                        let term_bg = if state.terminal_visible {
                                            colors.accent.with_alpha(0.18)
                                        } else {
                                            Color::TRANSPARENT
                                        };
                                        let term_btn = ui.container()
                                            .id(term_id)
                                            .clickable(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .bg(term_bg)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("\u{f489}")
                                                    .size(12.0)
                                                    .color(if state.terminal_visible { colors.accent } else { colors.text_muted })
                                                    .show();
                                            });
                                        if term_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if term_btn.clicked {
                                            state.terminal_visible = !state.terminal_visible;
                                            if state.terminal_visible {
                                                state.ensure_terminal_session();
                                            }
                                            state.save_persistent_state();
                                            ui.needs_redraw = true;
                                        }

                                        // 3. Inbox icon
                                        let inbox_id = Id::from_u64(999999862);
                                        let inbox_btn = ui.container()
                                            .id(inbox_id)
                                            .clickable(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .row()
                                            .valign(Align::Center)
                                            .gap(1.5)
                                            .show(|ui| {
                                                ui.text("\u{f48d}")
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
                                        if inbox_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }

                                        // 4. Web icon
                                        let chrome_id = Id::from_u64(999999863);
                                        let chrome_btn = ui.container()
                                            .id(chrome_id)
                                            .clickable(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .hover_bg(colors.highlight)
                                            .show(|ui| {
                                                ui.text("\u{f059f}")
                                                    .size(12.5)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });
                                        if chrome_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    });

                                // Right side when closed but has modified files: Reject all, Accept all, and Top Arrow ▴!
                                if has_modified_files {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(6.0)
                                        .show(|ui| {
                                            let rej_btn = ui.container()
                                                .id(Id::from_u64(999999866))
                                                .clickable(true)
                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                                .show(|ui| {
                                                    ui.text("Reject all")
                                                        .size(11.0)
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
                                                .padding(2.0, 8.0, 2.0, 8.0)
                                                .radius_all(4.0)
                                                .bg(colors.accent)
                                                .hover_bg(colors.accent.with_alpha(0.85))
                                                .show(|ui| {
                                                    ui.text("Accept all")
                                                        .size(11.0)
                                                        .weight(FontWeight::SemiBold)
                                                        .color(Color::WHITE)
                                                        .show();
                                                });
                                            if acc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if acc_btn.clicked {
                                                action_accept_all = true;
                                            }

                                            // Top arrow button to open/expand
                                            let arrow_btn = ui.container()
                                                .id(Id::from_u64(999999868))
                                                .clickable(true)
                                                .padding(2.0, 5.0, 2.0, 5.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05))
                                                .show(|ui| {
                                                    ui.text("\u{f077}")
                                                        .size(7.5)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });
                                            if arrow_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if arrow_btn.clicked {
                                                state.show_modified_files_popup = true;
                                                ui.needs_redraw = true;
                                            }
                                        });
                                }
                            });
                    }

                    let mut input_box_builder = ui.container()
                        .full_width()
                        .bg(input_box_bg)
                        .radius_all(14.0) // Matching rounded-2xl in HTML mockup
                        .padding(8.0, 12.0, 8.0, 12.0)
                        .column()
                        .gap(5.0);

                    if is_input_focused {
                        // Soft ambient glow when prompt box is active / focused
                        input_box_builder = input_box_builder.shadow(colors.accent.with_alpha(0.24), 0.0, 0.0, 12.0);
                    } else if state.ai_settings.effective_input_shadow() {
                        input_box_builder = input_box_builder.shadow(input_shadow_col, 0.0, 4.0, 18.0);
                    }

                    if state.ai_settings.effective_input_outline() {
                        let (outline_col, outline_w) = if is_input_focused {
                            (colors.accent.with_alpha(0.82), 0.75)
                        } else {
                            (input_border_col.with_alpha(0.70), 0.65)
                        };
                        input_box_builder = input_box_builder.border(outline_col, outline_w);
                    }

                    let prompt_text_size = state.ai_settings.effective_text_size().pt();

                    let input_box = input_box_builder.show(|ui| {
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
                            let placeholder = if input_dock_w >= 240.0 {
                                "Ask anything, @ to mention, / for actions"
                            } else {
                                "Ask..."
                            };

                            let input_w = (input_dock_w - 50.0).max(64.0);
                            let orb_id = Id::from_u64(999999990);
                            let mut input_submitted = false;
                            ui.container()
                                .full_width()
                                .row()
                                .valign(Align::Top)
                                .gap(4.0)
                                .show(|ui| {
                                    ui.container()
                                        .id(orb_id)
                                        .width(30.0)
                                        .height(30.0)
                                        .radius_all(8.0)
                                        .render_mode(zenthra::RenderMode::Continuous)
                                        .show(|ui| {
                                            let orb_kind = crate::ui_layout::thinking_orbs_demo_window::OrbStateKind::ALL
                                                [state.thinking_orbs_demo_mode % crate::ui_layout::thinking_orbs_demo_window::OrbStateKind::ALL.len()];
                                            let orb_time = state.orb_animation_start.elapsed().as_secs_f32();
                                            let orb_x = ui.cursor_x;
                                            let orb_y = ui.cursor_y;
                                            crate::ui_layout::thinking_orbs_demo_window::draw_inline_thinking_orb(
                                                ui,
                                                orb_kind,
                                                orb_x,
                                                orb_y,
                                                30.0,
                                                orb_time,
                                                colors,
                                            );
                                        });

                                    let text_res = ui.text_area(&mut state.ai_input, "ai_prompt_text_area")
                                        .width(input_w)
                                        .height(prompt_box_h)
                                        .max_height(MAX_PROMPT_H)
                                        .scrollable(true)
                                        .size(prompt_text_size)
                                        .line_height(1.30)
                                        .color(colors.text_primary)
                                        .cursor_color(colors.text_primary)
                                        .bg(if state.glassmorphism { Color::TRANSPARENT } else { input_box_bg })
                                        .border(Color::TRANSPARENT, 0.0)
                                        .focus_border(Color::TRANSPARENT, 0.0)
                                        .padding(2.0, 2.0, 2.0, 2.0)
                                        .placeholder(placeholder)
                                        .placeholder_color(colors.text_muted)
                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                        .submit_on_enter(true)
                                        .show();
                                    input_submitted = text_res.submitted;
                                });


                            // Bottom bar of input dock: Mode & Model pills on left, Send button on right
                            ui.container()
                                .full_width()
                                .height(26.0)
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .show(|ui| {
                                    // Left: Agent Mode pill + Model Selector pill
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(8.0)
                                        .show(|ui| {
                                            // 1. Agent Mode button
                                            let mode_btn = ui.container()
                                                .raw_id(AGENT_MODE_BTN_ID)
                                                .clickable(true)
                                                .padding(3.5, 8.0, 3.5, 8.0)
                                                .radius_all(6.0)
                                                .bg(if state.agent_mode_menu_open { pill_hover_bg } else { pill_bg })
                                                .hover_bg(pill_hover_bg)
                                                .border(pill_border, 1.0)
                                                .row()
                                                .valign(Align::Center)
                                                .gap(5.0)
                                                .show(|ui| {
                                                    ui.text(state.agent_mode.icon())
                                                        .size(9.5)
                                                        .color(colors.text_muted)
                                                        .show();
                                                    ui.text(state.agent_mode.display_name())
                                                        .size(10.0)
                                                        .weight(FontWeight::Medium)
                                                        .color(colors.text_primary)
                                                        .show();
                                                    ui.text(if state.agent_mode_menu_open { "\u{f077}" } else { "\u{f078}" })
                                                        .size(7.0)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });
                                            if mode_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if mode_btn.clicked {
                                                ui.clicked = false;
                                                state.agent_mode_menu_open = !state.agent_mode_menu_open;
                                                if state.agent_mode_menu_open {
                                                    state.ai_model_menu_open = false;
                                                }
                                                ui.needs_redraw = true;
                                            }

                                            // 2. Model Selector button
                                            let model_btn = ui.container()
                                                .raw_id(MODEL_PICKER_BTN_ID)
                                                .clickable(true)
                                                .padding(3.5, 8.0, 3.5, 8.0)
                                                .radius_all(6.0)
                                                .bg(if state.ai_model_menu_open { pill_hover_bg } else { pill_bg })
                                                .hover_bg(pill_hover_bg)
                                                .border(pill_border, 1.0)
                                                .row()
                                                .valign(Align::Center)
                                                .gap(5.0)
                                                .show(|ui| {
                                                    ui.text(state.selected_ai_model.display_name())
                                                        .size(10.0)
                                                        .weight(FontWeight::Medium)
                                                        .color(colors.text_primary)
                                                        .show();
                                                    ui.text(if state.ai_model_menu_open { "\u{f077}" } else { "\u{f078}" })
                                                        .size(7.0)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });
                                            if model_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if model_btn.clicked {
                                                ui.clicked = false;
                                                state.ai_model_menu_open = !state.ai_model_menu_open;
                                                if state.ai_model_menu_open {
                                                    state.agent_mode_menu_open = false;
                                                }
                                                ui.needs_redraw = true;
                                            }
                                        });

                                    // Right: Sleek Send Button (or Stop Button during processing)
                                    let is_processing = state.ai_messages.last().map(|m| m.is_working).unwrap_or(false);
                                    let has_text = !state.ai_input.trim().is_empty();

                                    let send_bg = if is_processing {
                                        Color::rgb(37.0 / 255.0, 37.0 / 255.0, 43.0 / 255.0) // #25252b
                                    } else if has_text {
                                        colors.accent
                                    } else {
                                        pill_bg
                                    };

                                    let send_hover_bg = if is_processing {
                                        Color::rgb(47.0 / 255.0, 47.0 / 255.0, 55.0 / 255.0) // #2f2f37
                                    } else if has_text {
                                        colors.highlight
                                    } else {
                                        pill_hover_bg
                                    };

                                    let send_border = if is_processing {
                                        Color::rgb(46.0 / 255.0, 46.0 / 255.0, 56.0 / 255.0) // #2e2e38
                                    } else {
                                        pill_border
                                    };

                                    let send_btn = ui.container()
                                        .width(24.0)
                                        .height(24.0)
                                        .radius_all(5.0)
                                        .bg(send_bg)
                                        .hover_bg(send_hover_bg)
                                        .border(send_border, 1.0)
                                        .halign(Align::Center)
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            if is_processing {
                                                // Square red stop box matching HTML mockup
                                                ui.container()
                                                    .width(8.0)
                                                    .height(8.0)
                                                    .radius_all(1.5)
                                                    .bg(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0)) // #ef4444
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
                                            state.cancel_active_ai();
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
                                    } else if trigger_send || input_submitted || send_btn.clicked {
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
                        for tab in &mut state.tabs {
                            tab.lines_added = 0;
                            tab.lines_removed = 0;
                            tab.is_dirty = false;
                        }
                        for msg in &mut state.ai_messages {
                            msg.trajectory.retain(|step| !matches!(step, TrajectoryStep::FileEdit { .. }));
                        }
                        state.show_modified_files_popup = false;
                        ui.needs_redraw = true;
                    }
                    if action_reject_all {
                        state.reject_all_reviews();
                        for tab in &mut state.tabs {
                            tab.lines_added = 0;
                            tab.lines_removed = 0;
                            tab.is_dirty = false;
                        }
                        for msg in &mut state.ai_messages {
                            msg.trajectory.retain(|step| !matches!(step, TrajectoryStep::FileEdit { .. }));
                        }
                        state.show_modified_files_popup = false;
                        ui.needs_redraw = true;
                    }
                    if let Some(path) = target_reject_file {
                        state.reject_review_for_path(&path);
                        for tab in &mut state.tabs {
                            if tab.path.as_ref().map(|p| path_matches(p, &path)).unwrap_or(false) {
                                tab.lines_added = 0;
                                tab.lines_removed = 0;
                                tab.is_dirty = false;
                            }
                        }
                        for msg in &mut state.ai_messages {
                            msg.trajectory.retain(|step| {
                                if let TrajectoryStep::FileEdit { path: edit_path, .. } = step {
                                    let pb = std::path::PathBuf::from(edit_path);
                                    let full = if pb.is_absolute() { pb } else { state.current_dir.join(&pb) };
                                    !path_matches(&full, &path) && edit_path != &path.to_string_lossy()
                                } else {
                                    true
                                }
                            });
                        }
                        ui.needs_redraw = true;
                    }
                    if let Some(path) = target_open_file {
                        state.open_file(&path);
                        ui.needs_redraw = true;
                    }
                });
            });
}


use pulldown_cmark::{
    Alignment, BlockQuoteKind, Event, HeadingLevel, Options, Parser, Tag, TagEnd,
};

// ─────────────────────────────────────────────────────────────────────────────
// Inline span model
// ─────────────────────────────────────────────────────────────────────────────

/// A fragment of inline-formatted text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum InlineSpan {
    /// Plain text
    Text(String),
    /// `**bold**`
    Bold(String),
    /// `*italic*`
    Italic(String),
    /// `***bold-italic***`
    BoldItalic(String),
    /// `` `code` ``
    Code(String),
    /// ~~strikethrough~~
    #[allow(dead_code)]
    Strike(String),
}

impl InlineSpan {
    #[cfg(test)]
    fn plain_text(&self) -> &str {
        match self {
            Self::Text(s)
            | Self::Bold(s)
            | Self::Italic(s)
            | Self::BoldItalic(s)
            | Self::Code(s)
            | Self::Strike(s) => s,
        }
    }

    #[cfg(test)]
    fn to_plain_string(spans: &[InlineSpan]) -> String {
        spans.iter().map(|s| s.plain_text()).collect()
    }
}

/// GitHub-style blockquote kinds.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum QuoteKind {
    Note,
    Tip,
    Important,
    Warning,
    Caution,
    Plain,
}

// ─────────────────────────────────────────────────────────────────────────────
// Block model
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum MarkdownBlock {
    Heading { level: u8, spans: Vec<InlineSpan> },
    CodeBlock { lang: Option<String>, code: String },
    /// depth: 0 = top-level, 1 = nested, etc.  checked: None = not a task item.
    BulletItem { depth: u8, spans: Vec<InlineSpan>, checked: Option<bool> },
    NumberedItem { num: String, depth: u8, spans: Vec<InlineSpan> },
    Table {
        alignments: Vec<Alignment>,
        headers: Vec<Vec<InlineSpan>>,
        rows: Vec<Vec<Vec<InlineSpan>>>,
    },
    BlockQuote { kind: QuoteKind, spans: Vec<InlineSpan> },
    Rule,
    Paragraph(Vec<InlineSpan>),
}

// ── Compat helpers used only in tests ─────────────────────────────────────────
#[allow(dead_code)]
impl MarkdownBlock {
    #[cfg(test)]
    fn plain_paragraph(s: impl Into<String>) -> Self {
        MarkdownBlock::Paragraph(vec![InlineSpan::Text(s.into())])
    }
    #[cfg(test)]
    fn plain_heading(level: u8, s: impl Into<String>) -> Self {
        MarkdownBlock::Heading { level, spans: vec![InlineSpan::Text(s.into())] }
    }
    #[cfg(test)]
    fn plain_bullet(s: impl Into<String>) -> Self {
        MarkdownBlock::BulletItem { depth: 0, spans: vec![InlineSpan::Text(s.into())], checked: None }
    }
    #[cfg(test)]
    fn plain_numbered(num: impl Into<String>, s: impl Into<String>) -> Self {
        MarkdownBlock::NumberedItem { num: num.into(), depth: 0, spans: vec![InlineSpan::Text(s.into())] }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Parser
// ─────────────────────────────────────────────────────────────────────────────



// ─────────────────────────────────────────────────────────────────────────────
// Actual parser (replaces the broken draft above)
// ─────────────────────────────────────────────────────────────────────────────
/// Parse a markdown string into a flat list of [`MarkdownBlock`]s using pulldown-cmark.
/// This is the real implementation used at runtime.
pub(crate) fn parse_markdown_blocks_impl(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks: Vec<MarkdownBlock> = Vec::new();

    let opts = Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_TABLES;
    let parser = Parser::new_ext(content, opts);

    // ── Inline span accumulator ─────────────────────────────────────────────
    let mut spans: Vec<InlineSpan> = Vec::new();
    let mut text_buf = String::new();
    let mut in_strong: u8 = 0; // nesting depth
    let mut in_em: u8 = 0;
    let mut in_strike: u8 = 0;

    // ── Block state ─────────────────────────────────────────────────────────
    let mut heading_level: u8 = 0;
    // list nesting stack: (is_ordered, next_counter)
    let mut list_stack: Vec<(bool, u64)> = Vec::new();
    let mut task_checked: Option<bool> = None;

    // blockquote
    let mut bq_depth: u8 = 0;
    let mut bq_kind = QuoteKind::Plain;
    let mut bq_spans: Vec<InlineSpan> = Vec::new();
    let mut bq_text = String::new();
    let mut bq_strong: u8 = 0;
    let mut bq_em: u8 = 0;

    // table state
    let mut table_alignments: Vec<Alignment> = Vec::new();
    let mut in_table_head = false;
    let mut table_headers: Vec<Vec<InlineSpan>> = Vec::new();
    let mut table_rows: Vec<Vec<Vec<InlineSpan>>> = Vec::new();
    let mut cur_row: Vec<Vec<InlineSpan>> = Vec::new();
    let mut in_cell = false;
    let mut cell_spans: Vec<InlineSpan> = Vec::new();
    let mut cell_text = String::new();
    let mut cell_strong: u8 = 0;
    let mut cell_em: u8 = 0;

    // code block
    let mut code_lang: Option<String> = None;
    let mut in_code_block = false;
    let mut code_buf = String::new();

    // ── Flush helpers ───────────────────────────────────────────────────────
    macro_rules! flush {
        ($buf:expr, $spans:expr, $s:expr, $e:expr) => {{
            if !$buf.is_empty() {
                let t = std::mem::take(&mut $buf);
                let sp = match ($s > 0, $e > 0) {
                    (true, true) => InlineSpan::BoldItalic(t),
                    (true, false) => InlineSpan::Bold(t),
                    (false, true) => InlineSpan::Italic(t),
                    _ => InlineSpan::Text(t),
                };
                $spans.push(sp);
            }
        }};
    }

    for event in parser {
        // ── Code block raw accumulation (highest priority) ───────────────────
        if in_code_block {
            match event {
                Event::Text(t) => { code_buf.push_str(&t); }
                Event::End(TagEnd::CodeBlock) => {
                    let code = std::mem::take(&mut code_buf);
                    let code = code.trim_end_matches('\n').to_string();
                    blocks.push(MarkdownBlock::CodeBlock { lang: code_lang.take(), code });
                    in_code_block = false;
                }
                _ => {}
            }
            continue;
        }

        // ── Table cell inline accumulation ───────────────────────────────────
        if in_cell {
            match event {
                Event::Text(t) => { cell_text.push_str(&t); }
                Event::Code(t) => {
                    flush!(cell_text, cell_spans, cell_strong, cell_em);
                    cell_spans.push(InlineSpan::Code(t.to_string()));
                }
                Event::Start(Tag::Strong) => { flush!(cell_text, cell_spans, cell_strong, cell_em); cell_strong += 1; }
                Event::End(TagEnd::Strong) => { flush!(cell_text, cell_spans, cell_strong, cell_em); cell_strong = cell_strong.saturating_sub(1); }
                Event::Start(Tag::Emphasis) => { flush!(cell_text, cell_spans, cell_strong, cell_em); cell_em += 1; }
                Event::End(TagEnd::Emphasis) => { flush!(cell_text, cell_spans, cell_strong, cell_em); cell_em = cell_em.saturating_sub(1); }
                Event::SoftBreak | Event::HardBreak => { cell_text.push(' '); }
                Event::End(TagEnd::TableCell) => {
                    flush!(cell_text, cell_spans, cell_strong, cell_em);
                    cur_row.push(std::mem::take(&mut cell_spans));
                    in_cell = false;
                    cell_strong = 0; cell_em = 0;
                }
                _ => {}
            }
            continue;
        }

        // ── Blockquote inline accumulation ───────────────────────────────────
        if bq_depth > 0 {
            match event {
                Event::Text(t) => { bq_text.push_str(&t); }
                Event::Code(t) => {
                    flush!(bq_text, bq_spans, bq_strong, bq_em);
                    bq_spans.push(InlineSpan::Code(t.to_string()));
                }
                Event::Start(Tag::Strong) => { flush!(bq_text, bq_spans, bq_strong, bq_em); bq_strong += 1; }
                Event::End(TagEnd::Strong) => { flush!(bq_text, bq_spans, bq_strong, bq_em); bq_strong = bq_strong.saturating_sub(1); }
                Event::Start(Tag::Emphasis) => { flush!(bq_text, bq_spans, bq_strong, bq_em); bq_em += 1; }
                Event::End(TagEnd::Emphasis) => { flush!(bq_text, bq_spans, bq_strong, bq_em); bq_em = bq_em.saturating_sub(1); }
                Event::SoftBreak => { bq_text.push(' '); }
                Event::HardBreak => { bq_text.push('\n'); }
                // Paragraphs inside blockquotes just emit into bq_spans.
                Event::Start(Tag::Paragraph) | Event::End(TagEnd::Paragraph) => {}
                Event::End(TagEnd::BlockQuote(_)) => {
                    bq_depth -= 1;
                    if bq_depth == 0 {
                        flush!(bq_text, bq_spans, bq_strong, bq_em);
                        let s = std::mem::take(&mut bq_spans);
                        if !s.is_empty() {
                            blocks.push(MarkdownBlock::BlockQuote {
                                kind: std::mem::replace(&mut bq_kind, QuoteKind::Plain),
                                spans: s,
                            });
                        }
                    }
                }
                _ => {}
            }
            continue;
        }

        match event {
            // ── Horizontal rule ──────────────────────────────────────────────
            Event::Rule => {
                blocks.push(MarkdownBlock::Rule);
            }

            // ── Blockquote ───────────────────────────────────────────────────
            Event::Start(Tag::BlockQuote(kind)) => {
                bq_kind = match kind {
                    Some(BlockQuoteKind::Note) => QuoteKind::Note,
                    Some(BlockQuoteKind::Tip) => QuoteKind::Tip,
                    Some(BlockQuoteKind::Important) => QuoteKind::Important,
                    Some(BlockQuoteKind::Warning) => QuoteKind::Warning,
                    Some(BlockQuoteKind::Caution) => QuoteKind::Caution,
                    None => QuoteKind::Plain,
                };
                bq_depth += 1;
                bq_spans.clear();
                bq_text.clear();
            }

            // ── Headings ─────────────────────────────────────────────────────
            Event::Start(Tag::Heading { level, .. }) => {
                spans.clear(); text_buf.clear();
                in_strong = 0; in_em = 0; in_strike = 0;
                heading_level = match level {
                    HeadingLevel::H1 => 1, HeadingLevel::H2 => 2, HeadingLevel::H3 => 3,
                    HeadingLevel::H4 => 4, HeadingLevel::H5 => 5, HeadingLevel::H6 => 6,
                };
            }
            Event::End(TagEnd::Heading(_)) => {
                flush!(text_buf, spans, in_strong, in_em);
                let s = std::mem::take(&mut spans);
                if !s.is_empty() { blocks.push(MarkdownBlock::Heading { level: heading_level, spans: s }); }
                heading_level = 0;
            }

            // ── Paragraphs ───────────────────────────────────────────────────
            Event::Start(Tag::Paragraph) => { spans.clear(); text_buf.clear(); in_strong = 0; in_em = 0; in_strike = 0; }
            Event::End(TagEnd::Paragraph) => {
                flush!(text_buf, spans, in_strong, in_em);
                let s = std::mem::take(&mut spans);
                if !s.is_empty() { blocks.push(MarkdownBlock::Paragraph(s)); }
            }

            // ── Inline bold / italic / strike ─────────────────────────────────
            Event::Start(Tag::Strong) => { flush!(text_buf, spans, in_strong, in_em); in_strong += 1; }
            Event::End(TagEnd::Strong) => { flush!(text_buf, spans, in_strong, in_em); in_strong = in_strong.saturating_sub(1); }
            Event::Start(Tag::Emphasis) => { flush!(text_buf, spans, in_strong, in_em); in_em += 1; }
            Event::End(TagEnd::Emphasis) => { flush!(text_buf, spans, in_strong, in_em); in_em = in_em.saturating_sub(1); }
            Event::Start(Tag::Strikethrough) => { flush!(text_buf, spans, in_strong, in_em); in_strike += 1; }
            Event::End(TagEnd::Strikethrough) => { flush!(text_buf, spans, in_strong, in_em); in_strike = in_strike.saturating_sub(1); }

            // ── Lists ─────────────────────────────────────────────────────────
            Event::Start(Tag::List(start_num)) => {
                list_stack.push((start_num.is_some(), start_num.unwrap_or(1)));
            }
            Event::End(TagEnd::List(_)) => { list_stack.pop(); }
            Event::Start(Tag::Item) => {
                spans.clear(); text_buf.clear();
                in_strong = 0; in_em = 0; in_strike = 0;
                task_checked = None;
            }
            Event::TaskListMarker(checked) => { task_checked = Some(checked); }
            Event::End(TagEnd::Item) => {
                flush!(text_buf, spans, in_strong, in_em);
                let s = std::mem::take(&mut spans);
                if !s.is_empty() {
                    let depth = (list_stack.len().saturating_sub(1)) as u8;
                    if let Some(last) = list_stack.last_mut() {
                        if last.0 {
                            let num = last.1.to_string();
                            last.1 += 1;
                            blocks.push(MarkdownBlock::NumberedItem { num, depth, spans: s });
                        } else {
                            blocks.push(MarkdownBlock::BulletItem { depth, spans: s, checked: task_checked });
                        }
                    }
                }
                task_checked = None;
            }

            // ── Fenced code blocks ────────────────────────────────────────────
            Event::Start(Tag::CodeBlock(kind)) => {
                code_lang = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(l) => {
                        let l = l.trim().to_string();
                        if l.is_empty() { None } else { Some(l) }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };
                code_buf.clear();
                in_code_block = true;
            }

            // ── Tables ────────────────────────────────────────────────────────
            Event::Start(Tag::Table(aligns)) => {

                table_alignments = aligns;
                table_headers.clear();
                table_rows.clear();
            }
            Event::End(TagEnd::Table) => {
                blocks.push(MarkdownBlock::Table {
                    alignments: std::mem::take(&mut table_alignments),
                    headers: std::mem::take(&mut table_headers),
                    rows: std::mem::take(&mut table_rows),
                });

            }
            Event::Start(Tag::TableHead) => { in_table_head = true; cur_row.clear(); }
            Event::End(TagEnd::TableHead) => {
                table_headers = std::mem::take(&mut cur_row);
                in_table_head = false;
            }
            Event::Start(Tag::TableRow) => { cur_row.clear(); }
            Event::End(TagEnd::TableRow) => {
                if !in_table_head {
                    table_rows.push(std::mem::take(&mut cur_row));
                }
            }
            Event::Start(Tag::TableCell) => {
                in_cell = true;
                cell_spans.clear();
                cell_text.clear();
                cell_strong = 0; cell_em = 0;
            }
            // TableCell End handled inside in_cell branch above.

            // ── Inline text & inline code ─────────────────────────────────────
            Event::Text(t) => { text_buf.push_str(&t); }
            Event::Code(t) => {
                flush!(text_buf, spans, in_strong, in_em);
                spans.push(InlineSpan::Code(t.to_string()));
            }
            Event::SoftBreak => { text_buf.push(' '); }
            Event::HardBreak => { text_buf.push('\n'); }

            _ => {}
        }
    }

    blocks
}

/// Render a slice of [`InlineSpan`]s as a horizontally-wrapped rich text row.
///
/// Because the underlying text widget can only render a single style at a time we emit
/// each span as its own text widget inside a wrapping row container. Inline `Code` spans
/// get a subtle pill background drawn around them.
pub(crate) fn draw_inline_spans(
    ui: &mut Ui,
    spans: &[InlineSpan],
    max_width: f32,
    base_size: f32,
    base_weight: FontWeight,
    base_color: Color,
    colors: &crate::theme::ThemeColors,
) {
    if spans.is_empty() {
        return;
    }

    // Fast-path: single plain text span (most common case)
    if spans.len() == 1 {
        match &spans[0] {
            InlineSpan::Text(t) => {
                ui.text(t)
                    .size(base_size)
                    .weight(base_weight)
                    .color(base_color)
                    .line_height(1.6)
                    .max_width(max_width)
                    .wrap(zenthra_text::prelude::TextWrap::Word)
                    .show();
                return;
            }
            InlineSpan::Code(t) => {
                ui.container()
                    .radius_all(3.0)
                    .bg(colors.bg_base.with_alpha(0.8))
                    .border(colors.border.with_alpha(0.4), 0.5)
                    .padding(1.0, 4.0, 1.0, 4.0)
                    .show(|ui| {
                        ui.text(t)
                            .size((base_size - 0.5).max(8.5))
                            .monospace()
                            .color(colors.accent.with_alpha(0.9))
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    });
                return;
            }
            _ => {}
        }
    }

    // Multi-span: row with wrapping
    ui.container()
        .full_width()
        .row()
        .wrap(Wrap::Auto)
        .valign(Align::Top)
        .gap(0.0)
        .show(|ui| {
            for span in spans {
                match span {
                    InlineSpan::Text(t) | InlineSpan::Strike(t) => {
                        let is_strike = matches!(span, InlineSpan::Strike(_));
                        let color = if is_strike {
                            base_color.with_alpha(0.5)
                        } else {
                            base_color
                        };
                        ui.text(t)
                            .size(base_size)
                            .weight(base_weight)
                            .color(color)
                            .line_height(1.6)
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::Bold(t) => {
                        ui.text(t)
                            .size(base_size)
                            .weight(FontWeight::SemiBold)
                            .color(base_color)
                            .line_height(1.6)
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::Italic(t) => {
                        ui.text(t)
                            .size(base_size)
                            .weight(base_weight)
                            .italic()
                            .color(base_color.with_alpha(0.85))
                            .line_height(1.6)
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::BoldItalic(t) => {
                        ui.text(t)
                            .size(base_size)
                            .weight(FontWeight::SemiBold)
                            .italic()
                            .color(base_color)
                            .line_height(1.6)
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::Code(t) => {
                        ui.container()
                            .radius_all(3.0)
                            .bg(colors.bg_base.with_alpha(0.8))
                            .border(colors.border.with_alpha(0.4), 0.5)
                            .padding(1.0, 4.0, 1.0, 4.0)
                            .show(|ui| {
                                ui.text(t)
                                    .size((base_size - 0.5).max(8.5))
                                    .monospace()
                                    .color(colors.accent.with_alpha(0.9))
                                    .max_width(max_width)
                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                    .show();
                            });
                    }
                }
            }
        });
}

fn draw_session_history_panel(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    panel_w: f32,
    panel_h: f32,
    colors: &crate::theme::ThemeColors,
) {
    let mut load_session_id: Option<String> = None;
    let mut delete_session_id: Option<String> = None;
    let mut toggle_project_path: Option<String> = None;

    let hist_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 0.90).clamp(0.30, 0.90))
    } else {
        colors.bg_sidebar
    };
    let header_bg = if state.glassmorphism {
        Color::TRANSPARENT
    } else {
        colors.bg_sidebar
    };

    let hist_builder = ui.container()
        .width(panel_w)
        .height(panel_h)
        .max_height(panel_h)
        .bg(hist_bg)
        .column()
        .clip(true);

    hist_builder.show(|ui| {
            // ── Top Header Bar (Full height aligned with chat header) ────────
            const HEADER_H: f32 = 36.0;
            ui.container()
                .width(panel_w)
                .height(HEADER_H)
                .padding_left(10.0)
                .padding_right(10.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .bg(header_bg)
                .show(|ui| {
                    if !state.ai_panel_expanded {
                        // Docked mode: Back button with chevron + "Chat"
                        let back_btn = ui.container()
                            .clickable(true)
                            .padding(2.0, 6.0, 2.0, 4.0)
                            .radius_all(4.0)
                            .hover_bg(colors.highlight)
                            .row()
                            .valign(Align::Center)
                            .gap(5.0)
                            .show(|ui| {
                                ui.text("\u{f053}") // NF_FA_CHEVRON_LEFT
                                    .size(9.0)
                                    .color(colors.accent)
                                    .show();
                                ui.text("Chat")
                                    .size(11.0)
                                    .color(colors.text_primary)
                                    .show();
                            });
                        if back_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                        if back_btn.clicked {
                            state.ai_sessions_visible = false;
                            ui.needs_redraw = true;
                        }
                    } else {
                        // Expanded mode: Clock icon + "Session History"
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text("\u{f1da}") // fa-clock-rotate-left
                                    .size(10.0)
                                    .color(colors.accent)
                                    .show();
                                ui.text("Session History")
                                    .size(11.5)
                                    .weight(FontWeight::Medium)
                                    .color(colors.text_primary)
                                    .show();
                            });
                    }

                    // Right: Close button
                    let close_side_btn = ui.container()
                        .clickable(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .hover_bg(colors.highlight)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f00d}")
                                .size(9.5)
                                .color(colors.text_muted)
                                .show();
                        });
                    if close_side_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if close_side_btn.clicked {
                        state.ai_sessions_visible = false;
                        ui.needs_redraw = true;
                    }
                });

            // 1px hairline below header
            ui.container()
                .width(panel_w)
                .height(1.0)
                .bg(colors.border)
                .show(|_| {});

            // ── Search Bar (Native pill with no outline) ──────────────
            let search_h: f32 = 34.0;
            let search_input_id = Id::from_u64(999_999_888);
            ui.container()
                .width(panel_w)
                .height(search_h)
                .padding(5.0, 10.0, 5.0, 10.0)
                .row()
                .valign(Align::Center)
                .bg(colors.bg_sidebar)
                .show(|ui| {
                    ui.container()
                        .fill_x()
                        .height(24.0)
                        .radius_all(4.0)
                        .padding(0.0, 8.0, 0.0, 8.0)
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .bg(colors.bg_base.with_alpha(0.6))
                        .show(|ui| {
                            ui.text("\u{f002}") // fa-search
                                .size(9.5)
                                .color(colors.text_muted)
                                .show();

                            let query_empty = state.ai_sessions_search_query.is_empty();

                            ui.input(&mut state.ai_sessions_search_query, search_input_id)
                                .fill_x()
                                .size(11.0)
                                .color(colors.text_primary)
                                .no_bg()
                                .no_border()
                                .show();

                            if !query_empty {
                                let clear_btn = ui.container()
                                    .padding(2.0, 4.0, 2.0, 4.0)
                                    .radius_all(3.0)
                                    .hover_bg(colors.highlight)
                                    .clickable(true)
                                    .show(|ui| {
                                        ui.text("\u{f00d}")
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                                if clear_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if clear_btn.clicked {
                                    state.ai_sessions_search_query.clear();
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });

            // 1px hairline below search bar
            ui.container()
                .width(panel_w)
                .height(1.0)
                .bg(colors.border.with_alpha(0.35))
                .show(|_| {});

            // ── Scrollable Sessions List ────────────────────────────
            let list_h = (panel_h - HEADER_H - 1.0 - search_h - 1.0).max(20.0);
            let scroll_id = Id::from_u64(999_999_889);

            ui.container()
                .id(scroll_id)
                .width(panel_w)
                .height(list_h)
                .max_height(list_h)
                .padding(8.0, 10.0, 12.0, 10.0)
                .column()
                .gap(12.0)
                .scroll_y(true)
                .clip(true)
                .show(|ui| {
                    let cur_canon = state.current_dir.canonicalize().unwrap_or_else(|_| state.current_dir.clone());
                    let cur_proj_str = cur_canon.to_string_lossy().to_string();

                    // Group sessions by project
                    // 1. Collect unique project paths in order
                    let mut project_paths: Vec<(String, String)> = Vec::new();
                    if state.has_workspace {
                        // Current workspace project is placed at the top
                        let cur_name = cur_canon.file_name().and_then(|n| n.to_str()).unwrap_or("project").to_string();
                        project_paths.push((cur_proj_str.clone(), cur_name));
                    } else {
                        // When no workspace is opened, place "General" at the top
                        project_paths.push(("general".to_string(), "General".to_string()));
                    }

                    for sess in &state.ai_sessions {
                        if !project_paths.iter().any(|(p, _)| p == &sess.project_path) {
                            project_paths.push((sess.project_path.clone(), sess.project_name.clone()));
                        }
                    }

                    let query_lower = state.ai_sessions_search_query.to_lowercase();
                    let mut total_rendered_sessions = 0;

                    for (proj_idx, (proj_path, proj_name)) in project_paths.iter().enumerate() {
                        let is_general = proj_path == "general";
                        let is_current = if state.has_workspace {
                            proj_path == &cur_proj_str
                        } else {
                            is_general
                        };

                        // Filter sessions for this project
                        let filtered_sessions: Vec<&SessionSummary> = state.ai_sessions.iter()
                            .filter(|s| &s.project_path == proj_path)
                            .filter(|s| {
                                if query_lower.is_empty() {
                                    true
                                } else {
                                    s.title.to_lowercase().contains(&query_lower)
                                        || s.project_name.to_lowercase().contains(&query_lower)
                                }
                            })
                            .collect();

                        // If user is searching and no sessions match this project, skip project
                        if !query_lower.is_empty() && filtered_sessions.is_empty() {
                            continue;
                        }

                        let session_count = filtered_sessions.len();
                        total_rendered_sessions += session_count;

                        // By default, current project / active general is expanded
                        let is_expanded = state.ai_expanded_projects.contains(proj_path)
                            || (!query_lower.is_empty() && !filtered_sessions.is_empty());

                        let badge_reserve = if is_current { 52.0 } else { 0.0 };
                        let avail_w = (panel_w - 32.0).max(180.0);
                        let left_max_w = (avail_w - badge_reserve).max(120.0);

                        // ── Project Line with details ────────────────────────
                        // ▾ 💬 General · 1 session · No workspace (active)  OR  ▾ 📁 zenthree (current)
                        let proj_line_id = Id::from_u64(0x7F00_0000 + (proj_idx as u64) * 100);

                        let proj_hdr = ui.container()
                            .id(proj_line_id)
                            .full_width()
                            .clickable(true)
                            .padding(3.0, 4.0, 3.0, 4.0)
                            .radius_all(4.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .no_wrap()
                            .show(|ui| {
                                // Left details: arrow, icon, name, count, optional path
                                ui.container()
                                    .max_width(left_max_w)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(5.0)
                                    .no_wrap()
                                    .show(|ui| {
                                        ui.container()
                                            .width(12.0)
                                            .valign(Align::Center)
                                            .halign(Align::Center)
                                            .show(|ui| {
                                                let arrow = if is_expanded { "\u{f078}" } else { "\u{f054}" };
                                                ui.text(arrow)
                                                    .size(7.5)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });

                                        if is_general {
                                            ui.text("\u{f086}") // fa-comments
                                                .size(10.5)
                                                .color(colors.accent)
                                                .show();
                                        } else {
                                            ui.text("\u{f07b}") // fa-folder
                                                .size(10.5)
                                                .color(colors.accent)
                                                .show();
                                        }

                                        let display_name = if is_general {
                                            "General"
                                        } else {
                                            proj_name.as_str()
                                        };

                                        ui.text(display_name)
                                            .size(11.5)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .max_width(75.0)
                                            .ellipsis(true)
                                            .show();

                                        ui.text("·")
                                            .size(10.0)
                                            .color(colors.text_muted.with_alpha(0.6))
                                            .show();

                                        ui.text(&format!("{} session{}", session_count, if session_count == 1 { "" } else { "s" }))
                                            .size(10.0)
                                            .color(colors.text_muted)
                                            .show();

                                        // Shortened path display or "No workspace"
                                        let remaining_for_path = left_max_w - 145.0;
                                        if is_general {
                                            if remaining_for_path >= 50.0 {
                                                ui.text("·")
                                                    .size(10.0)
                                                    .color(colors.text_muted.with_alpha(0.5))
                                                    .show();

                                                ui.text("No workspace")
                                                    .size(9.5)
                                                    .color(colors.text_muted.with_alpha(0.65))
                                                    .max_width(remaining_for_path)
                                                    .ellipsis(true)
                                                    .show();
                                            }
                                        } else {
                                            let short_path = if let Ok(home) = std::env::var("HOME") {
                                                if let Some(rest) = proj_path.strip_prefix(&home) {
                                                    format!("~{}", rest)
                                                } else {
                                                    proj_path.clone()
                                                }
                                            } else {
                                                proj_path.clone()
                                            };
                                            if remaining_for_path >= 35.0 {
                                                let max_chars = (remaining_for_path / 6.2) as usize;
                                                let display_path = if short_path.len() > max_chars && max_chars > 6 {
                                                    format!("...{}", &short_path[short_path.len() - (max_chars.saturating_sub(3))..])
                                                } else {
                                                    short_path
                                                };

                                                ui.text("·")
                                                    .size(10.0)
                                                    .color(colors.text_muted.with_alpha(0.5))
                                                    .show();

                                                ui.text(&display_path)
                                                    .size(9.5)
                                                    .color(colors.text_muted.with_alpha(0.65))
                                                    .max_width(remaining_for_path)
                                                    .ellipsis(true)
                                                    .show();
                                            }
                                        }
                                    });

                                // Right side: Current workspace badge
                                if is_current {
                                    ui.container()
                                        .padding(1.0, 4.0, 1.0, 4.0)
                                        .radius_all(3.0)
                                        .bg(colors.accent.with_alpha(0.15))
                                        .valign(Align::Center)
                                        .show(|ui| {
                                            ui.text(if is_general { "active" } else { "current" })
                                                .size(8.5)
                                                .color(colors.accent)
                                                .weight(FontWeight::Medium)
                                                .show();
                                        });
                                }
                            });

                        if proj_hdr.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                        if proj_hdr.clicked {
                            toggle_project_path = Some(proj_path.clone());
                        }

                        // ── Project Session Items (if expanded) ─────────────
                        if is_expanded {
                            ui.container()
                                .full_width()
                                .column()
                                .gap(6.0)
                                .padding_left(16.0)
                                .show(|ui| {
                                    if filtered_sessions.is_empty() {
                                        ui.container()
                                            .padding(4.0, 16.0, 4.0, 16.0)
                                            .show(|ui| {
                                                ui.text("No sessions in this project yet.")
                                                    .size(10.5)
                                                    .italic()
                                                    .color(colors.text_muted.with_alpha(0.6))
                                                    .show();
                                            });
                                    } else {
                                        for (s_idx, sess) in filtered_sessions.iter().enumerate() {
                                            let is_active = state.ai_session_id.as_deref() == Some(&sess.id);
                                            let sess_card_id = Id::from_u64(0x8A00_0000 + (proj_idx as u64) * 1000 + (s_idx as u64) * 10);
                                            let sess_del_id = Id::from_u64(0x8A00_0000 + (proj_idx as u64) * 1000 + (s_idx as u64) * 10 + 1);

                                            let mut del_clicked = false;

                                            let card_res = ui.container()
                                                .id(sess_card_id)
                                                .full_width()
                                                .height(36.0)
                                                .clickable(true)
                                                .padding(0.0, 10.0, 0.0, 10.0)
                                                .radius_all(6.0)
                                                .row()
                                                .valign(Align::Center)
                                                .halign(Align::SpaceBetween)
                                                .bg(if is_active {
                                                    colors.accent.with_alpha(0.12)
                                                } else {
                                                    colors.bg_sidebar
                                                })
                                                .hover_bg(if is_active {
                                                    colors.accent.with_alpha(0.18)
                                                } else {
                                                    colors.highlight
                                                })
                                                .border(if is_active {
                                                    colors.accent.with_alpha(0.60)
                                                } else {
                                                    colors.border.with_alpha(0.50)
                                                }, 1.0)
                                                .show(|ui| {
                                                    // Left: optional active dot + Title
                                                    ui.container()
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(6.0)
                                                        .show(|ui| {
                                                            if is_active {
                                                                ui.container()
                                                                    .width(6.0)
                                                                    .height(6.0)
                                                                    .radius_all(3.0)
                                                                    .bg(colors.accent)
                                                                    .show(|_| {});
                                                            }

                                                            let title_max_w = (panel_w - 130.0).max(40.0);
                                                            ui.text(&sess.title)
                                                                .size(11.5)
                                                                .weight(if is_active { FontWeight::Bold } else { FontWeight::Medium })
                                                                .color(colors.text_primary)
                                                                .max_width(title_max_w)
                                                                .ellipsis(true)
                                                                .show();
                                                        });

                                                    // Right: Time on left side of Delete, Delete on far right
                                                    ui.container()
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(8.0)
                                                        .show(|ui| {
                                                            ui.text(&sess.relative_time_string())
                                                                .size(9.5)
                                                                .color(colors.text_muted.with_alpha(0.7))
                                                                .show();

                                                            let del_btn = ui.container()
                                                                .id(sess_del_id)
                                                                .clickable(true)
                                                                .padding(2.5, 5.0, 2.5, 5.0)
                                                                .radius_all(3.0)
                                                                .hover_bg(Color::rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.18))
                                                                .show(|ui| {
                                                                    ui.text("\u{f014}") // trash / delete icon
                                                                        .size(10.0)
                                                                        .color(colors.text_muted.with_alpha(0.75))
                                                                        .show();
                                                                });
                                                            if del_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                                            if del_btn.clicked {
                                                                del_clicked = true;
                                                            }
                                                        });
                                                });

                                            if card_res.hovered { ui.cursor_icon = CursorIcon::Pointer; }

                                            if del_clicked {
                                                delete_session_id = Some(sess.id.clone());
                                            } else if card_res.clicked {
                                                load_session_id = Some(sess.id.clone());
                                            }
                                        }
                                    }
                                });
                        }
                    }

                    if total_rendered_sessions == 0 && !query_lower.is_empty() {
                        ui.container()
                            .padding_top(20.0)
                            .halign(Align::Center)
                            .full_width()
                            .show(|ui| {
                                ui.text("No matching sessions found.")
                                    .size(11.0)
                                    .italic()
                                    .color(colors.text_muted)
                                    .show();
                            });
                    }
                });
        });

    if let Some(path) = toggle_project_path {
        if state.ai_expanded_projects.contains(&path) {
            state.ai_expanded_projects.remove(&path);
        } else {
            state.ai_expanded_projects.insert(path);
        }
        ui.needs_redraw = true;
    }

    if let Some(id) = delete_session_id {
        state.delete_ai_session(&id);
        ui.needs_redraw = true;
    } else if let Some(id) = load_session_id {
        state.load_ai_session(&id);
        if !state.ai_panel_expanded {
            state.ai_sessions_visible = false;
        }
        ui.needs_redraw = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_markdown_blocks_comprehensive() {
        let md = "I have inspected the project.\n\n## Diagnostics Summary\n- Errors: 0\n- Warnings: 0\n\n```rust\nfn main() {\n    println!(\"Hello Zenthra!\");\n}\n```\n\n1. First step\n2. Second step\n\nAll systems operational.";
        let blocks = parse_markdown_blocks_impl(md);

        // paragraph, heading, 2 bullets, code block, 2 numbered, paragraph
        assert_eq!(blocks.len(), 8, "got: {:#?}", blocks);

        assert!(matches!(&blocks[0], MarkdownBlock::Paragraph(s) if InlineSpan::to_plain_string(s).contains("I have inspected")));
        assert!(matches!(&blocks[1], MarkdownBlock::Heading { level: 2, .. }));
        assert!(matches!(&blocks[2], MarkdownBlock::BulletItem { depth: 0, .. }));
        assert!(matches!(&blocks[3], MarkdownBlock::BulletItem { depth: 0, .. }));
        match &blocks[4] {
            MarkdownBlock::CodeBlock { lang, code } => {
                assert_eq!(lang.as_deref(), Some("rust"));
                assert!(code.contains("fn main()"));
            }
            other => panic!("Expected CodeBlock, got {:?}", other),
        }
        assert!(matches!(&blocks[5], MarkdownBlock::NumberedItem { num, .. } if num == "1"));
        assert!(matches!(&blocks[6], MarkdownBlock::NumberedItem { num, .. } if num == "2"));
        assert!(matches!(&blocks[7], MarkdownBlock::Paragraph(s) if InlineSpan::to_plain_string(s).contains("All systems")));
    }

    #[test]
    fn test_parse_markdown_blocks_quota_error() {
        let err = r#"Agent error: Gemini api error: 429 Too Many Requests: { "error": { "code": 429, "message": "Resource has been exhausted (e.g. check quota).", "status": "RESOURCE_EXHAUSTED", "details": [ { "@type": "type.googleapis.com/google.rpc.QuotaFailure", "violations": [ { "subject": "project:12345", "description": "GenerateRequestsPerMinutePerProjectPerModel-FreeTier" } ] } ] } }"#;
        let blocks = parse_markdown_blocks_impl(err);
        assert_eq!(blocks.len(), 1);
    }

    #[test]
    fn test_inline_bold_italic() {
        let md = "Normal **bold** and *italic* and ***bold-italic*** and `code`";
        let blocks = parse_markdown_blocks_impl(md);
        assert_eq!(blocks.len(), 1);
        if let MarkdownBlock::Paragraph(spans) = &blocks[0] {
            let has_bold  = spans.iter().any(|s| matches!(s, InlineSpan::Bold(_)));
            let has_italic = spans.iter().any(|s| matches!(s, InlineSpan::Italic(_)));
            let has_bi    = spans.iter().any(|s| matches!(s, InlineSpan::BoldItalic(_)));
            let has_code  = spans.iter().any(|s| matches!(s, InlineSpan::Code(_)));
            assert!(has_bold,   "missing Bold span; got {:?}", spans);
            assert!(has_italic, "missing Italic span; got {:?}", spans);
            assert!(has_bi,     "missing BoldItalic span; got {:?}", spans);
            assert!(has_code,   "missing Code span; got {:?}", spans);
        } else {
            panic!("Expected Paragraph, got {:?}", blocks);
        }
    }

    #[test]
    fn test_table_parsing() {
        let md = "| Name | Value |\n|------|-------|\n| Foo  | 42    |\n| Bar  | 99    |";
        let blocks = parse_markdown_blocks_impl(md);
        assert_eq!(blocks.len(), 1, "got: {:#?}", blocks);
        match &blocks[0] {
            MarkdownBlock::Table { headers, rows, .. } => {
                assert_eq!(headers.len(), 2);
                assert_eq!(rows.len(), 2);
                assert!(InlineSpan::to_plain_string(&headers[0]).contains("Name"));
                assert!(InlineSpan::to_plain_string(&rows[0][1]).contains("42"));
            }
            other => panic!("Expected Table, got {:?}", other),
        }
    }

    #[test]
    fn test_horizontal_rule() {
        let md = "Before\n\n---\n\nAfter";
        let blocks = parse_markdown_blocks_impl(md);
        assert!(blocks.iter().any(|b| b == &MarkdownBlock::Rule), "got: {:#?}", blocks);
    }

    #[test]
    fn test_task_list() {
        let md = "- [x] Done\n- [ ] Todo";
        let blocks = parse_markdown_blocks_impl(md);
        assert_eq!(blocks.len(), 2, "got: {:#?}", blocks);
        assert!(matches!(&blocks[0], MarkdownBlock::BulletItem { checked: Some(true), .. }));
        assert!(matches!(&blocks[1], MarkdownBlock::BulletItem { checked: Some(false), .. }));
    }
}
