use crate::state::{ZenthreeState, AiModel, AiProvider, AgentMode, SelectedAgent, AntigravityMode, TrajectoryStep, ExplorationItem, path_matches, SessionSummary, AiSettingsTab, AiMessageBg};
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon, use_context, Placement, PopoverAlign, Wrap, Style, TextDecoration};
use crate::theme::ThemeColors;

const AGENT_MODE_BTN_ID: Id = Id::from_u64(999999885);
const MODEL_PICKER_BTN_ID: Id = Id::from_u64(999999886);
const AI_MORE_BTN_ID: Id = Id::from_u64(999999887);

const SPINNER_FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn get_thinking_spinner_frame() -> &'static str {
    let elapsed_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    SPINNER_FRAMES[(elapsed_ms / 80 % SPINNER_FRAMES.len() as u128) as usize]
}

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
                        .interactive(true)
                        .width(btn_w)
                        .height(btn_h)
                        .radius_all(4.0)
                        .bg(if is_expanded { colors.highlight } else { Color::TRANSPARENT })
                        .hover(Style::bg(colors.highlight))
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
                            let (dot_color, title_text, badge_text) = match state.selected_agent {
                                SelectedAgent::Zene => (colors.accent, "ZENE", None),
                                SelectedAgent::Antigravity => {
                                    let is_connected = agent::external_provider::AntigravityAuthManager::with_default_config()
                                        .load_token()
                                        .is_some();
                                    let dot = if is_connected {
                                        Color::rgb(0.2, 0.85, 0.4) // green
                                    } else {
                                        Color::rgb(0.9, 0.6, 0.2) // orange
                                    };
                                    let badge = if is_connected { "ACP" } else { "Sign In" };
                                    (dot, "Antigravity", Some(badge))
                                }
                            };

                            ui.container()
                                .width(7.0)
                                .height(7.0)
                                .radius_all(3.5)
                                .bg(dot_color)
                                .show(|_| {});

                            ui.text(title_text)
                                .size(12.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();

                            if let Some(badge) = badge_text {
                                ui.container()
                                    .padding(1.0, 4.0, 1.0, 4.0)
                                    .radius_all(3.0)
                                    .bg(colors.highlight)
                                    .show(|ui| {
                                        ui.text(badge)
                                            .size(8.5)
                                            .font_weight(FontWeight::Medium)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                            }
                        });
                });

            // Right Actions (New, History, 3-dot Menu, Close)
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(3.0)
                .show(|ui| {
                    let new_btn = ui.container()
                        .interactive(true)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(4.0)
                        .hover(Style::bg(colors.highlight))
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
                        .interactive(true)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(4.0)
                        .bg(if state.ai_sessions_visible { colors.highlight } else { Color::TRANSPARENT })
                        .hover(Style::bg(colors.highlight))
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
                        .interactive(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .bg(if is_orb_active { colors.highlight } else { Color::TRANSPARENT })
                        .hover(Style::bg(colors.highlight))
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
                        .interactive(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .bg(if state.ai_menu_open { colors.highlight } else { Color::TRANSPARENT })
                        .hover(Style::bg(colors.highlight))
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
                        ui.consume_click();
                        state.ai_menu_open = !state.ai_menu_open;
                        ui.needs_redraw = true;
                    }

                    let close_btn = ui.container()
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .hover(Style::bg(colors.highlight))
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

fn draw_antigravity_signin_card(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: &ThemeColors,
    chat_w: f32,
) {
    ui.container()
        .fill_x()
        .padding_top(24.0)
        .column()
        .halign(Align::Center)
        .gap(12.0)
        .show(|ui| {
            // Antigravity Pill Icon
            ui.container()
                .width(44.0)
                .height(44.0)
                .radius_all(22.0)
                .bg(colors.accent.with_alpha(0.14))
                .border((colors.accent.with_alpha(0.35), 1.0))
                .halign(Align::Center)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("\u{e7f0}")
                        .size(18.0)
                        .color(colors.accent)
                        .show();
                });

            ui.container()
                .column()
                .halign(Align::Center)
                .gap(3.0)
                .show(|ui| {
                    ui.text("Google Antigravity")
                        .size(14.0)
                        .font_weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();

                    ui.text("Autonomous Coding Agent (ACP)")
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });

            // Feature Box
            let box_w = (chat_w - 32.0).clamp(200.0, 320.0);
            ui.container()
                .width(box_w)
                .padding(10.0, 12.0, 10.0, 12.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .border((colors.border.with_alpha(0.6), 1.0))
                .column()
                .gap(6.0)
                .show(|ui| {
                    let bullets = [
                        "Direct local Google OAuth authentication",
                        "Access to Gemini 3.8 Flash & Pro models",
                        "Client-delegated filesystem & terminal tools",
                    ];
                    for b in bullets {
                        ui.container().row().gap(6.0).valign(Align::Center).show(|ui| {
                            ui.text("✓")
                                .size(9.0)
                                .color(colors.accent)
                                .show();
                            ui.text(b)
                                .size(9.5)
                                .color(colors.text_muted)
                                .show();
                        });
                    }
                });

            // Sign In Button
            let is_authenticating = state.antigravity_signing_in;
            let btn_text = if is_authenticating {
                "Waiting for browser..."
            } else {
                "Sign In via Antigravity"
            };

            let signin_btn = ui.container()
                .interactive(!is_authenticating)
                .width(box_w)
                .height(30.0)
                .radius_all(6.0)
                .bg(colors.accent)
                .hover(Style::bg(colors.accent.with_alpha(0.85)))
                .halign(Align::Center)
                .valign(Align::Center)
                .row()
                .gap(6.0)
                .show(|ui| {
                    if is_authenticating {
                        ui.text(get_thinking_spinner_frame())
                            .size(10.0)
                            .color(Color::WHITE)
                            .show();
                    } else {
                        ui.text("\u{e7f0}")
                            .size(10.0)
                            .color(Color::WHITE)
                            .show();
                    }
                    ui.text(btn_text)
                        .size(10.5)
                        .font_weight(FontWeight::SemiBold)
                        .color(Color::WHITE)
                        .show();
                });
            if signin_btn.hovered && !is_authenticating { ui.cursor_icon = CursorIcon::Pointer; }
            if signin_btn.clicked && !is_authenticating {
                state.antigravity_signing_in = true;
                std::thread::spawn(|| {
                    let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
                    let _ = auth.login_interactive();
                });
                state.set_status("Antigravity sign-in initiated in browser...");
                ui.needs_redraw = true;
            }

            // Return to ZENE button
            let back_btn = ui.container()
                .interactive(true)
                .padding(3.0, 8.0, 3.0, 8.0)
                .radius_all(4.0)
                .hover(Style::bg(colors.highlight))
                .row()
                .gap(4.0)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("⟵ Switch back to ZENE")
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                });
            if back_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
            if back_btn.clicked {
                state.selected_agent = SelectedAgent::Zene;
                ui.needs_redraw = true;
            }
        });
}

fn draw_antigravity_connected_welcome(
    ui: &mut Ui,
    _state: &mut ZenthreeState,
    colors: &ThemeColors,
) {
    ui.container()
        .fill_x()
        .padding_top(40.0)
        .column()
        .halign(Align::Center)
        .gap(6.0)
        .show(|ui| {
            // Subtle status dot + Antigravity
            ui.container()
                .row()
                .halign(Align::Center)
                .valign(Align::Center)
                .gap(5.0)
                .show(|ui| {
                    ui.text("●")
                        .size(6.0)
                        .color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                        .show();
                    ui.text("Antigravity")
                        .size(10.0)
                        .font_weight(FontWeight::Medium)
                        .color(colors.text_muted)
                        .show();
                });

            ui.text("How can I help you today?")
                .size(13.0)
                .font_weight(FontWeight::Medium)
                .color(colors.text_primary)
                .show();
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
        let menu_w = 180.0;
        let menu_h = 142.0;

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
            .border((colors.border.with_alpha(0.8), 1.0))
            .radius_all(6.0)
            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.5), 0.0, 4.0, 16.0)
            .padding(3.0, 3.0, 3.0, 3.0);

        if state.glassmorphism {
            popover_builder = popover_builder.backdrop_blur(state.blur_strength.max(4.0));
        }

        let popover_resp = popover_builder.show(|ui| {
                ui.container()
                    .fill_x()
                    .column()
                    .gap(1.0)
                    .show(|ui| {
                        // ── SECTION 1: NATIVE AGENT ──
                        ui.container().padding(3.0, 6.0, 2.0, 6.0).show(|ui| {
                            ui.text("NATIVE AGENT")
                                .size(8.5)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_muted)
                                .show();
                        });

                        let is_zene_active = state.selected_agent == SelectedAgent::Zene;
                        let zene_btn = ui.container()
                            .fill_x()
                            .padding(4.0, 6.0, 4.0, 6.0)
                            .radius_all(3.5)
                            .bg(if is_zene_active { colors.highlight } else { Color::TRANSPARENT })
                            .hover(Style::bg(colors.highlight))
                            .interactive(true)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                                    ui.text("✦")
                                        .size(10.0)
                                        .color(colors.accent)
                                        .show();
                                    ui.text("ZENE")
                                        .size(10.5)
                                        .font_weight(if is_zene_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                        .color(colors.text_primary)
                                        .show();
                                });
                                if is_zene_active {
                                    ui.text("✓")
                                        .size(9.5)
                                        .color(colors.accent)
                                        .show();
                                }
                            });
                        if zene_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                        if zene_btn.clicked {
                            ui.clicked = false;
                            state.selected_agent = SelectedAgent::Zene;
                            state.ai_menu_open = false;
                            ui.needs_redraw = true;
                        }

                        // Divider
                        ui.container().fill_x().padding(2.0, 4.0, 2.0, 4.0).show(|ui| {
                            ui.container().fill_x().height(1.0).bg(colors.border.with_alpha(0.4)).show(|_| {});
                        });

                        // ── SECTION 2: EXTERNAL AGENTS ──
                        ui.container().padding(2.0, 6.0, 2.0, 6.0).show(|ui| {
                            ui.text("EXTERNAL AGENTS")
                                .size(8.5)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_muted)
                                .show();
                        });

                        let is_ag_active = state.selected_agent == SelectedAgent::Antigravity;
                        let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
                        let is_ag_connected = auth.load_token().is_some();

                        let ag_btn = ui.container()
                            .fill_x()
                            .padding(4.0, 6.0, 4.0, 6.0)
                            .radius_all(3.5)
                            .bg(if is_ag_active { colors.highlight } else { Color::TRANSPARENT })
                            .hover(Style::bg(colors.highlight))
                            .interactive(true)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                                    ui.text("\u{e7f0}")
                                        .size(10.0)
                                        .color(colors.accent)
                                        .show();
                                    ui.text("Google Antigravity")
                                        .size(10.5)
                                        .font_weight(if is_ag_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                        .color(colors.text_primary)
                                        .show();
                                });
                                if is_ag_active {
                                    ui.text("✓")
                                        .size(9.5)
                                        .color(colors.accent)
                                        .show();
                                } else if is_ag_connected {
                                    ui.text("●")
                                        .size(7.0)
                                        .color(Color::rgb(0.2, 0.85, 0.4))
                                        .show();
                                } else {
                                    ui.text("Sign In")
                                        .size(8.0)
                                        .color(colors.text_muted)
                                        .show();
                                }
                            });
                        if ag_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                        if ag_btn.clicked {
                            ui.clicked = false;
                            state.selected_agent = SelectedAgent::Antigravity;
                            state.ai_menu_open = false;
                            ui.needs_redraw = true;
                        }

                        // Divider
                        ui.container().fill_x().padding(2.0, 4.0, 2.0, 4.0).show(|ui| {
                            ui.container().fill_x().height(1.0).bg(colors.border.with_alpha(0.4)).show(|_| {});
                        });

                        // ── SECTION 3: SETTINGS ──
                        let set_btn = ui.container()
                            .fill_x()
                            .padding(4.0, 6.0, 4.0, 6.0)
                            .radius_all(3.5)
                            .hover(Style::bg(colors.highlight))
                            .interactive(true)
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text("\u{f013}") // gear icon
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                                ui.text("AI & Agent Settings...")
                                    .size(10.5)
                                    .color(colors.text_primary)
                                    .show();
                            });
                        if set_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                        if set_btn.clicked {
                            ui.clicked = false;
                            state.ai_menu_open = false;
                            state.ai_settings.open = true;
                            state.ai_settings.just_opened = true;
                            state.ai_settings.active_tab = AiSettingsTab::ExternalAgents;
                            ui.needs_redraw = true;
                        }
                    });
            });

        if ui.clicked && !popover_resp.hovered {
            state.ai_menu_open = false;
            ui.needs_redraw = true;
        }
    }

    // ─── Floating Menu Panel: Agent Mode ─────────────────────────────────
    if state.agent_mode_menu_open {
        let menu_w = 138.0;
        let num_modes = match state.selected_agent {
            SelectedAgent::Zene => AgentMode::ALL.len(),
            SelectedAgent::Antigravity => AntigravityMode::ALL.len(),
        };
        let menu_h = num_modes as f32 * 27.5 + 8.0;

        let mut changed_mode = false;
        let popover_resp = ui.popover(AGENT_MODE_BTN_ID)
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
            .border(((if colors.bg_base.r > 0.5 {
                colors.border
            } else {
                Color::rgb(45.0 / 255.0, 45.0 / 255.0, 54.0 / 255.0)
            }), 1.0))
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
                    .raw_id(Id::from_u64(0xF00D_1102))
                    .fill_x()
                    .column()
                    .gap(1.5)
                    .show(|ui| {
                        match state.selected_agent {
                            SelectedAgent::Zene => {
                                for mode in AgentMode::ALL {
                                    let is_sel = state.agent_mode == mode;
                                    let row_id = ui.id_from(format!("pop_agent_mode_{:?}", mode));
                                    let row = ui.container()
                                        .raw_id(row_id)
                                        .interactive(true)
                                        .fill_x()
                                        .height(26.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .radius_all(3.5)
                                        .bg(if is_sel { colors.accent.with_alpha(0.12) } else { Color::TRANSPARENT })
                                        .hover(Style::bg(colors.highlight))
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
                                                    .font_weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                                    .color(if is_sel { colors.accent } else { colors.text_primary })
                                                    .show();
                                            });
                                            if is_sel {
                                                ui.text("\u{f00c}").size(8.5).color(colors.accent).show();
                                            }
                                        });
                                    if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if row.clicked || (row.hovered && ui.clicked) {
                                        ui.consume_click();
                                        state.agent_mode = mode;
                                        state.agent_mode_menu_open = false;
                                        changed_mode = true;
                                        ui.needs_redraw = true;
                                    }
                                }
                            }
                            SelectedAgent::Antigravity => {
                                for mode in AntigravityMode::ALL {
                                    let is_sel = state.antigravity_mode == mode;
                                    let row_id = ui.id_from(format!("pop_ag_mode_{:?}", mode));
                                    let row = ui.container()
                                        .raw_id(row_id)
                                        .interactive(true)
                                        .fill_x()
                                        .height(26.0)
                                        .padding(2.0, 6.0, 2.0, 6.0)
                                        .radius_all(3.5)
                                        .bg(if is_sel { colors.accent.with_alpha(0.12) } else { Color::TRANSPARENT })
                                        .hover(Style::bg(colors.highlight))
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
                                                    .font_weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                                    .color(if is_sel { colors.accent } else { colors.text_primary })
                                                    .show();
                                            });
                                            if is_sel {
                                                ui.text("\u{f00c}").size(8.5).color(colors.accent).show();
                                            }
                                        });
                                    if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if row.clicked || (row.hovered && ui.clicked) {
                                        ui.consume_click();
                                        state.antigravity_mode = mode;
                                        state.agent_mode_menu_open = false;
                                        changed_mode = true;
                                        ui.needs_redraw = true;
                                    }
                                }
                            }
                        }
                    });
            });

        if !changed_mode && ui.clicked && !popover_resp.hovered {
            state.agent_mode_menu_open = false;
            ui.needs_redraw = true;
        }
    }

    // ─── Floating Menu Panel: Model Selector ──────────────────────────────
    draw_ai_model_popover(ui, state, &colors);
}

fn draw_prompt_model_button(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: &crate::theme::ThemeColors,
    pill_bg: Color,
    pill_hover_bg: Color,
    pill_border: Color,
) {
    // Keep agent selection aligned with selected model
    if state.selected_ai_model.is_antigravity() {
        state.selected_agent = SelectedAgent::Antigravity;
    } else {
        state.selected_agent = SelectedAgent::Zene;
    }

    let model_btn = ui.container()
        .raw_id(MODEL_PICKER_BTN_ID)
        .interactive(true)
        .padding(3.5, 8.0, 3.5, 8.0)
        .radius_all(6.0)
        .bg(if state.ai_model_menu_open { pill_hover_bg } else { pill_bg })
        .hover(Style::bg(pill_hover_bg))
        .border((pill_border, 1.0))
        .row()
        .valign(Align::Center)
        .gap(5.0)
        .show(|ui| {
            ui.text(state.selected_ai_model.display_name())
                .size(10.0)
                .font_weight(FontWeight::Medium)
                .color(colors.text_primary)
                .show();
            ui.text(if state.ai_model_menu_open { "\u{f077}" } else { "\u{f078}" })
                .size(7.0)
                .color(colors.text_muted)
                .show();
        });

    if model_btn.hovered {
        ui.cursor_icon = CursorIcon::Pointer;
    }
    if model_btn.clicked {
        ui.clicked = false;
        state.ai_model_menu_open = !state.ai_model_menu_open;
        if state.ai_model_menu_open {
            state.agent_mode_menu_open = false;
        }
        ui.needs_redraw = true;
    }
}

fn draw_ai_model_popover(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: &crate::theme::ThemeColors,
) {
    if !state.ai_model_menu_open {
        return;
    }

    let models_to_show: Vec<AiModel> = vec![
        // Groq
        AiModel::GroqGptOss120b,
        AiModel::GroqGptOss20b,
        AiModel::GroqQwen3_8_27b,
        // Google Gemini
        AiModel::Gemini35Flash,
        AiModel::Gemini35FlashLite,
        AiModel::Gemini25Flash,
        AiModel::Gemini25Pro,
        // Google Antigravity
        AiModel::AntigravityGemini38FlashHigh,
        AiModel::AntigravityGemini38FlashMedium,
        AiModel::AntigravityGemini37FlashHigh,
        AiModel::AntigravityGeminiProAgent,
    ];

    // Menu width & height
    let menu_w = 236.0;
    let total_item_h = models_to_show.len() as f32 * 28.0 + 85.0;
    let max_allowed_h = (ui.height - 110.0).max(200.0);
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

    let mut select_model: Option<AiModel> = None;
    let popover_resp = ui.popover(MODEL_PICKER_BTN_ID)
        .placement(Placement::Top)
        .align(PopoverAlign::Start)
        .gap(3.0)
        .auto_flip(false)
        .width(menu_w)
        .height(menu_h)
        .bg(menu_bg)
        .border((menu_border, 1.0))
        .radius_all(6.0)
        .shadow(menu_shadow, 0.0, 4.0, 16.0)
        .padding(3.0, 3.0, 3.0, 3.0)
        .show(|ui| {
            ui.container()
                .raw_id(Id::from_u64(0xF00D_1101))
                .fill_x()
                .height(menu_h - 6.0)
                .column()
                .gap(1.2)
                .scroll_y(true)
                .show(|ui| {
                    let mut last_provider: Option<AiProvider> = None;
                    for model in models_to_show {
                        if last_provider != Some(model.provider()) {
                            let label = match model.provider() {
                                AiProvider::Antigravity => "GOOGLE ANTIGRAVITY (EXTERNAL AGENT)",
                                AiProvider::Groq => "GROQ",
                                AiProvider::Gemini => "GEMINI",
                            };
                            if last_provider.is_some() {
                                ui.container().height(3.0).show(|_| {});
                            }
                            ui.container().padding(3.0, 6.0, 1.0, 6.0).show(|ui| {
                                ui.text(label)
                                    .size(7.5)
                                    .font_weight(FontWeight::Bold)
                                    .color(colors.text_muted.with_alpha(0.65))
                                    .show();
                            });
                            last_provider = Some(model.provider());
                        }

                        let is_selected = state.selected_ai_model == model;
                        let row_id = ui.id_from(format!("pop_model_opt_{}", model.model_id()));
                        let row = ui.container()
                            .raw_id(row_id)
                            .interactive(true)
                            .fill_x()
                            .height(26.0)
                            .padding(2.0, 8.0, 2.0, 8.0)
                            .radius_all(4.0)
                            .bg(if is_selected { colors.accent.with_alpha(0.14) } else { Color::TRANSPARENT })
                            .hover(Style::bg(colors.highlight))
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.text(model.display_name())
                                    .size(9.5)
                                    .font_weight(if is_selected { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_selected { colors.accent } else { colors.text_primary })
                                    .show();

                                if is_selected {
                                    ui.text("\u{f00c}") // Checkmark icon
                                        .size(8.5)
                                        .color(colors.accent)
                                        .show();
                                }
                            });

                        if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                        if row.clicked || (row.hovered && ui.clicked) {
                            ui.consume_click();
                            select_model = Some(model);
                        }
                    }
                });
        });

    if let Some(model) = select_model {
        state.selected_ai_model = model;
        if model.is_antigravity() {
            state.selected_agent = SelectedAgent::Antigravity;
        } else {
            state.selected_agent = SelectedAgent::Zene;
        }
        state.ai_model_menu_open = false;
        state.ai_session_json = None;
        state.save_persistent_state();
        ui.consume_click();
        ui.needs_redraw = true;
    } else if ui.clicked && !popover_resp.hovered {
        // Outside click dismissed the popover
        state.ai_model_menu_open = false;
        ui.needs_redraw = true;
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
    const MAX_INPUT_W: f32 = 720.0;
    const MAX_CHAT_CONTENT_W: f32 = 900.0;
    let input_dock_w = (chat_w - 24.0).min(MAX_INPUT_W).max(120.0);
    let chat_content_w = (chat_w - 24.0).min(MAX_CHAT_CONTENT_W).max(120.0);

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
                    if added == 0 && rev.is_new_file && !rev.proposed_content.is_empty() {
                        added = rev.proposed_content.lines().count().max(1);
                    }
                    upsert_file(rev.path.clone(), fname, rel, added, removed);
                }

                // 2. From tabs that have unaccepted changes
                for tab in &state.tabs {
                    if tab.is_dirty && (tab.lines_added > 0 || tab.lines_removed > 0) {
                        if let Some(ref tp) = tab.path {
                            let fname = tab.name.clone();
                            let rel = tp.strip_prefix(&state.current_dir).unwrap_or(tp).to_string_lossy().to_string();
                            upsert_file(tp.clone(), fname, rel, tab.lines_added, tab.lines_removed);
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

            let has_active_terminal_tasks = !state.active_agent_terminal_tasks.is_empty();

            let modified_section_h = if state.show_agent_terminal_popup && has_active_terminal_tasks {
                let term_list_h = (state.active_agent_terminal_tasks.len().max(1) as f32 * 24.0 + 8.0).min(140.0);
                term_list_h + 6.0 + 20.0
            } else if state.show_modified_files_popup && has_modified_files {
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
                .halign(Align::Center)
                .gap(20.0)
                .scroll_y(true)
                .show(|ui| {
                    let is_ag = state.selected_agent == SelectedAgent::Antigravity;
                    let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
                    let is_ag_connected = auth.load_token().is_some();

                    if is_ag && !is_ag_connected {
                        draw_antigravity_signin_card(ui, state, colors, chat_content_w);
                    } else if state.ai_messages.is_empty() {
                        if is_ag {
                            draw_antigravity_connected_welcome(ui, state, colors);
                        } else {
                            // Minimalist Welcome State
                            ui.container()
                                .width(chat_content_w)
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
                                        .border((colors.accent.with_alpha(0.25), 1.0))
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
                                        .font_weight(FontWeight::Medium)
                                        .color(colors.text_primary)
                                        .show();

                                    ui.text("Ask questions, refactor code, or generate tests.")
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                        }
                    } else {
                        let mut toggle_expand_idx: Option<usize> = None;
                        let mut toggle_thoughts_idx: Option<usize> = None;
                        let mut toggle_trajectory_group: Option<(usize, usize)> = None;
                        let mut toggle_exploration_item: Option<(usize, usize, usize)> = None;
                        let mut toggle_file_edit_step: Option<(usize, usize)> = None;
                        let mut copy_target_idx: Option<usize> = None;
                        let mut copy_code_snippet: Option<(u64, String)> = None;
                        let mut chat_open_file: Option<std::path::PathBuf> = None;
                        let mut open_implementation_plan = false;
                        let mut proceed_with_plan = false;

                        let base_text_size = state.ai_settings.effective_text_size().pt();
                        let user_font_sz = (base_text_size + 0.5).max(12.5);
                        let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);

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
                                let bubble_max_w = (chat_content_w - 48.0).min(MAX_INPUT_W).max(80.0);
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
                                    .width(chat_content_w)
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
                                                .padding(6.0, 14.0, 6.0, 14.0)
                                                .bg(user_bg)
                                                .hover(Style::bg(user_hover_bg))
                                                .border((user_border, 1.0))
                                                .radius_all(14.0)
                                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.35), 0.0, 2.0, 6.0)
                                                .row()
                                                .valign(Align::Center)
                                                .gap(8.0)
                                                .show(|ui| {
                                                    ui.text(display_text)
                                                        .size(user_font_sz)
                                                        .decoration(TextDecoration::LineHeight(1.35))
                                                        .color(user_text_col)
                                                        .show();

                                                    let copy_btn = ui.container()
                                                        .id(copy_btn_id)
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .radius_all(4.0)
                                                        .hover(Style::bg(user_hover_bg))
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
                                                .padding(8.0, 14.0, 8.0, 14.0)
                                                .bg(user_bg)
                                                .hover(Style::bg(user_hover_bg))
                                                .border((user_border, 1.0))
                                                .radius_all(14.0)
                                                .shadow(Color::rgba(0.0, 0.0, 0.0, 0.35), 0.0, 2.0, 6.0)
                                                .column()
                                                .halign(Align::Right)
                                                .gap(4.0)
                                                .show(|ui| {
                                                    ui.text(display_text)
                                                        .size(user_font_sz)
                                                        .decoration(TextDecoration::LineHeight(1.4))
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
                                                                .hover(Style::bg(user_hover_bg))
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
                                // ─── Assistant Message (clean container, capped width) ───
                                let ai_text_w = (chat_content_w - 16.0).max(60.0);
                                let code_box_w = ai_text_w.max(50.0);

                                ui.container()
                                    .width(chat_content_w)
                                    .id(bubble_id)
                                    .padding(4.0, 4.0, 8.0, 4.0)
                                    .column()
                                    .gap(14.0)
                                    .show(|ui| {
                                        // ── Sibling 1: Trajectory Steps ─────────────────────
                                        for (step_idx, step) in msg.trajectory.iter().enumerate() {
                                            let step_id = Id::from_u64(999_200_000 + (msg_idx as u64) * 1000 + (step_idx as u64));
                                            match step {
                                                TrajectoryStep::Exploration(grp) => {
                                                    let chevron = if grp.expanded { "\u{f078}" } else { "\u{f054}" };
                                                    let summary = grp.summary_label();

                                                    let exp_card = ui.container()
                                                        .fill_x()
                                                        .column()
                                                        .gap(4.0);

                                                    exp_card.show(|ui| {
                                                        let header_row = ui.container()
                                                            .id(step_id)
                                                            .fill_x()
                                                            .padding(2.0, 4.0, 2.0, 4.0)
                                                            .radius_all(4.0)
                                                            .hover(Style::bg(colors.highlight))
                                                            .row()
                                                            .valign(Align::Center)
                                                            .gap(6.0)
                                                            .show(|ui| {
                                                                if grp.is_active {
                                                                    ui.request_redraw_after(std::time::Duration::from_millis(80));
                                                                    ui.container()
                                                                        .width(6.0)
                                                                        .height(6.0)
                                                                        .radius_all(3.0)
                                                                        .bg(colors.accent)
                                                                        .shadow(colors.accent.with_alpha(0.8), 0.0, 0.0, 6.0)
                                                                        .show(|_| {});
                                                                }
                                                                ui.text(&summary)
                                                                    .size(12.0)
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
                                                                .fill_x()
                                                                .padding_left(16.0)
                                                                .column()
                                                                .gap(3.0)
                                                                .show(|ui| {
                                                                for (item_idx, item) in grp.items.iter().enumerate() {
                                                                    let is_item_running = grp.is_active && (item_idx + 1 == grp.items.len());
                                                                    if is_item_running {
                                                                        ui.request_redraw_after(std::time::Duration::from_millis(80));
                                                                    }
                                                                    match item {
                                                                        ExplorationItem::AnalyzedFile { path, start_line, end_line, symbol, is_peek, output, expanded } => {
                                                                            let path_obj = std::path::Path::new(path);
                                                                            let resolved_path = if path_obj.is_absolute() {
                                                                                path_obj.to_path_buf()
                                                                            } else {
                                                                                state.current_dir.join(path_obj)
                                                                            };
                                                                            let is_dir = resolved_path.is_dir();
                                                                            let file_exists = resolved_path.is_file() || is_dir;

                                                                            let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());
                                                                            let file_icon = if is_dir {
                                                                                Some(icon_theme.get_icon_for_folder(fname, *expanded))
                                                                            } else if file_exists {
                                                                                Some(icon_theme.get_icon_for_file(fname, Some(&resolved_path)))
                                                                            } else {
                                                                                None
                                                                            };

                                                                            let (action_verb, verb_color, badge) = if is_dir {
                                                                                (if is_item_running { "Exploring" } else { "Explored" }, colors.text_muted, String::new())
                                                                            } else if !file_exists {
                                                                                (
                                                                                    "Not found",
                                                                                    Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 0.9 },
                                                                                    " [not found]".to_string(),
                                                                                )
                                                                            } else {
                                                                                let verb = match (*is_peek, is_item_running) {
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
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(if file_exists { colors.highlight } else { colors.highlight.with_alpha(0.3) }))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(verb_color)
                                                                                        .show();
                                                                                    if let Some(ref ic) = file_icon {
                                                                                        ic.show(ui, 12.0);
                                                                                    } else {
                                                                                        ui.text("\u{f00d}")
                                                                                            .size(10.5)
                                                                                            .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 0.9 })
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(&display_path)
                                                                                        .size(11.5)
                                                                                        .color(if file_exists { colors.text_primary } else { colors.text_muted })
                                                                                        .show();

                                                                                    if !badge.is_empty() {
                                                                                        ui.text(&badge)
                                                                                            .size(10.0)
                                                                                            .color(if file_exists { colors.accent } else { Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 0.8 } })
                                                                                            .show();
                                                                                    }

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if file_row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if file_row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let file_card_id = Id::from_u64(999_500_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let (file_meta, file_content) = if let Some(out) = output.as_deref() {
                                                                                    (format!("{} (streamed)", fname), out.to_string())
                                                                                } else if let Ok(disk) = std::fs::read_to_string(&resolved_path) {
                                                                                    let total_lines = disk.lines().count();
                                                                                    let bytes = disk.len();
                                                                                    let size_str = if bytes < 1024 { format!("{} B", bytes) } else { format!("{:.1} KB", bytes as f32 / 1024.0) };
                                                                                    let meta = match (start_line, end_line) {
                                                                                        (Some(s), Some(e)) => format!("{} lines (L{}-L{}) • {}", e.saturating_sub(*s) + 1, s, e, size_str),
                                                                                        (Some(s), None) => format!("from line {} • {}", s, size_str),
                                                                                        _ => format!("{} lines • {}", total_lines, size_str),
                                                                                    };
                                                                                    let slice = match (start_line, end_line) {
                                                                                        (Some(s), Some(e)) => disk.lines().skip(s.saturating_sub(1)).take(e.saturating_sub(*s) + 1).collect::<Vec<_>>().join("\n"),
                                                                                        (Some(s), None) => disk.lines().skip(s.saturating_sub(1)).collect::<Vec<_>>().join("\n"),
                                                                                        _ => disk,
                                                                                    };
                                                                                    (meta, slice)
                                                                                } else {
                                                                                    ("File".to_string(), "File not found or cannot be read.".to_string())
                                                                                };

                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    file_card_id,
                                                                                    file_icon.as_ref(),
                                                                                    None,
                                                                                    fname,
                                                                                    &file_meta,
                                                                                    &file_content,
                                                                                    &mut copy_code_snippet,
                                                                                    if file_exists { Some((&mut chat_open_file, resolved_path.clone())) } else { None },
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::BatchRead { paths, output, expanded } => {
                                                                            let action_verb = if is_item_running { "Reading batch" } else { "Read batch" };
                                                                            let list_str = paths
                                                                                .iter()
                                                                                .filter_map(|p| std::path::Path::new(p).file_name().and_then(|s| s.to_str()))
                                                                                .collect::<Vec<_>>()
                                                                                .join(", ");
                                                                            let item_id = Id::from_u64(999_310_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text("\u{f15b}")
                                                                                        .size(10.5)
                                                                                        .color(colors.accent)
                                                                                        .show();
                                                                                    ui.text(&format!("[{}]", list_str))
                                                                                        .size(11.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let batch_card_id = Id::from_u64(999_800_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let content = output.as_deref().unwrap_or_else(|| list_str.as_str());
                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    batch_card_id,
                                                                                    None,
                                                                                    Some(("\u{f15b}", colors.accent)),
                                                                                    "Batch Read Files",
                                                                                    &format!("{} files", paths.len()),
                                                                                    content,
                                                                                    &mut copy_code_snippet,
                                                                                    None,
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::Searched { query, scope, output, expanded } => {
                                                                            let search_prefix = if is_item_running { "Searching" } else { "Searched" };
                                                                            let action_verb = if let Some(ref sc) = scope {
                                                                                format!("{} {}", search_prefix, sc)
                                                                            } else {
                                                                                search_prefix.to_string()
                                                                            };
                                                                            let item_id = Id::from_u64(999_320_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(&action_verb)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text("\u{f002}")
                                                                                        .size(10.5)
                                                                                        .color(colors.accent)
                                                                                        .show();
                                                                                    ui.text(&format!("\"{}\"", query))
                                                                                        .size(11.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let search_card_id = Id::from_u64(999_700_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let res_text = output.as_deref().unwrap_or("Search completed.");
                                                                                let header_desc = if let Some(ref sc) = scope { format!("in {}", sc) } else { String::new() };
                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    search_card_id,
                                                                                    None,
                                                                                    Some(("\u{f002}", colors.accent)),
                                                                                    &format!("Search: \"{}\"", query),
                                                                                    &header_desc,
                                                                                    res_text,
                                                                                    &mut copy_code_snippet,
                                                                                    None,
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::ExploredDirectory { path, output, expanded } => {
                                                                            let action_verb = if is_item_running { "Exploring" } else { "Explored" };
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
                                                                            let display_dir_formatted = if display_dir.ends_with('/') {
                                                                                display_dir.clone()
                                                                            } else {
                                                                                format!("{}/", display_dir)
                                                                            };
                                                                            let folder_icon = icon_theme.get_icon_for_folder(&display_dir, *expanded);

                                                                            let item_id = Id::from_u64(999_330_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    folder_icon.show(ui, 12.0);
                                                                                    ui.text(&display_dir_formatted)
                                                                                        .size(11.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let dir_card_id = Id::from_u64(999_400_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let (summary_text, entries_text) = {
                                                                                    let looks_like_listing = output.as_deref().map_or(false, |s| {
                                                                                        let trimmed = s.trim();
                                                                                        !trimmed.is_empty() && !trimmed.starts_with("List ") && trimmed.lines().count() > 1
                                                                                    });
                                                                                    if looks_like_listing {
                                                                                        ("Results".to_string(), output.as_deref().unwrap().to_string())
                                                                                    } else if resolved.is_dir() {
                                                                                        read_local_dir_summary(&resolved)
                                                                                    } else if let Some(out) = output.as_deref().filter(|s| !s.trim().is_empty() && !s.trim().starts_with("List ")) {
                                                                                        ("Results".to_string(), out.to_string())
                                                                                    } else {
                                                                                        read_local_dir_summary(&resolved)
                                                                                    }
                                                                                };

                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    dir_card_id,
                                                                                    Some(&folder_icon),
                                                                                    None,
                                                                                    &display_dir_formatted,
                                                                                    &summary_text,
                                                                                    &entries_text,
                                                                                    &mut copy_code_snippet,
                                                                                    None,
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::ActivatedSkill { name, output, expanded } => {
                                                                            let action_verb = if is_item_running { "Activating skill" } else { "Activated skill" };
                                                                            let item_id = Id::from_u64(999_340_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text("\u{f0e7}")
                                                                                        .size(10.5)
                                                                                        .color(colors.accent)
                                                                                        .show();
                                                                                    ui.text(name)
                                                                                        .size(11.5)
                                                                                        .color(colors.accent)
                                                                                        .show();

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let skill_card_id = Id::from_u64(999_850_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let content = output.as_deref().unwrap_or("Skill instructions loaded into agent context.");
                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    skill_card_id,
                                                                                    None,
                                                                                    Some(("\u{f0e7}", colors.accent)),
                                                                                    &format!("Skill: {}", name),
                                                                                    "Active",
                                                                                    content,
                                                                                    &mut copy_code_snippet,
                                                                                    None,
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::ExecutedCommand { command, output, expanded } => {
                                                                            let action_verb = if is_item_running { "Running" } else { "Ran" };
                                                                            let item_id = Id::from_u64(999_350_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text("\u{f489}")
                                                                                        .size(10.5)
                                                                                        .color(colors.accent)
                                                                                        .show();
                                                                                    ui.text(command)
                                                                                        .size(11.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let cmd_card_id = Id::from_u64(999_600_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let cmd_text = output.as_deref().unwrap_or("Command completed.");
                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    cmd_card_id,
                                                                                    None,
                                                                                    Some(("\u{f489}", colors.accent)),
                                                                                    "Terminal Output",
                                                                                    command,
                                                                                    cmd_text,
                                                                                    &mut copy_code_snippet,
                                                                                    None,
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::CheckedDiagnostics { output, expanded } => {
                                                                            let action_verb = if is_item_running { "Checking diagnostics" } else { "Checked diagnostics" };
                                                                            let item_id = Id::from_u64(999_360_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(colors.text_muted)
                                                                                        .show();
                                                                                    ui.text("\u{f058}")
                                                                                        .size(10.5)
                                                                                        .color(colors.accent)
                                                                                        .show();

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered {
                                                                                ui.cursor_icon = CursorIcon::Pointer;
                                                                            }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let diag_card_id = Id::from_u64(999_900_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                let content = output.as_deref().unwrap_or("0 errors, 0 warnings.");
                                                                                render_tool_output_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    diag_card_id,
                                                                                    None,
                                                                                    Some(("\u{f058}", colors.accent)),
                                                                                    "Diagnostics",
                                                                                    "Compiler / AST",
                                                                                    content,
                                                                                    &mut copy_code_snippet,
                                                                                    None,
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::WrittenFile { path, lines_added, lines_removed, is_new, output, expanded } => {
                                                                            let action_verb = if *is_new { "Created" } else { "Edited" };
                                                                            let action_color = if *is_new { Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 } } else { colors.text_muted };
                                                                            let path_obj = std::path::Path::new(path);
                                                                            let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());

                                                                            let pb = std::path::PathBuf::from(path);
                                                                            let full_p = if pb.is_absolute() { pb } else { state.current_dir.join(&pb) };
                                                                            let display_path = if let Ok(rel) = full_p.strip_prefix(&state.current_dir) {
                                                                                let s = rel.to_string_lossy().to_string();
                                                                                if s.is_empty() || s == "." { fname.to_string() } else { s }
                                                                            } else {
                                                                                fname.to_string()
                                                                            };
                                                                            let file_icon = icon_theme.get_icon_for_file(fname, Some(&full_p));

                                                                            let item_id = Id::from_u64(999_370_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                            let row = ui.container()
                                                                                .id(item_id)
                                                                                .fill_x()
                                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                                .radius_all(3.0)
                                                                                .hover(Style::bg(colors.highlight))
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(5.0)
                                                                                .show(|ui| {
                                                                                    if is_item_running {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(action_verb)
                                                                                        .size(10.5)
                                                                                        .color(action_color)
                                                                                        .show();
                                                                                    file_icon.show(ui, 12.0);
                                                                                    ui.text(&display_path)
                                                                                        .size(11.5)
                                                                                        .color(colors.text_primary)
                                                                                        .show();

                                                                                    if *is_new {
                                                                                        if *lines_added > 0 {
                                                                                            ui.text(&format!("+{}", lines_added))
                                                                                                .size(10.5)
                                                                                                .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                                                .show();
                                                                                        }
                                                                                    } else {
                                                                                        if *lines_added > 0 {
                                                                                            ui.text(&format!("+{}", lines_added))
                                                                                                .size(10.5)
                                                                                                .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                                                .show();
                                                                                        }
                                                                                        if *lines_removed > 0 {
                                                                                            ui.text(&format!("-{}", lines_removed))
                                                                                                .size(10.5)
                                                                                                .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                                                                .show();
                                                                                        }
                                                                                    }

                                                                                    let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                                                    ui.text(chevron_glyph)
                                                                                        .size(7.0)
                                                                                        .color(colors.text_muted.with_alpha(0.6))
                                                                                        .show();
                                                                                });

                                                                            if row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                                                            if row.clicked {
                                                                                toggle_exploration_item = Some((msg_idx, step_idx, item_idx));
                                                                            }

                                                                            if *expanded {
                                                                                let (rev_is_created, preview_lines, total_lines, full_content) = prepare_file_preview(&full_p, &state.pending_reviews, output.as_deref());
                                                                                let final_is_created = *is_new || rev_is_created;
                                                                                let delta_info = if final_is_created {
                                                                                    format!("(+{})", lines_added)
                                                                                } else if *lines_removed > 0 {
                                                                                    format!("(+{}, -{})", lines_added, lines_removed)
                                                                                } else {
                                                                                    format!("(+{})", lines_added)
                                                                                };
                                                                                let file_card_id = Id::from_u64(999_890_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100 + (item_idx as u64));
                                                                                render_write_file_box(
                                                                                    ui,
                                                                                    colors,
                                                                                    file_card_id,
                                                                                    Some(&file_icon),
                                                                                    final_is_created,
                                                                                    &display_path,
                                                                                    &delta_info,
                                                                                    &preview_lines,
                                                                                    total_lines,
                                                                                    &full_content,
                                                                                    &mut copy_code_snippet,
                                                                                    &mut chat_open_file,
                                                                                    full_p.clone(),
                                                                                );
                                                                            }
                                                                        }
                                                                        ExplorationItem::Thought { summary, duration_secs } => {
                                                                            let is_spinning = is_item_running || (grp.is_active && duration_secs.is_none());
                                                                            let thought_label = match (duration_secs, is_spinning) {
                                                                                (Some(secs), _) => format!("Thought for {}s", secs),
                                                                                (None, true) => if summary.is_empty() { "Thinking...".to_string() } else { summary.clone() },
                                                                                (None, false) => if summary.is_empty() { "Thought".to_string() } else { summary.clone() },
                                                                            };
                                                                            if is_spinning {
                                                                                ui.request_redraw_after(std::time::Duration::from_millis(80));
                                                                            }
                                                                            ui.container()
                                                                                .fill_x()
                                                                                .row()
                                                                                .valign(Align::Center)
                                                                                .gap(4.0)
                                                                                .show(|ui| {
                                                                                    if is_spinning {
                                                                                        ui.text(get_thinking_spinner_frame())
                                                                                            .size(11.0)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    ui.text(&thought_label)
                                                                                        .size(11.5)
                                                                                        .decoration(TextDecoration::Italic)
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
                                                TrajectoryStep::FileEdit { path, lines_added, lines_removed, is_new, expanded } => {
                                                    let path_obj = std::path::Path::new(path);
                                                    let fname = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or(path.as_str());

                                                    let mut added = *lines_added;
                                                    let mut removed = *lines_removed;
                                                    let pb = std::path::PathBuf::from(path);
                                                    let full_p = if pb.is_absolute() { pb } else { state.current_dir.join(&pb) };
                                                    let file_icon = icon_theme.get_icon_for_file(fname, Some(&full_p));
                                                    let mut is_created = *is_new || if let Some(rev) = state.pending_reviews.iter().find(|r| path_matches(&r.path, &full_p)) {
                                                        rev.is_new_file
                                                    } else {
                                                        !full_p.exists()
                                                    };
                                                    if added == 0 && removed == 0 {
                                                        if let Some(rev) = state.pending_reviews.iter().find(|r| path_matches(&r.path, &full_p)) {
                                                            added = rev.diff_lines.iter().filter(|(_, s)| *s == crate::state::DiffLineState::Added).count();
                                                            removed = rev.diff_lines.iter().filter(|(_, s)| *s == crate::state::DiffLineState::Removed).count();
                                                            if rev.is_new_file {
                                                                is_created = true;
                                                                if added == 0 {
                                                                    added = rev.proposed_content.lines().count().max(1);
                                                                }
                                                            }
                                                        }
                                                        if is_created && added == 0 {
                                                            if let Ok(content) = std::fs::read_to_string(&full_p) {
                                                                let count = content.lines().count();
                                                                if count > 0 {
                                                                    added = count;
                                                                }
                                                            }
                                                        }
                                                    }

                                                    let display_path = if let Ok(rel) = full_p.strip_prefix(&state.current_dir) {
                                                        let s = rel.to_string_lossy().to_string();
                                                        if s.is_empty() || s == "." { fname.to_string() } else { s }
                                                    } else {
                                                        fname.to_string()
                                                    };

                                                    let action_verb = if is_created { "Created" } else { "Edited" };
                                                    let action_color = if is_created { Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 } } else { colors.text_muted };

                                                    let edit_row = ui.container()
                                                        .id(step_id)
                                                        .fill_x()
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .radius_all(3.0)
                                                        .interactive(true)
                                                        .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(5.0)
                                                        .show(|ui| {
                                                            ui.text(action_verb)
                                                                .size(11.0)
                                                                .color(action_color)
                                                                .show();
                                                            file_icon.show(ui, 12.0);
                                                            ui.text(&display_path)
                                                                .size(11.5)
                                                                .color(colors.text_primary)
                                                                .show();

                                                            if is_created {
                                                                if added > 0 {
                                                                    ui.text(&format!("+{}", added))
                                                                        .size(10.5)
                                                                        .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                        .show();
                                                                }
                                                            } else {
                                                                if added > 0 {
                                                                    ui.text(&format!("+{}", added))
                                                                        .size(10.5)
                                                                        .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                        .show();
                                                                }
                                                                if removed > 0 {
                                                                    ui.text(&format!("-{}", removed))
                                                                        .size(10.5)
                                                                        .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                                        .show();
                                                                }
                                                            }

                                                            let chevron_glyph = if *expanded { "\u{f078}" } else { "\u{f054}" };
                                                            ui.text(chevron_glyph)
                                                                .size(7.0)
                                                                .color(colors.text_muted.with_alpha(0.6))
                                                                .show();
                                                        });

                                                    if edit_row.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                                    if edit_row.clicked {
                                                        toggle_file_edit_step = Some((msg_idx, step_idx));
                                                    }

                                                    if *expanded {
                                                        let (rev_is_created, preview_lines, total_lines, full_content) = prepare_file_preview(&full_p, &state.pending_reviews, None);
                                                        let final_is_created = is_created || rev_is_created;
                                                        let delta_info = if final_is_created {
                                                            format!("(+{})", added)
                                                        } else if removed > 0 {
                                                            format!("(+{}, -{})", added, removed)
                                                        } else {
                                                            format!("(+{})", added)
                                                        };
                                                        let file_card_id = Id::from_u64(999_880_000 + (msg_idx as u64) * 10000 + (step_idx as u64) * 100);
                                                        render_write_file_box(
                                                            ui,
                                                            colors,
                                                            file_card_id,
                                                            Some(&file_icon),
                                                            final_is_created,
                                                            &display_path,
                                                            &delta_info,
                                                            &preview_lines,
                                                            total_lines,
                                                            &full_content,
                                                            &mut copy_code_snippet,
                                                            &mut chat_open_file,
                                                            full_p.clone(),
                                                        );
                                                    }
                                                }
                                                TrajectoryStep::Command { command } => {
                                                    ui.container()
                                                        .id(step_id)
                                                        .fill_x()
                                                        .padding(2.0, 4.0, 2.0, 4.0)
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(5.0)
                                                        .show(|ui| {
                                                            ui.text("Ran")
                                                                .size(11.0)
                                                                .color(colors.text_muted)
                                                                .show();
                                                            ui.text("\u{f120}")
                                                                .size(10.5)
                                                                .color(colors.accent)
                                                                .show();
                                                            ui.text(&format!("$ {}", command))
                                                                .size(11.5)
                                                                .font_family("monospace")
                                                                .color(colors.text_primary)
                                                                .show();
                                                        });
                                                }
                                                TrajectoryStep::Plan { title: _, steps, expanded: _ } => {
                                                    let plan_badge_id = Id::from_u64(999_400_000 + (msg_idx as u64) * 100 + (step_idx as u64));
                                                    let proceed_btn_id = Id::from_u64(999_400_000 + (msg_idx as u64) * 100 + (step_idx as u64) + 1);

                                                    ui.container()
                                                        .fill_x()
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
                                                                .border((colors.accent.with_alpha(0.35), 1.0))
                                                                .hover(Style::bg(colors.accent.with_alpha(0.24)))
                                                                .show(|ui| {
                                                                    ui.text("\u{f15c}")
                                                                        .size(10.0)
                                                                        .color(colors.accent)
                                                                        .show();
                                                                    ui.text("Implementation Plan")
                                                                        .size(10.5)
                                                                        .font_weight(FontWeight::SemiBold)
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
                                                                                    .font_weight(FontWeight::Medium)
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
                                                                .border((Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0).with_alpha(0.38), 1.0))
                                                                .hover(Style::bg(Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0).with_alpha(0.26)))
                                                                .show(|ui| {
                                                                    ui.text("\u{f04b}")
                                                                        .size(9.0)
                                                                        .color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                                                                        .show();
                                                                    ui.text("Proceed")
                                                                        .size(10.5)
                                                                        .font_weight(FontWeight::SemiBold)
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
                                                        .fill_x()
                                                        .column()
                                                        .gap(4.0)
                                                        .show(|ui| {
                                                            let header_res = ui.container()
                                                                .id(thought_id)
                                                                .fill_x()
                                                                .interactive(true)
                                                                .padding(2.0, 4.0, 2.0, 4.0)
                                                                .radius_all(4.0)
                                                                .hover(Style::bg(colors.highlight))
                                                                .row()
                                                                .valign(Align::Center)
                                                                .gap(6.0)
                                                                .show(|ui| {
                                                                    let is_thinking_active = msg.is_working && full_text.is_empty();
                                                                    if is_thinking_active {
                                                                        ui.request_redraw_after(std::time::Duration::from_millis(80));
                                                                    }
                                                                    ui.text(chevron)
                                                                        .size(7.5)
                                                                        .color(colors.text_muted)
                                                                        .show();
                                                                    if is_thinking_active {
                                                                        ui.text(get_thinking_spinner_frame())
                                                                            .size(11.0)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                    } else {
                                                                        ui.text("\u{f0eb}") // fa-lightbulb
                                                                            .size(10.5)
                                                                            .color(colors.accent)
                                                                            .show();
                                                                    }
                                                                    let title = if is_thinking_active {
                                                                        "Thinking..."
                                                                    } else {
                                                                        "Thought Process"
                                                                    };
                                                                    ui.text(title)
                                                                        .size(11.5)
                                                                        .font_weight(FontWeight::Medium)
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
                                                                let thought_content_w = (ai_text_w - 20.0).max(40.0);
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_left(16.0)
                                                                    .padding_top(2.0)
                                                                    .padding_bottom(4.0)
                                                                    .show(|ui| {
                                                                        ui.text(thoughts.trim())
                                                                            .size(11.5)
                                                                            .decoration(TextDecoration::LineHeight(1.4))
                                                                            .color(colors.text_muted)
                                                                            .max_width(thought_content_w)
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
                                            ui.request_redraw_after(std::time::Duration::from_millis(80));
                                            ui.container()
                                                .row()
                                                .valign(Align::Center)
                                                .gap(6.0)
                                                .padding(3.0, 4.0, 3.0, 4.0)
                                                .show(|ui| {
                                                    ui.text(get_thinking_spinner_frame())
                                                        .size(12.0)
                                                        .color(colors.accent)
                                                        .show();
                                                    ui.text("Thinking...")
                                                        .size(12.0)
                                                        .decoration(TextDecoration::Italic)
                                                        .color(colors.accent)
                                                        .show();
                                                });
                                        } else if !msg.is_working && full_text.is_empty() && msg.trajectory.is_empty() && !has_visible_thoughts {
                                            ui.text("No response generated. Please check your model or API quota.")
                                                .size(11.5)
                                                .decoration(TextDecoration::Italic)
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
                                                .fill_x()
                                                .radius_all(8.0)
                                                .bg(Color::rgba(0.95, 0.25, 0.25, 0.08))
                                                .border((Color::rgba(0.95, 0.30, 0.30, 0.28), 1.0))
                                                .padding(10.0, 12.0, 10.0, 12.0)
                                                .column()
                                                .gap(8.0)
                                                .show(|ui| {
                                                    // Header row: Icon + Title + Error Badge
                                                    ui.container()
                                                        .fill_x()
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
                                                                        .font_weight(FontWeight::Bold)
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
                                                                        .font_weight(FontWeight::Medium)
                                                                        .color(Color::rgba(0.98, 0.55, 0.55, 0.95))
                                                                        .show();
                                                                });
                                                        });

                                                    // Friendly Description
                                                    ui.text(&err_desc)
                                                        .size(11.0)
                                                        .decoration(TextDecoration::LineHeight(1.4))
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
                                                            .hover(Style::bg(Color::rgba(1.0, 1.0, 1.0, 0.05)))
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
                                                                .border((Color::rgba(0.95, 0.30, 0.30, 0.2), 0.5))
                                                                .column()
                                                                .gap(4.0)
                                                                .show(|ui| {
                                                                    ui.text(raw)
                                                                        .size(9.5)
                                                                        .font_family("monospace")
                                                                        .decoration(TextDecoration::LineHeight(1.5))
                                                                        .color(colors.text_muted)
                                                                        .max_width((raw_w - 20.0).max(40.0))
                                                                        .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                        .show();
                                                                });
                                                        }
                                                    }

                                                    // Footer: Copy Button
                                                    ui.container()
                                                        .fill_x()
                                                        .row()
                                                        .halign(Align::Right)
                                                        .valign(Align::Center)
                                                        .padding_top(4.0)
                                                        .show(|ui| {
                                                            let copy_btn = ui.container()
                                                                .id(err_copy_id)
                                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                                .radius_all(4.0)
                                                                .hover(Style::bg(Color::rgba(1.0, 1.0, 1.0, 0.08)))
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
                                            ui.container()
                                                .fill_x()
                                                .column()
                                                .gap(0.0)
                                                .show(|ui| {
                                                    for (block_idx, block) in blocks.iter().enumerate() {
                                                        match block {
                                                            MarkdownBlock::Heading { level, spans } => {
                                                                let (font_size, pad_t, pad_b) = match level {
                                                                    1 => (base_text_size + 2.5, if block_idx == 0 { 0.0_f32 } else { 12.0_f32 }, 6.0_f32),
                                                                    2 => (base_text_size + 1.0, if block_idx == 0 { 0.0 } else { 10.0 }, 5.0),
                                                                    3 => (base_text_size, if block_idx == 0 { 0.0 } else { 8.0 }, 4.0),
                                                                    _ => (base_text_size - 0.5, if block_idx == 0 { 0.0 } else { 6.0 }, 3.0),
                                                                };
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_top(pad_t)
                                                                    .padding_bottom(pad_b)
                                                                    .show(|ui| {
                                                                        draw_inline_spans(ui, spans, ai_text_w, font_size, FontWeight::Bold, colors.text_primary, colors);
                                                                    });
                                                            }
                                                            MarkdownBlock::Rule => {
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding(10.0, 0.0, 10.0, 0.0)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .fill_x()
                                                                            .height(1.0)
                                                                            .bg(colors.border.with_alpha(0.4))
                                                                            .show(|_| {});
                                                                    });
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
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_bottom(8.0)
                                                                    .show(|ui| {
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
                                                                                                        .size(9.5)
                                                                                                        .color(accent_col)
                                                                                                        .show();
                                                                                                    ui.text(badge_label)
                                                                                                        .size(9.5)
                                                                                                        .font_weight(FontWeight::Bold)
                                                                                                        .color(accent_col)
                                                                                                        .show();
                                                                                                });
                                                                                        }
                                                                                        draw_inline_spans(ui, spans, (ai_text_w - 20.0).max(40.0), (base_text_size - 0.5).max(11.0), FontWeight::Regular, colors.text_muted, colors);
                                                                                    });
                                                                            });
                                                                    });
                                                            }
                                                            MarkdownBlock::Table { alignments: _aligns, headers, rows } => {
                                                                let col_count = headers.len().max(1);
                                                                let col_w = ((ai_text_w - 2.0) / col_count as f32).max(40.0);
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_bottom(8.0)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .max_width(ai_text_w)
                                                                            .radius_all(6.0)
                                                                            .border((colors.border.with_alpha(0.5), 0.5))
                                                                            .clip(true)
                                                                            .column()
                                                                            .show(|ui| {
                                                                                // Header row
                                                                                ui.container()
                                                                                    .fill_x()
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
                                                                                                    draw_inline_spans(ui, cell, col_w - 16.0, (base_text_size - 1.0).max(11.0), FontWeight::SemiBold, colors.text_primary, colors);
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
                                                                                ui.container().fill_x().height(0.5).bg(colors.border.with_alpha(0.5)).show(|_| {});
                                                                                // Data rows
                                                                                for (ri, row) in rows.iter().enumerate() {
                                                                                    let row_bg = if ri % 2 == 1 {
                                                                                        colors.bg_sidebar.with_alpha(0.3)
                                                                                    } else {
                                                                                        Color::TRANSPARENT
                                                                                    };
                                                                                    ui.container()
                                                                                        .fill_x()
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
                                                                                                        draw_inline_spans(ui, cell, col_w - 16.0, (base_text_size - 1.0).max(11.0), FontWeight::Regular, colors.text_muted, colors);
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
                                                                                        ui.container().fill_x().height(0.5).bg(colors.border.with_alpha(0.2)).show(|_| {});
                                                                                    }
                                                                                }
                                                                            });
                                                                    });
                                                            }

                                                            MarkdownBlock::BulletItem { depth, spans, checked } => {
                                                                let indent = (*depth as f32) * 14.0;
                                                                let marker_w = 16.0;
                                                                let avail_w = (ai_text_w - indent - marker_w - 6.0).max(40.0);
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_bottom(4.0)
                                                                    .row()
                                                                    .valign(Align::Top)
                                                                    .gap(6.0)
                                                                    .padding_left(indent)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .width(marker_w)
                                                                            .padding_top(2.5) // Optical baseline alignment with base font
                                                                            .row()
                                                                            .halign(Align::Center)
                                                                            .valign(Align::Top)
                                                                            .show(|ui| {
                                                                                match checked {
                                                                                    Some(true) => {
                                                                                        ui.text("\u{f14a}")
                                                                                            .size(11.0)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                    Some(false) => {
                                                                                        ui.text("\u{f0c8}")
                                                                                            .size(11.0)
                                                                                            .color(colors.text_muted)
                                                                                            .show();
                                                                                    }
                                                                                    None => {
                                                                                        let bullet = if *depth == 0 { "•" } else { "◦" };
                                                                                        ui.text(bullet)
                                                                                            .size(10.5)
                                                                                            .color(colors.accent)
                                                                                            .show();
                                                                                    }
                                                                                }
                                                                            });
                                                                        draw_inline_spans(ui, spans, avail_w, base_text_size, FontWeight::Regular, colors.text_primary, colors);
                                                                    });
                                                            }
                                                            MarkdownBlock::NumberedItem { num, depth, spans } => {
                                                                let indent = (*depth as f32) * 14.0;
                                                                let num_w = 20.0;
                                                                let avail_w = (ai_text_w - indent - num_w - 6.0).max(40.0);
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_bottom(4.0)
                                                                    .row()
                                                                    .valign(Align::Top)
                                                                    .gap(6.0)
                                                                    .padding_left(indent)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .width(num_w)
                                                                            .padding_top(0.5) // Optical baseline alignment with base font
                                                                            .row()
                                                                            .halign(Align::Left)
                                                                            .valign(Align::Top)
                                                                            .show(|ui| {
                                                                                ui.text(&format!("{}.", num))
                                                                                    .size(12.0)
                                                                                    .font_weight(FontWeight::Medium)
                                                                                    .color(colors.accent)
                                                                                    .show();
                                                                            });
                                                                        draw_inline_spans(ui, spans, avail_w, base_text_size, FontWeight::Regular, colors.text_primary, colors);
                                                                    });
                                                            }
                                                            MarkdownBlock::CodeBlock { lang, code } => {
                                                                let code_btn_id = Id::from_u64(999_500_000 + (msg_idx as u64) * 1000 + (block_idx as u64));
                                                                let is_code_copied = state.copied_ai_code_id.as_ref()
                                                                    .map(|(id, t)| *id == code_btn_id.raw() && t.elapsed().as_secs_f32() < 2.0)
                                                                    .unwrap_or(false);

                                                                let lang_display = lang.as_deref().unwrap_or("code").to_uppercase();

                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_bottom(8.0)
                                                                    .show(|ui| {
                                                                        ui.container()
                                                                            .width(code_box_w)
                                                                            .radius_all(6.0)
                                                                            .bg(colors.bg_base.with_alpha(0.85))
                                                                            .border((colors.border.with_alpha(0.5), 0.5))
                                                                            .column()
                                                                            .show(|ui| {
                                                                                // Code card header: language badge + Copy Code button
                                                                                ui.container()
                                                                                    .fill_x()
                                                                                    .height(24.0)
                                                                                    .padding(0.0, 8.0, 0.0, 8.0)
                                                                                    .bg(colors.bg_sidebar.with_alpha(0.6))
                                                                                    .radius_top(6.0)
                                                                                    .row()
                                                                                    .halign(Align::SpaceBetween)
                                                                                    .valign(Align::Center)
                                                                                    .show(|ui| {
                                                                                        ui.text(&lang_display)
                                                                                            .size(10.0)
                                                                                            .font_weight(FontWeight::Bold)
                                                                                            .color(colors.text_muted)
                                                                                            .show();

                                                                                        let copy_code_btn = ui.container()
                                                                                            .id(code_btn_id)
                                                                                            .padding(2.0, 6.0, 2.0, 6.0)
                                                                                            .radius_all(3.0)
                                                                                            .hover(Style::bg(colors.highlight))
                                                                                            .row()
                                                                                            .valign(Align::Center)
                                                                                            .gap(4.0)
                                                                                            .show(|ui| {
                                                                                                ui.text(if is_code_copied { "\u{f00c}" } else { "\u{f0c5}" })
                                                                                                    .size(9.5)
                                                                                                    .color(if is_code_copied { colors.accent } else { colors.text_muted })
                                                                                                    .show();
                                                                                                ui.text(if is_code_copied { "Copied" } else { "Copy" })
                                                                                                    .size(9.5)
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
                                                                                    .padding(8.0, 10.0, 8.0, 10.0)
                                                                                    .fill_x()
                                                                                    .show(|ui| {
                                                                                        ui.text(code)
                                                                                            .size((base_text_size - 1.0).max(10.5))
                                                                                            .font_family("monospace")
                                                                                            .decoration(TextDecoration::LineHeight(1.45))
                                                                                            .color(colors.text_primary)
                                                                                            .max_width((code_box_w - 20.0).max(40.0))
                                                                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                                                                            .show();
                                                                                    });
                                                                            });
                                                                    });
                                                            }
                                                            MarkdownBlock::Paragraph(spans) => {
                                                                ui.container()
                                                                    .fill_x()
                                                                    .padding_bottom(8.0)
                                                                    .show(|ui| {
                                                                        draw_inline_spans(ui, spans, ai_text_w, base_text_size, FontWeight::Regular, colors.text_primary, colors);
                                                                    });
                                                            }
                                                        }
                                                    }
                                                });

                                            // ── Footer: copy button pinned right ──────────────
                                            ui.container()
                                                .fill_x()
                                                .row()
                                                .halign(Align::Right)
                                                .valign(Align::Center)
                                                .padding_top(4.0)
                                                .show(|ui| {
                                                    let copy_btn = ui.container()
                                                        .id(copy_btn_id)
                                                        .padding(2.0, 5.0, 2.0, 5.0)
                                                        .radius_all(4.0)
                                                        .hover(Style::bg(colors.highlight))
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

                        if let Some((m_idx, s_idx, it_idx)) = toggle_exploration_item {
                            if let Some(m) = state.ai_messages.get_mut(m_idx) {
                                if let Some(step) = m.trajectory.get_mut(s_idx) {
                                    if let TrajectoryStep::Exploration(ref mut grp) = step {
                                        if let Some(item) = grp.items.get_mut(it_idx) {
                                            item.toggle_expanded();
                                            ui.needs_redraw = true;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((m_idx, s_idx)) = toggle_file_edit_step {
                            if let Some(m) = state.ai_messages.get_mut(m_idx) {
                                if let Some(step) = m.trajectory.get_mut(s_idx) {
                                    if let TrajectoryStep::FileEdit { ref mut expanded, .. } = step {
                                        *expanded = !*expanded;
                                        ui.needs_redraw = true;
                                    }
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

                    // ─── Active Agent Terminal Running View / Modified Files View / Quick Toolbar ──
                    if state.show_agent_terminal_popup && !state.active_agent_terminal_tasks.is_empty() {
                        let term_list_h = (state.active_agent_terminal_tasks.len().max(1) as f32 * 24.0 + 8.0).min(140.0);
                        let mut term_box_builder = ui.container()
                            .fill_x()
                            .height(term_list_h)
                            .bg(input_box_bg)
                            .radius_all(7.0)
                            .padding(4.0, 6.0, 4.0, 6.0)
                            .scroll_y(true)
                            .column()
                            .gap(2.0);

                        if state.ai_settings.effective_input_shadow() {
                            term_box_builder = term_box_builder.shadow(input_shadow_col, 0.0, 2.0, 12.0);
                        }
                        if state.ai_settings.effective_input_outline() {
                            term_box_builder = term_box_builder.border((colors.border.with_alpha(0.6), 0.75));
                        }
                        if state.glassmorphism {
                            term_box_builder = term_box_builder.backdrop_blur(state.blur_strength.max(4.0));
                        }

                        let mut target_cancel_task_id: Option<String> = None;

                        term_box_builder.show(|ui| {
                            for (idx, task) in state.active_agent_terminal_tasks.iter().enumerate() {
                                let row_id = ui.id_from(&format!("active_term_task_{}", idx));
                                let row = ui.container()
                                    .id(row_id)
                                    .fill_x()
                                    .height(22.0)
                                    .radius_all(3.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                    .padding(0.0, 4.0, 0.0, 4.0)
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .show(|ui| {
                                        // Left: 🟢 pulsating green indicator + command name + elapsed/status
                                        ui.container()
                                            .row()
                                            .valign(Align::Center)
                                            .gap(6.0)
                                            .show(|ui| {
                                                // Pulsing green circle
                                                ui.container()
                                                    .width(7.0)
                                                    .height(7.0)
                                                    .radius_all(3.5)
                                                    .bg(Color { r: 34.0 / 255.0, g: 197.0 / 255.0, b: 94.0 / 255.0, a: 1.0 })
                                                    .show(|_| {});

                                                // Command string (e.g. cargo test -p agent)
                                                ui.text(&task.command)
                                                    .size(11.0)
                                                    .font_weight(FontWeight::Medium)
                                                    .color(colors.text_primary)
                                                    .show();

                                                // Elapsed time (e.g. 1.8s)
                                                let elapsed = task.started_at.elapsed().as_secs_f32();
                                                let elapsed_str = format!("({:.1}s)", elapsed);
                                                ui.text(&elapsed_str)
                                                    .size(10.0)
                                                    .color(colors.text_muted)
                                                    .show();
                                            });

                                        // Right: [✕] CROSS ICON ON THE LIST to cancel/kill
                                        let cross_id = ui.id_from(&format!("term_task_cross_{}", idx));
                                        let cross_btn = ui.container()
                                            .id(cross_id)
                                            .interactive(true)
                                            .padding(2.0, 5.0, 2.0, 5.0)
                                            .radius_all(2.5)
                                            .hover(Style::bg(Color::rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.15)))
                                            .show(|ui| {
                                                ui.text("✕")
                                                    .size(9.0)
                                                    .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                    .show();
                                            });
                                        if cross_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if cross_btn.clicked {
                                            target_cancel_task_id = Some(task.id.clone());
                                        }
                                    });
                                if row.hovered { ui.cursor_icon = CursorIcon::Default; }
                            }
                        });

                        if let Some(id) = target_cancel_task_id {
                            state.cancel_agent_terminal_task(&id);
                            ui.needs_redraw = true;
                        }

                        // 2. Toolbar row below container: [ ‹ 💻 Terminal ] on left, [ ▾ ] on right
                        ui.container()
                            .fill_x()
                            .height(20.0)
                            .padding(0.0, 2.0, 0.0, 2.0)
                            .row()
                            .halign(Align::SpaceBetween)
                            .valign(Align::Center)
                            .show(|ui| {
                                // Left: [ ‹ 💻 Terminal ] toggle button with back arrow ‹
                                let back_btn = ui.container()
                                    .id(Id::from_u64(999999870))
                                    .interactive(true)
                                    .padding(2.0, 5.0, 2.0, 5.0)
                                    .radius_all(3.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                    .row()
                                    .valign(Align::Center)
                                    .gap(5.0)
                                    .show(|ui| {
                                        ui.text("‹")
                                            .size(13.0)
                                            .font_weight(FontWeight::Bold)
                                            .color(colors.text_muted)
                                            .show();
                                        ui.text("\u{f489}")
                                            .size(12.0)
                                            .color(colors.accent)
                                            .show();
                                        let count = state.active_agent_terminal_tasks.len();
                                        let label = if count <= 1 {
                                            "Terminal".to_string()
                                        } else {
                                            format!("{} Running", count)
                                        };
                                        ui.text(&label)
                                            .size(10.5)
                                            .font_weight(FontWeight::Medium)
                                            .color(colors.accent)
                                            .show();
                                    });
                                if back_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if back_btn.clicked {
                                    state.show_agent_terminal_popup = false;
                                    ui.needs_redraw = true;
                                }

                                // Right: Down arrow ▾ to collapse
                                let arrow_btn = ui.container()
                                    .id(Id::from_u64(999999871))
                                    .interactive(true)
                                    .padding(2.0, 5.0, 2.0, 5.0)
                                    .radius_all(3.0)
                                    .bg(Color::TRANSPARENT)
                                    .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                    .show(|ui| {
                                        ui.text("\u{f078}")
                                            .size(7.5)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                                if arrow_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if arrow_btn.clicked {
                                    state.show_agent_terminal_popup = false;
                                    ui.needs_redraw = true;
                                }
                            });
                    } else if state.show_modified_files_popup {
                        if has_modified_files {
                            // 1. Scrollable file list container (only for the file list) with input background & drop shadow
                            let file_list_h = (modified_files.len().max(1) as f32 * 22.0 + 8.0).min(140.0);
                            let mut mod_files_builder = ui.container()
                                .fill_x()
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
                                mod_files_builder = mod_files_builder.border((colors.border.with_alpha(0.6), 0.75));
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
                                            .fill_x()
                                            .height(20.0)
                                            .radius_all(3.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                            .padding(0.0, 4.0, 0.0, 4.0)
                                            .interactive(true)
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
                                                            .border((icon_color, 1.0))
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
                                                            .font_weight(FontWeight::Medium)
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
                                                                .font_weight(FontWeight::Medium)
                                                                .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                                                .show();
                                                        }
                                                        if item.lines_removed > 0 {
                                                            ui.text(&format!("-{}", item.lines_removed))
                                                                .size(10.0)
                                                                .font_weight(FontWeight::Medium)
                                                                .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                                                .show();
                                                        }

                                                        // Cross icon to reject this individual file
                                                        let cross_id = ui.id_from(&format!("mod_file_cross_{}_{}", idx, item.file_name));
                                                        let cross_btn = ui.container()
                                                            .id(cross_id)
                                                            .interactive(true)
                                                            .padding(1.0, 3.0, 1.0, 3.0)
                                                            .radius_all(2.5)
                                                            .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.08)))
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
                                .fill_x()
                                .height(20.0)
                                .padding(0.0, 2.0, 0.0, 2.0)
                                .row()
                                .halign(Align::SpaceBetween)
                                .valign(Align::Center)
                                .show(|ui| {
                                    // Left: [ ‹ 📄 N Files] toggle button (left back icon to close)
                                    let toggle_btn = ui.container()
                                        .id(Id::from_u64(999999865))
                                        .interactive(true)
                                        .padding(2.0, 5.0, 2.0, 5.0)
                                        .radius_all(3.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                        .row()
                                        .valign(Align::Center)
                                        .gap(5.0)
                                        .show(|ui| {
                                            ui.text("‹")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
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
                                                .font_weight(FontWeight::Medium)
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
                                                .interactive(true)
                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
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
                                                .interactive(true)
                                                .padding(2.0, 8.0, 2.0, 8.0)
                                                .radius_all(4.0)
                                                .bg(colors.accent)
                                                .hover(Style::bg(colors.accent.with_alpha(0.85)))
                                                .show(|ui| {
                                                    ui.text("Accept all")
                                                        .size(11.0)
                                                        .font_weight(FontWeight::SemiBold)
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
                                                .interactive(true)
                                                .padding(2.0, 5.0, 2.0, 5.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
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
                                .fill_x()
                                .height(20.0)
                                .padding(0.0, 2.0, 0.0, 2.0)
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    let toggle_btn = ui.container()
                                        .interactive(true)
                                        .padding(2.0, 4.0, 2.0, 4.0)
                                        .radius_all(3.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
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
                            .fill_x()
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
                                            .interactive(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
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
                                                        .font_weight(FontWeight::Medium)
                                                        .color(colors.accent)
                                                        .show();
                                                }
                                            });
                                        if file_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if file_btn.clicked {
                                            state.show_modified_files_popup = !state.show_modified_files_popup;
                                            if state.show_modified_files_popup {
                                                state.show_agent_terminal_popup = false;
                                            }
                                            ui.needs_redraw = true;
                                        }

                                        // 2. Terminal icon
                                        let term_id = Id::from_u64(999999861);
                                        let has_active_agent_terminal = !state.active_agent_terminal_tasks.is_empty();
                                        let term_bg = if has_active_agent_terminal || state.terminal_visible {
                                            colors.accent.with_alpha(0.18)
                                        } else {
                                            Color::TRANSPARENT
                                        };
                                        let term_btn = ui.container()
                                            .id(term_id)
                                            .interactive(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .bg(term_bg)
                                            .hover(Style::bg(colors.highlight))
                                            .row()
                                            .valign(Align::Center)
                                            .gap(2.5)
                                            .show(|ui| {
                                                ui.text("\u{f489}")
                                                    .size(12.0)
                                                    .color(if has_active_agent_terminal || state.terminal_visible { colors.accent } else { colors.text_muted })
                                                    .show();
                                                if has_active_agent_terminal {
                                                    ui.container()
                                                        .width(4.5)
                                                        .height(4.5)
                                                        .radius_all(2.25)
                                                        .bg(Color { r: 34.0 / 255.0, g: 197.0 / 255.0, b: 94.0 / 255.0, a: 1.0 })
                                                        .show(|_| {});
                                                }
                                            });
                                        if term_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                        if term_btn.clicked {
                                            if has_active_agent_terminal {
                                                state.show_agent_terminal_popup = !state.show_agent_terminal_popup;
                                                if state.show_agent_terminal_popup {
                                                    state.show_modified_files_popup = false;
                                                }
                                                ui.needs_redraw = true;
                                            } else {
                                                state.terminal_visible = !state.terminal_visible;
                                                if state.terminal_visible {
                                                    state.ensure_terminal_session();
                                                }
                                                state.save_persistent_state();
                                                ui.needs_redraw = true;
                                            }
                                        }

                                        // 3. Inbox icon
                                        let inbox_id = Id::from_u64(999999862);
                                        let inbox_btn = ui.container()
                                            .id(inbox_id)
                                            .interactive(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .hover(Style::bg(colors.highlight))
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
                                            .interactive(true)
                                            .padding(2.0, 4.0, 2.0, 4.0)
                                            .radius_all(3.0)
                                            .hover(Style::bg(colors.highlight))
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
                                                .interactive(true)
                                                .padding(2.0, 6.0, 2.0, 6.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
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
                                                .interactive(true)
                                                .padding(2.0, 8.0, 2.0, 8.0)
                                                .radius_all(4.0)
                                                .bg(colors.accent)
                                                .hover(Style::bg(colors.accent.with_alpha(0.85)))
                                                .show(|ui| {
                                                    ui.text("Accept all")
                                                        .size(11.0)
                                                        .font_weight(FontWeight::SemiBold)
                                                        .color(Color::WHITE)
                                                        .show();
                                                });
                                            if acc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if acc_btn.clicked {
                                                action_accept_all = true;
                                            }

                                            // Top arrow button to open/expand
                                            let arrow_btn = ui.container()
                                                .id(Id::from_u64(999999869))
                                                .interactive(true)
                                                .padding(2.0, 5.0, 2.0, 5.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
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
                                } else if !state.active_agent_terminal_tasks.is_empty() {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(5.0)
                                        .show(|ui| {
                                            if let Some(task) = state.active_agent_terminal_tasks.first() {
                                                let elapsed = task.started_at.elapsed().as_secs_f32();
                                                ui.text(&format!("Running ({:.1}s)", elapsed))
                                                    .size(10.5)
                                                    .color(colors.accent)
                                                    .show();
                                            }
                                            let arrow_btn = ui.container()
                                                .id(Id::from_u64(999999872))
                                                .interactive(true)
                                                .padding(2.0, 5.0, 2.0, 5.0)
                                                .radius_all(3.0)
                                                .bg(Color::TRANSPARENT)
                                                .hover(Style::bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.05)))
                                                .show(|ui| {
                                                    ui.text("\u{f077}")
                                                        .size(7.5)
                                                        .color(colors.text_muted)
                                                        .show();
                                                });
                                            if arrow_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                            if arrow_btn.clicked {
                                                state.show_agent_terminal_popup = true;
                                                ui.needs_redraw = true;
                                            }
                                        });
                                }
                            });
                    }

                    let mut input_box_builder = ui.container()
                        .fill_x()
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
                        input_box_builder = input_box_builder.border((outline_col, outline_w));
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
                            let placeholder = match state.selected_agent {
                                SelectedAgent::Zene => {
                                    if input_dock_w >= 240.0 {
                                        "Ask ZENE anything, @ for context, / for actions"
                                    } else {
                                        "Ask ZENE..."
                                    }
                                }
                                SelectedAgent::Antigravity => {
                                    if input_dock_w >= 240.0 {
                                        "Ask Antigravity or type / for commands..."
                                    } else {
                                        "Ask Antigravity..."
                                    }
                                }
                            };

                            let input_w = (input_dock_w - 50.0).max(64.0);
                            let orb_id = Id::from_u64(999999990);
                            let mut input_submitted = false;
                            ui.container()
                                .fill_x()
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

                                    let text_res = ui.text_area(&mut state.ai_input).id("ai_prompt_text_area")
                                        .width(input_w)
                                        .height(prompt_box_h)
                                        .max_height(MAX_PROMPT_H)
                                        .scrollable(true)
                                        .size(prompt_text_size)
                                                                                .color(colors.text_primary)
                                        .cursor_color(colors.text_primary)
                                        .bg(if state.glassmorphism { Color::TRANSPARENT } else { input_box_bg })
                                        .border((Color::TRANSPARENT, 0.0))
                                        .focus_border((Color::TRANSPARENT, 0.0))
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
                                .fill_x()
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
                                            let (mode_icon, mode_name) = match state.selected_agent {
                                                SelectedAgent::Zene => (state.agent_mode.icon(), state.agent_mode.display_name()),
                                                SelectedAgent::Antigravity => (state.antigravity_mode.icon(), state.antigravity_mode.display_name()),
                                            };

                                            let mode_btn = ui.container()
                                                .raw_id(AGENT_MODE_BTN_ID)
                                                .interactive(true)
                                                .padding(3.5, 8.0, 3.5, 8.0)
                                                .radius_all(6.0)
                                                .bg(if state.agent_mode_menu_open { pill_hover_bg } else { pill_bg })
                                                .hover(Style::bg(pill_hover_bg))
                                                .border((pill_border, 1.0))
                                                .row()
                                                .valign(Align::Center)
                                                .gap(5.0)
                                                .show(|ui| {
                                                    ui.text(mode_icon)
                                                        .size(9.5)
                                                        .color(colors.text_muted)
                                                        .show();
                                                    ui.text(mode_name)
                                                        .size(10.0)
                                                        .font_weight(FontWeight::Medium)
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
                                            draw_prompt_model_button(ui, state, colors, pill_bg, pill_hover_bg, pill_border);
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
                                        .hover(Style::bg(send_hover_bg))
                                        .border((send_border, 1.0))
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
                        // Trajectory steps are preserved in chat history
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
                        // Trajectory steps are preserved in chat history
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
                    .font_weight(base_weight)
                    .color(base_color)
                    .decoration(TextDecoration::LineHeight(1.4))
                    .max_width(max_width)
                    .wrap(zenthra_text::prelude::TextWrap::Word)
                    .show();
                return;
            }
            InlineSpan::Code(t) => {
                ui.container()
                    .radius_all(3.0)
                    .bg(colors.bg_base.with_alpha(0.8))
                    .border((colors.border.with_alpha(0.4), 0.5))
                    .padding(1.0, 4.0, 1.0, 4.0)
                    .show(|ui| {
                        ui.text(t)
                            .size((base_size - 0.5).max(8.5))
                            .font_family("monospace")
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
        .fill_x()
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
                            .font_weight(base_weight)
                            .color(color)
                            .decoration(TextDecoration::LineHeight(1.4))
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::Bold(t) => {
                        ui.text(t)
                            .size(base_size)
                            .font_weight(FontWeight::SemiBold)
                            .color(base_color)
                            .decoration(TextDecoration::LineHeight(1.4))
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::Italic(t) => {
                        ui.text(t)
                            .size(base_size)
                            .font_weight(base_weight)
                            .decoration(TextDecoration::Italic)
                            .color(base_color.with_alpha(0.85))
                            .decoration(TextDecoration::LineHeight(1.4))
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::BoldItalic(t) => {
                        ui.text(t)
                            .size(base_size)
                            .font_weight(FontWeight::SemiBold)
                            .decoration(TextDecoration::Italic)
                            .color(base_color)
                            .decoration(TextDecoration::LineHeight(1.4))
                            .max_width(max_width)
                            .wrap(zenthra_text::prelude::TextWrap::Word)
                            .show();
                    }
                    InlineSpan::Code(t) => {
                        ui.container()
                            .radius_all(3.0)
                            .bg(colors.bg_base.with_alpha(0.8))
                            .border((colors.border.with_alpha(0.4), 0.5))
                            .padding(1.0, 4.0, 1.0, 4.0)
                            .show(|ui| {
                                ui.text(t)
                                    .size((base_size - 0.5).max(8.5))
                                    .font_family("monospace")
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
                            .interactive(true)
                            .padding(2.0, 6.0, 2.0, 4.0)
                            .radius_all(4.0)
                            .hover(Style::bg(colors.highlight))
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
                                    .font_weight(FontWeight::Medium)
                                    .color(colors.text_primary)
                                    .show();
                            });
                    }

                    // Right: Close button
                    let close_side_btn = ui.container()
                        .interactive(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .hover(Style::bg(colors.highlight))
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

                            ui.input(&mut state.ai_sessions_search_query).id(search_input_id)
                                .fill_x()
                                .size(11.0)
                                .color(colors.text_primary)
                                .bg(Color::TRANSPARENT)
                                .border(None)
                                .show();

                            if !query_empty {
                                let clear_btn = ui.container()
                                    .padding(2.0, 4.0, 2.0, 4.0)
                                    .radius_all(3.0)
                                    .hover(Style::bg(colors.highlight))
                                    .interactive(true)
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
                            .fill_x()
                            .interactive(true)
                            .padding(3.0, 4.0, 3.0, 4.0)
                            .radius_all(4.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                // Left details: arrow, icon, name, count, optional path
                                ui.container()
                                    .max_width(left_max_w)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(5.0)
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
                                            .font_weight(FontWeight::Bold)
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
                                                .font_weight(FontWeight::Medium)
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
                                .fill_x()
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
                                                    .decoration(TextDecoration::Italic)
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
                                                .fill_x()
                                                .height(36.0)
                                                .interactive(true)
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
                                                .hover(Style::bg(if is_active {
                                                    colors.accent.with_alpha(0.18)
                                                } else {
                                                    colors.highlight
                                                }))
                                                .border((if is_active {
                                                    colors.accent.with_alpha(0.60)
                                                } else {
                                                    colors.border.with_alpha(0.50)
                                                }, 1.0))
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
                                                                .font_weight(if is_active { FontWeight::Bold } else { FontWeight::Medium })
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
                                                                .interactive(true)
                                                                .padding(2.5, 5.0, 2.5, 5.0)
                                                                .radius_all(3.0)
                                                                .hover(Style::bg(Color::rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.18)))
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
                            .fill_x()
                            .show(|ui| {
                                ui.text("No matching sessions found.")
                                    .size(11.0)
                                    .decoration(TextDecoration::Italic)
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

fn read_local_dir_summary(path: &std::path::Path) -> (String, String) {
    if let Ok(entries) = std::fs::read_dir(path) {
        let mut dirs = Vec::new();
        let mut files = Vec::new();
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if let Ok(ft) = e.file_type() {
                if ft.is_dir() {
                    dirs.push(name);
                } else {
                    let size_str = if let Ok(meta) = e.metadata() {
                        let len = meta.len();
                        if len < 1024 {
                            format!("{} B", len)
                        } else {
                            format!("{:.1} KB", len as f32 / 1024.0)
                        }
                    } else {
                        String::new()
                    };
                    files.push((name, size_str));
                }
            }
        }
        dirs.sort();
        files.sort_by(|a, b| a.0.cmp(&b.0));

        let meta = format!("{} items ({} files, {} folders)", files.len() + dirs.len(), files.len(), dirs.len());
        let mut lines = Vec::new();
        for d in &dirs {
            lines.push(format!("\u{f07b} {}/", d));
        }
        for (f, sz) in &files {
            if sz.is_empty() {
                lines.push(format!("\u{f15b} {}", f));
            } else {
                lines.push(format!("\u{f15b} {:<24} {}", f, sz));
            }
        }
        (meta, lines.join("\n"))
    } else {
        ("Directory".to_string(), "Directory not found or unreadable.".to_string())
    }
}

fn prepare_file_preview(
    path: &std::path::Path,
    pending_reviews: &[crate::state::FileDiffReview],
    output: Option<&str>,
) -> (bool, Vec<(usize, String, Option<crate::state::DiffLineState>)>, usize, String) {
    if let Some(rev) = pending_reviews.iter().find(|r| path_matches(&r.path, path)) {
        let is_created = rev.is_new_file;
        let diffs = &rev.diff_lines;
        let total = diffs.len();
        let preview = diffs.iter().take(5).enumerate().map(|(i, (line, state))| {
            (i + 1, line.clone(), Some(*state))
        }).collect();
        return (is_created, preview, total, rev.proposed_content.clone());
    }

    if let Ok(content) = std::fs::read_to_string(path) {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();
        let preview = lines.iter().take(5).enumerate().map(|(i, line)| {
            (i + 1, line.to_string(), None)
        }).collect();
        return (false, preview, total, content);
    }

    if let Some(out) = output {
        let lines: Vec<&str> = out.lines().collect();
        let total = lines.len();
        let preview = lines.iter().take(5).enumerate().map(|(i, line)| {
            (i + 1, line.to_string(), None)
        }).collect();
        return (false, preview, total, out.to_string());
    }

    (false, Vec::new(), 0, String::new())
}

fn render_write_file_box(
    ui: &mut Ui,
    colors: &ThemeColors,
    card_id: Id,
    file_icon: Option<&crate::theme::FileIcon>,
    is_created: bool,
    display_path: &str,
    delta_info: &str,
    preview_lines: &[(usize, String, Option<crate::state::DiffLineState>)],
    total_lines: usize,
    full_content: &str,
    copy_code_snippet: &mut Option<(u64, String)>,
    chat_open_file: &mut Option<std::path::PathBuf>,
    target_path: std::path::PathBuf,
) {
    let card_bg = colors.bg_base;

    ui.container()
        .fill_x()
        .padding(6.0, 8.0, 6.0, 8.0)
        .radius_all(5.0)
        .bg(card_bg)
        .border((colors.border.with_alpha(0.6), 1.0))
        .column()
        .gap(5.0)
        .show(|ui| {
            // Header row with title on left, delta_info and action buttons on right
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left: File icon + Title
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            if let Some(ic) = file_icon {
                                ic.show(ui, 13.0);
                            } else {
                                let icon_glyph = if is_created { "\u{f15b}" } else { "\u{f044}" };
                                ui.text(icon_glyph)
                                    .size(11.0)
                                    .color(if is_created { Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 } } else { colors.accent })
                                    .show();
                            }
                            let title_prefix = if is_created { "Created:" } else { "Edited:" };
                            ui.text(&format!("{} {}", title_prefix, display_path))
                                .size(11.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Right: Delta/lines info + Copy & Open action buttons
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            if !delta_info.is_empty() {
                                ui.text(delta_info)
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            }

                            // Copy button
                            let copy_btn_id = Id::from_u64(card_id.raw() + 1);
                            let copy_btn = ui.container()
                                .id(copy_btn_id)
                                .padding(2.0, 5.0, 2.0, 5.0)
                                .radius_all(3.0)
                                .hover(Style::bg(colors.highlight))
                                .row()
                                .valign(Align::Center)
                                .gap(3.0)
                                .show(|ui| {
                                    ui.text("\u{f0c5}")
                                        .size(8.5)
                                        .color(colors.text_muted)
                                        .show();
                                    ui.text("Copy")
                                        .size(9.5)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if copy_btn.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if copy_btn.clicked {
                                *copy_code_snippet = Some((copy_btn_id.raw(), full_content.to_string()));
                            }

                            // Open button
                            let open_btn_id = Id::from_u64(card_id.raw() + 2);
                            let open_btn = ui.container()
                                .id(open_btn_id)
                                .padding(2.0, 5.0, 2.0, 5.0)
                                .radius_all(3.0)
                                .hover(Style::bg(colors.highlight))
                                .row()
                                .valign(Align::Center)
                                .gap(3.0)
                                .show(|ui| {
                                    ui.text("\u{f08e}")
                                        .size(8.5)
                                        .color(colors.accent)
                                        .show();
                                    ui.text("Open")
                                        .size(9.5)
                                        .color(colors.accent)
                                        .show();
                                });
                            if open_btn.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if open_btn.clicked {
                                *chat_open_file = Some(target_path.clone());
                            }
                        });
                });

            // Content preview container (max 5 lines)
            ui.container()
                .fill_x()
                .padding(5.0, 7.0, 5.0, 7.0)
                .radius_all(3.5)
                .bg(colors.bg_sidebar.with_alpha(0.6))
                .border((colors.border.with_alpha(0.3), 1.0))
                .column()
                .gap(1.5)
                .show(|ui| {
                    for (line_num, line_str, diff_state) in preview_lines {
                        ui.container()
                            .fill_x()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text(&format!("{:>3}", line_num))
                                    .size(9.5)
                                    .font_family("monospace")
                                    .color(colors.text_muted.with_alpha(0.5))
                                    .show();
                                match diff_state {
                                    Some(crate::state::DiffLineState::Added) => {
                                        ui.text("+")
                                            .size(10.0)
                                            .font_family("monospace")
                                            .color(Color { r: 74.0 / 255.0, g: 222.0 / 255.0, b: 128.0 / 255.0, a: 1.0 })
                                            .show();
                                        ui.text(line_str)
                                            .size(10.0)
                                            .font_family("monospace")
                                            .color(Color { r: 187.0 / 255.0, g: 247.0 / 255.0, b: 208.0 / 255.0, a: 1.0 })
                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                            .show();
                                    }
                                    Some(crate::state::DiffLineState::Removed) => {
                                        ui.text("-")
                                            .size(10.0)
                                            .font_family("monospace")
                                            .color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                                            .show();
                                        ui.text(line_str)
                                            .size(10.0)
                                            .font_family("monospace")
                                            .color(Color { r: 254.0 / 255.0, g: 202.0 / 255.0, b: 202.0 / 255.0, a: 1.0 })
                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                            .show();
                                    }
                                    _ => {
                                        ui.text(line_str)
                                            .size(10.0)
                                            .font_family("monospace")
                                            .color(colors.text_primary)
                                            .wrap(zenthra_text::prelude::TextWrap::Word)
                                            .show();
                                    }
                                }
                            });
                    }

                    // Show more lines button (opens directly in editor buffer)
                    if total_lines > preview_lines.len() {
                        let remaining = total_lines - preview_lines.len();
                        let more_btn_id = Id::from_u64(card_id.raw() + 5);
                        let more_btn = ui.container()
                            .id(more_btn_id)
                            .fill_x()
                            .padding(3.0, 4.0, 3.0, 4.0)
                            .radius_all(3.0)
                            .hover(Style::bg(colors.highlight))
                            .row()
                            .valign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                ui.text("\u{f078}")
                                    .size(8.0)
                                    .color(colors.accent)
                                    .show();
                                ui.text(&format!("Show {} more lines", remaining))
                                    .size(9.5)
                                    .color(colors.accent)
                                    .show();
                            });
                        if more_btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if more_btn.clicked {
                            *chat_open_file = Some(target_path);
                        }
                    }
                });
        });
}

fn render_tool_output_box(
    ui: &mut Ui,
    colors: &ThemeColors,
    card_id: Id,
    file_icon: Option<&crate::theme::FileIcon>,
    fallback_icon: Option<(&str, Color)>,
    title: &str,
    meta_info: &str,
    content: &str,
    copy_code_snippet: &mut Option<(u64, String)>,
    mut open_file_action: Option<(&mut Option<std::path::PathBuf>, std::path::PathBuf)>,
) {
    let card_bg = colors.bg_base;

    ui.container()
        .fill_x()
        .padding(6.0, 8.0, 6.0, 8.0)
        .radius_all(5.0)
        .bg(card_bg)
        .border((colors.border.with_alpha(0.6), 1.0))
        .column()
        .gap(5.0)
        .show(|ui| {
            // Header row with title on left, metadata and action buttons on right
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left: File/folder/tool icon + Title
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            if let Some(ic) = file_icon {
                                ic.show(ui, 13.0);
                            } else if let Some((glyph, col)) = fallback_icon {
                                ui.text(glyph)
                                    .size(11.0)
                                    .color(col)
                                    .show();
                            }
                            ui.text(title)
                                .size(11.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Right: Lines/metadata info + Copy & Open action buttons
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            if !meta_info.is_empty() {
                                ui.text(meta_info)
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            }

                            // Copy button
                            let copy_btn_id = Id::from_u64(card_id.raw() + 1);
                            let copy_btn = ui.container()
                                .id(copy_btn_id)
                                .padding(2.0, 5.0, 2.0, 5.0)
                                .radius_all(3.0)
                                .hover(Style::bg(colors.highlight))
                                .row()
                                .valign(Align::Center)
                                .gap(3.0)
                                .show(|ui| {
                                    ui.text("\u{f0c5}")
                                        .size(8.5)
                                        .color(colors.text_muted)
                                        .show();
                                    ui.text("Copy")
                                        .size(9.5)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if copy_btn.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if copy_btn.clicked {
                                *copy_code_snippet = Some((copy_btn_id.raw(), content.to_string()));
                            }

                            // Optional [Open in editor] button
                            if let Some((ref mut target_open, ref p)) = open_file_action {
                                let open_btn_id = Id::from_u64(card_id.raw() + 2);
                                let open_btn = ui.container()
                                    .id(open_btn_id)
                                    .padding(2.0, 5.0, 2.0, 5.0)
                                    .radius_all(3.0)
                                    .hover(Style::bg(colors.highlight))
                                    .row()
                                    .valign(Align::Center)
                                    .gap(3.0)
                                    .show(|ui| {
                                        ui.text("\u{f08e}")
                                            .size(8.5)
                                            .color(colors.accent)
                                            .show();
                                        ui.text("Open")
                                            .size(9.5)
                                            .color(colors.accent)
                                            .show();
                                    });
                                if open_btn.hovered {
                                    ui.cursor_icon = CursorIcon::Pointer;
                                }
                                if open_btn.clicked {
                                    **target_open = Some(p.clone());
                                }
                            }
                        });
                });

            // Content preview container
            ui.container()
                .fill_x()
                .padding(5.0, 7.0, 5.0, 7.0)
                .radius_all(3.5)
                .bg(colors.bg_sidebar.with_alpha(0.6))
                .border((colors.border.with_alpha(0.3), 1.0))
                .column()
                .gap(1.5)
                .show(|ui| {
                    let total_lines = content.lines().count();
                    let max_display_lines = 25;
                    for (i, line) in content.lines().take(max_display_lines).enumerate() {
                        ui.container()
                            .fill_x()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text(&format!("{:>3}", i + 1))
                                    .size(9.5)
                                    .font_family("monospace")
                                    .color(colors.text_muted.with_alpha(0.5))
                                    .show();
                                ui.text(line)
                                    .size(10.0)
                                    .font_family("monospace")
                                    .color(colors.text_primary)
                                    .wrap(zenthra_text::prelude::TextWrap::Word)
                                    .show();
                            });
                    }
                    if total_lines > max_display_lines {
                        ui.text(&format!("... ({} more lines)", total_lines - max_display_lines))
                            .size(9.5)
                            .color(colors.text_muted)
                            .show();
                    }
                });
        });
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
