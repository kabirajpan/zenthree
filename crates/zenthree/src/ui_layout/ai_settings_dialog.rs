use crate::state::{
    AgentMode, AiInputTone, AiMessageBg, AiModel, AiPresetMode, AiSettingsTab, AiTextSize,
    AiThinkingBudget, SelectedAgent, ZenthreeState,
};
use crate::theme::ThemeColors;
use zenthra::{use_context, Align, Color, CursorIcon, FontWeight, Id, Placement, PopoverAlign, Ui};

const AG_SETTINGS_MODEL_BTN_ID: Id = Id::from_u64(999999815);

static ANTIGRAVITY_LOGIN_RUNNING: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);
static ANTIGRAVITY_LOGIN_ERROR: std::sync::Mutex<Option<String>> =
    std::sync::Mutex::new(None);

pub fn draw_ai_settings_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.ai_settings.open {
        return;
    }

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let is_light = colors.bg_base.r > 0.5;

    // Generous, premium floating window dimensions
    let win_w = 800.0;
    let win_h = 560.0;
    let win_radius = 8.0;

    let viewport = ui.current_viewport;
    let center_x = viewport.origin.x + (viewport.size.width - win_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height - win_h) / 2.0;

    // 1. Semi-transparent backdrop to dim screen in the overlays pass (pure draw command, NO occluding container)
    ui.overlays.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [viewport.origin.x, viewport.origin.y],
            size: [viewport.size.width, viewport.size.height],
            color: [0.0, 0.0, 0.0, 0.50],
            radius: [0.0; 4],
            ..Default::default()
        },
    }));

    // Escape key handling to close dialog
    let mut close_on_esc = false;
    ui.input_events.retain(|event| {
        if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
            if *key == winit::keyboard::KeyCode::Escape {
                close_on_esc = true;
                return false;
            }
        }
        true
    });

    let modal_id = Id::from_u64(999999812);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_window_id = Some(modal_id);
    let prev_win_id = ui.current_window_id;
    ui.current_window_id = Some(modal_id);
    ui.record_layout(modal_id, zenthra_core::Rect::new(center_x, center_y, win_w, win_h));

    if close_on_esc {
        if state.ai_settings.antigravity_model_dropdown_open {
            state.ai_settings.antigravity_model_dropdown_open = false;
            ui.needs_redraw = true;
            return;
        }
        state.ai_settings.open = false;
        ui.interaction_state.remove(&modal_key);
        state.save_persistent_state();
        ui.needs_redraw = true;
        return;
    }

    // Outside click detection: dismiss only if user clicked OUTSIDE the dialog card bounds
    if state.ai_settings.just_opened {
        state.ai_settings.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, win_w, win_h);
        if ui.clicked && !mouse_in_dialog {
            if state.ai_settings.antigravity_model_dropdown_open {
                state.ai_settings.antigravity_model_dropdown_open = false;
                ui.consume_click();
                ui.needs_redraw = true;
                return;
            }
            state.ai_settings.open = false;
            ui.interaction_state.remove(&modal_key);
            ui.consume_click();
            state.save_persistent_state();
            ui.needs_redraw = true;
            return;
        }
    }

    // 2. Centered Floating Dialog Card
    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 1.2).clamp(0.40, 0.95))
    } else {
        colors.bg_sidebar
    };

    let mut card = ui.container()
        .id(Id::from_u64(999999711))
        .absolute(center_x, center_y)
        .overlay()
        .width(win_w)
        .height(win_h)
        .bg(card_bg)
        .border(colors.border.with_alpha(0.40), 1.0)
        .radius_all(win_radius)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.40), 0.0, 10.0, 28.0);

    if state.glassmorphism {
        card = card.backdrop_blur(state.blur_strength.max(4.0));
    }

    card.column()
        .clip(true)
        .show(|ui| {
            // Header bar
            ui.container()
                .full_width()
                .height(44.0)
                .padding(0.0, 18.0, 0.0, 18.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .bg(colors.bg_base.with_alpha(0.55))
                .radius_top(win_radius)
                .show(|ui| {
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(10.0)
                        .show(|ui| {
                            ui.text("\u{f005}") // star/sparkle
                                .size(14.0)
                                .color(colors.accent)
                                .show();
                            ui.text("AI & Agent Settings")
                                .size(13.5)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    let close_btn = ui.container()
                        .clickable(true)
                        .padding(5.0, 9.0, 5.0, 9.0)
                        .radius_all(5.0)
                        .hover_bg(colors.highlight)
                        .show(|ui| {
                            ui.text("\u{f00d}") // fa-times
                                .size(12.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    if close_btn.hovered {
                        ui.cursor_icon = CursorIcon::Pointer;
                    }
                    if close_btn.clicked {
                        state.ai_settings.open = false;
                        state.ai_settings.antigravity_model_dropdown_open = false;
                        state.save_persistent_state();
                        ui.needs_redraw = true;
                    }
                });

            // Header bottom border
            ui.container()
                .full_width()
                .height(1.0)
                .bg(colors.border.with_alpha(0.25))
                .show(|_| {});

            // Body: Left Sidebar + Right Content Area
            let body_h = win_h - 45.0;
            let sidebar_w = 180.0;
            let content_w = win_w - sidebar_w - 1.0;

            ui.container()
                .full_width()
                .height(body_h)
                .row()
                .show(|ui| {
                    // Left navigation tabs sidebar
                    ui.container()
                        .width(sidebar_w)
                        .height(body_h)
                        .bg(colors.bg_base.with_alpha(0.35))
                        .radius_bottom_left(win_radius)
                        .padding(12.0, 8.0, 12.0, 8.0)
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            for tab in &AiSettingsTab::ALL {
                                let is_active = state.ai_settings.active_tab == *tab;
                                let tab_btn = ui.container()
                                    .full_width()
                                    .clickable(true)
                                    .padding(5.5, 10.0, 5.5, 10.0)
                                    .radius_all(5.0)
                                    .bg(if is_active { colors.highlight.with_alpha(0.06) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(8.0)
                                    .show(|ui| {
                                        ui.text(tab.icon())
                                            .size(12.0)
                                            .color(if is_active { colors.accent } else { colors.text_muted })
                                            .show();
                                        ui.text(tab.display_name())
                                            .size(12.0)
                                            .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_active { colors.text_primary } else { colors.text_muted })
                                            .show();
                                    });

                                if tab_btn.hovered {
                                    ui.cursor_icon = CursorIcon::Pointer;
                                }
                                if tab_btn.clicked {
                                    state.ai_settings.active_tab = *tab;
                                    state.ai_settings.antigravity_model_dropdown_open = false;
                                    ui.needs_redraw = true;
                                }
                            }
                        });

                    // Vertical divider
                    ui.container()
                        .width(1.0)
                        .height(body_h)
                        .bg(colors.border.with_alpha(0.25))
                        .show(|_| {});

                    // Right Content Area
                    let scroll_id = Id::from_u64(999999715);
                    ui.container()
                        .id(scroll_id)
                        .width(content_w)
                        .height(body_h)
                        .clip(true)
                        .radius_bottom_right(win_radius)
                        .padding(16.0, 22.0, 16.0, 22.0)
                        .scroll_y(true)
                        .column()
                        .gap(12.0)
                        .show(|ui| {
                            match state.ai_settings.active_tab {
                                AiSettingsTab::Appearance => {
                                    draw_appearance_tab(ui, state, &colors, is_light);
                                }
                                AiSettingsTab::General => {
                                    draw_general_tab(ui, state, &colors);
                                }
                                AiSettingsTab::ApiKeys => {
                                    draw_api_keys_tab(ui, state, &colors);
                                }
                                AiSettingsTab::ExternalAgents => {
                                    draw_external_agents_tab(ui, state, &colors);
                                }
                                AiSettingsTab::Mcp => {
                                    draw_mcp_tab(ui, state, &colors);
                                }
                                AiSettingsTab::Model => {
                                    draw_model_tab(ui, state, &colors);
                                }
                                AiSettingsTab::Customization => {
                                    draw_customization_tab(ui, state, &colors);
                                }
                            }
                        });
                });
        });

    // NOTE: The Antigravity model popover is rendered from main.rs at the top level
    // (after all dialog content) so it correctly draws on top of the dialog card.

    ui.current_window_id = prev_win_id;
}

// ─────────────────────────────────────────────────────────────────────────────
// 1. Appearance Tab (REAL, WORKING SETTINGS)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_appearance_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors, _is_light: bool) {
    // Section Title & Description
    ui.container()
        .full_width()
        .column()
        .gap(3.0)
        .show(|ui| {
            ui.text("Appearance & Styling")
                .size(14.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();
            ui.text("Customize contrast, outlines, and drop shadows of the AI assistant panel.")
                .size(11.0)
                .color(colors.text_muted)
                .show();
        });

    ui.spacing(2.0);

    // ── Row 1: System Default vs Custom Preset Mode ──
    let is_sys_default = state.ai_settings.is_system_default();

    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Theme Preset Mode")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text(if is_sys_default {
                        "System default active (options below are grayed out)"
                    } else {
                        "Custom mode active (options below are unlocked)"
                    })
                    .size(10.5)
                    .color(if is_sys_default { colors.accent } else { colors.text_muted })
                    .show();
                });

            // Segmented pills with generous padding so edges never get cut off
            ui.container()
                .row()
                .gap(4.0)
                .padding(3.0, 4.0, 3.0, 4.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .show(|ui| {
                    let modes = [
                        (AiPresetMode::SystemDefault, "System Default"),
                        (AiPresetMode::Custom, "Custom"),
                    ];
                    for (mode, label) in modes {
                        let is_active = state.ai_settings.preset_mode == mode;
                        let btn = ui.container()
                            .clickable(true)
                            .padding(5.0, 12.0, 5.0, 12.0)
                            .radius_all(4.0)
                            .bg(if is_active { colors.accent } else { Color::TRANSPARENT })
                            .hover_bg(if is_active { colors.accent } else { colors.highlight })
                            .show(|ui| {
                                ui.text(label)
                                    .size(10.5)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active { Color::WHITE } else { colors.text_muted })
                                    .show();
                            });
                        if btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if btn.clicked {
                            state.ai_settings.preset_mode = mode;
                            ui.needs_redraw = true;
                        }
                    }
                });
        });

    draw_settings_divider(ui, colors);

    // If System Default is ON, all controls below are rendered grayed out (disabled)
    let is_locked = is_sys_default;
    let label_alpha = if is_locked { 0.45 } else { 1.0 };
    let sub_alpha = if is_locked { 0.35 } else { 1.0 };

    // ── Row 2: Input Box Outline (Official Toggle Widget) ──
    let mut locked_outline = state.ai_settings.effective_input_outline();
    let outline_ref = if is_locked { &mut locked_outline } else { &mut state.ai_settings.input_outline };
    if draw_toggle_row_inline(
        ui,
        999999721,
        "Input Box Outline",
        "Display a subtle perimeter border around the input dock",
        outline_ref,
        is_locked,
        label_alpha,
        sub_alpha,
        colors,
    ) {
        ui.needs_redraw = true;
    }

    draw_settings_divider(ui, colors);

    // ── Row 3: Drop Shadow (Official Toggle Widget) ──
    let mut locked_shadow = state.ai_settings.effective_input_shadow();
    let shadow_ref = if is_locked { &mut locked_shadow } else { &mut state.ai_settings.input_shadow };
    if draw_toggle_row_inline(
        ui,
        999999722,
        "Drop Shadow",
        "Render ambient elevation drop shadow beneath the input box",
        shadow_ref,
        is_locked,
        label_alpha,
        sub_alpha,
        colors,
    ) {
        ui.needs_redraw = true;
    }

    draw_settings_divider(ui, colors);

    // ── Row 4: Input Box Color Tone ──
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Input Box Tone")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary.with_alpha(label_alpha))
                        .show();
                    ui.text("Fill contrast tone: Dark (#141416), Charcoal (#202126), Bright (#2c2d34)")
                        .size(10.5)
                        .color(colors.text_muted.with_alpha(sub_alpha))
                        .show();
                });

            ui.container()
                .row()
                .gap(4.0)
                .padding(3.0, 4.0, 3.0, 4.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(if is_locked { 0.4 } else { 1.0 }))
                .show(|ui| {
                    let current_tone = state.ai_settings.effective_input_tone();
                    let tones = [
                        (AiInputTone::Dark, "Dark"),
                        (AiInputTone::Charcoal, "Charcoal"),
                        (AiInputTone::Bright, "Bright"),
                    ];
                    for (tone, label) in tones {
                        let is_active = current_tone == tone;
                        let active_bg = if is_locked {
                            colors.accent.with_alpha(0.4)
                        } else {
                            colors.accent
                        };
                        let btn = ui.container()
                            .clickable(!is_locked)
                            .padding(5.0, 12.0, 5.0, 12.0)
                            .radius_all(4.0)
                            .bg(if is_active { active_bg } else { Color::TRANSPARENT })
                            .hover_bg(if is_locked {
                                Color::TRANSPARENT
                            } else if is_active {
                                colors.accent
                            } else {
                                colors.highlight
                            })
                            .show(|ui| {
                                ui.text(label)
                                    .size(10.5)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active {
                                        Color::WHITE.with_alpha(if is_locked { 0.6 } else { 1.0 })
                                    } else {
                                        colors.text_muted.with_alpha(sub_alpha)
                                    })
                                    .show();
                            });

                        if !is_locked && btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if !is_locked && btn.clicked {
                            state.ai_settings.input_tone = tone;
                            ui.needs_redraw = true;
                        }
                    }
                });
        });

    draw_settings_divider(ui, colors);

    // ── Row 5: Message Area Background ──
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Message Area Background")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary.with_alpha(label_alpha))
                        .show();
                    ui.text("Deep dark backdrop (#121212) vs sidebar match (#181818)")
                        .size(10.5)
                        .color(colors.text_muted.with_alpha(sub_alpha))
                        .show();
                });

            ui.container()
                .row()
                .gap(4.0)
                .padding(3.0, 4.0, 3.0, 4.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(if is_locked { 0.4 } else { 1.0 }))
                .show(|ui| {
                    let current_bg = state.ai_settings.effective_message_bg();
                    let bgs = [
                        (AiMessageBg::DeepDark, "Deep Dark"),
                        (AiMessageBg::Sidebar, "Sidebar Match"),
                    ];
                    for (bg, label) in bgs {
                        let is_active = current_bg == bg;
                        let active_bg = if is_locked {
                            colors.accent.with_alpha(0.4)
                        } else {
                            colors.accent
                        };
                        let btn = ui.container()
                            .clickable(!is_locked)
                            .padding(5.0, 12.0, 5.0, 12.0)
                            .radius_all(4.0)
                            .bg(if is_active { active_bg } else { Color::TRANSPARENT })
                            .hover_bg(if is_locked {
                                Color::TRANSPARENT
                            } else if is_active {
                                colors.accent
                            } else {
                                colors.highlight
                            })
                            .show(|ui| {
                                ui.text(label)
                                    .size(10.5)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active {
                                        Color::WHITE.with_alpha(if is_locked { 0.6 } else { 1.0 })
                                    } else {
                                        colors.text_muted.with_alpha(sub_alpha)
                                    })
                                    .show();
                            });

                        if !is_locked && btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if !is_locked && btn.clicked {
                            state.ai_settings.message_bg = bg;
                            ui.needs_redraw = true;
                        }
                    }
                });
        });

    draw_settings_divider(ui, colors);

    // ── Row 6: Prompt Font Size ──
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Prompt Font Size")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary.with_alpha(label_alpha))
                        .show();
                    ui.text("Text sizing in the input prompt textarea")
                        .size(10.5)
                        .color(colors.text_muted.with_alpha(sub_alpha))
                        .show();
                });

            ui.container()
                .row()
                .gap(4.0)
                .padding(3.0, 4.0, 3.0, 4.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(if is_locked { 0.4 } else { 1.0 }))
                .show(|ui| {
                    let current_sz = state.ai_settings.effective_text_size();
                    let sizes = [
                        (AiTextSize::Compact, AiTextSize::Compact.display_name()),
                        (AiTextSize::Regular, AiTextSize::Regular.display_name()),
                        (AiTextSize::Large, AiTextSize::Large.display_name()),
                    ];
                    for (size, label) in sizes {
                        let is_active = current_sz == size;
                        let active_bg = if is_locked {
                            colors.accent.with_alpha(0.4)
                        } else {
                            colors.accent
                        };
                        let btn = ui.container()
                            .clickable(!is_locked)
                            .padding(5.0, 11.0, 5.0, 11.0)
                            .radius_all(4.0)
                            .bg(if is_active { active_bg } else { Color::TRANSPARENT })
                            .hover_bg(if is_locked {
                                Color::TRANSPARENT
                            } else if is_active {
                                colors.accent
                            } else {
                                colors.highlight
                            })
                            .show(|ui| {
                                ui.text(label)
                                    .size(10.5)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active {
                                        Color::WHITE.with_alpha(if is_locked { 0.6 } else { 1.0 })
                                    } else {
                                        colors.text_muted.with_alpha(sub_alpha)
                                    })
                                    .show();
                            });

                        if !is_locked && btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if !is_locked && btn.clicked {
                            state.ai_settings.text_size = size;
                            ui.needs_redraw = true;
                        }
                    }
                });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// 2. General Tab (UI Only, using official toggle widgets)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_general_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .full_width()
        .column()
        .gap(3.0)
        .show(|ui| {
            ui.text("General Preferences")
                .size(14.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();
            ui.text("Agent interaction behavior and notification parameters")
                .size(11.0)
                .color(colors.text_muted)
                .show();
        });

    ui.spacing(2.0);

    draw_toggle_row_inline(
        ui,
        999999731,
        "Auto-Scroll Stream",
        "Automatically scroll to bottom as agent produces new response tokens",
        &mut state.ai_settings.auto_scroll,
        false,
        1.0,
        1.0,
        colors,
    );

    draw_settings_divider(ui, colors);

    draw_toggle_row_inline(
        ui,
        999999732,
        "Show Reasoning & Thoughts",
        "Expand collapsible model thinking processes in the chat transcript",
        &mut state.ai_settings.show_thoughts,
        false,
        1.0,
        1.0,
        colors,
    );

    draw_settings_divider(ui, colors);

    draw_toggle_row_inline(
        ui,
        999999733,
        "Sound Notification",
        "Play subtle sound when a long-running agent task finishes",
        &mut state.ai_settings.sound_notifications,
        false,
        1.0,
        1.0,
        colors,
    );

    draw_settings_divider(ui, colors);

    draw_toggle_row_inline(
        ui,
        999999734,
        "Auto-Approve Read Tools",
        "Automatically grant permission for read-only filesystem searches",
        &mut state.ai_settings.auto_approve_read,
        false,
        1.0,
        1.0,
        colors,
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// 3. Model Tab (UI Only)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_model_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .full_width()
        .column()
        .gap(3.0)
        .show(|ui| {
            ui.text("Model Configuration")
                .size(14.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();
            ui.text("Default model engine, temperature, and reasoning budget")
                .size(11.0)
                .color(colors.text_muted)
                .show();
        });

    ui.spacing(2.0);

    // Row 1: Default AI Model inline picker
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Default AI Model")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Active intelligence provider for workspace sessions")
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });

            let model_pill = ui.container()
                .clickable(true)
                .padding(5.0, 12.0, 5.0, 12.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .border(colors.border.with_alpha(0.5), 0.75)
                .row()
                .valign(Align::Center)
                .gap(8.0)
                .show(|ui| {
                    ui.text(state.selected_ai_model.display_name())
                        .size(11.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text("\u{f078}") // fa-chevron-down
                        .size(8.0)
                        .color(colors.text_muted)
                        .show();
                });

            if model_pill.hovered {
                ui.cursor_icon = CursorIcon::Pointer;
            }
            if model_pill.clicked {
                let all = AiModel::ALL;
                let idx = all.iter().position(|m| *m == state.selected_ai_model).unwrap_or(0);
                state.selected_ai_model = all[(idx + 1) % all.len()];
                ui.needs_redraw = true;
            }
        });

    draw_settings_divider(ui, colors);

    // Row 2: Temperature Slider with Steppers
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Temperature")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Controls response randomness vs deterministic accuracy")
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });

            ui.container()
                .row()
                .valign(Align::Center)
                .gap(5.0)
                .show(|ui| {
                    let minus_btn = ui.container()
                        .clickable(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .bg(colors.highlight)
                        .hover_bg(colors.highlight)
                        .valign(Align::Center)
                        .halign(Align::Center)
                        .show(|ui| {
                            ui.text("-")
                                .size(13.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });
                    if minus_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if minus_btn.clicked {
                        state.ai_settings.temperature = ((state.ai_settings.temperature - 0.05) * 100.0).round() / 100.0;
                        state.ai_settings.temperature = state.ai_settings.temperature.max(0.0);
                        ui.needs_redraw = true;
                    }

                    let mut temp_val = state.ai_settings.temperature;
                    let slider_resp = ui.slider(&mut temp_val, "ai_temperature_slider")
                        .range(0.0, 1.0)
                        .step(0.05)
                        .track_size(110.0, 4.0)
                        .track_radius(2.0, 2.0, 2.0, 2.0)
                        .track_color(colors.highlight)
                        .thumb_size(12.0, 12.0)
                        .thumb_radius(6.0, 6.0, 6.0, 6.0)
                        .thumb_color(colors.accent)
                        .show();

                    if (temp_val - state.ai_settings.temperature).abs() > 0.001 || slider_resp.clicked || slider_resp.pressed {
                        state.ai_settings.temperature = temp_val;
                        ui.needs_redraw = true;
                    }

                    let plus_btn = ui.container()
                        .clickable(true)
                        .width(22.0)
                        .height(22.0)
                        .radius_all(4.0)
                        .bg(colors.highlight)
                        .hover_bg(colors.highlight)
                        .valign(Align::Center)
                        .halign(Align::Center)
                        .show(|ui| {
                            ui.text("+")
                                .size(13.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });
                    if plus_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if plus_btn.clicked {
                        state.ai_settings.temperature = ((state.ai_settings.temperature + 0.05) * 100.0).round() / 100.0;
                        state.ai_settings.temperature = state.ai_settings.temperature.min(1.0);
                        ui.needs_redraw = true;
                    }

                    ui.container()
                        .width(36.0)
                        .halign(Align::Right)
                        .show(|ui| {
                            ui.text(&format!("{:.2}", state.ai_settings.temperature))
                                .size(11.5)
                                .weight(FontWeight::Bold)
                                .color(colors.accent)
                                .show();
                        });
                });
        });

    draw_settings_divider(ui, colors);

    // Row 3: Extended Thinking Budget inline
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Thinking Token Budget")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Max tokens allowed for step-by-step reasoning")
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });

            ui.container()
                .row()
                .gap(4.0)
                .padding(3.0, 4.0, 3.0, 4.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .show(|ui| {
                    for b in &AiThinkingBudget::ALL {
                        let is_active = state.ai_settings.thinking_budget == *b;
                        let btn = ui.container()
                            .clickable(true)
                            .padding(4.0, 10.0, 4.0, 10.0)
                            .radius_all(4.0)
                            .bg(if is_active { colors.accent } else { Color::TRANSPARENT })
                            .hover_bg(if is_active { colors.accent } else { colors.highlight })
                            .show(|ui| {
                                ui.text(b.display_name())
                                    .size(10.5)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active { Color::WHITE } else { colors.text_muted })
                                    .show();
                            });
                        if btn.clicked {
                            state.ai_settings.thinking_budget = *b;
                            ui.needs_redraw = true;
                        }
                    }
                });
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// 3b. External Agents & Google Antigravity Tab
// ─────────────────────────────────────────────────────────────────────────────

// ─────────────────────────────────────────────────────────────────────────────
// 3a. API Keys & Subscriptions Tab (ZENE Native Agent)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_api_keys_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    use agent::provider::{ZenthreeConfig, KNOWN_PROVIDERS};

    let cfg = ZenthreeConfig::load();
    let config_path = ZenthreeConfig::config_path();
    let config_path_str = config_path.to_string_lossy().to_string();

    // ── Header row ──
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container().column().gap(2.0).show(|ui| {
                ui.text("API Keys")
                    .size(14.0)
                    .weight(FontWeight::Bold)
                    .color(colors.text_primary)
                    .show();
                ui.text(&config_path_str)
                    .size(9.5)
                    .color(colors.text_muted)
                    .show();
            });

            let edit_btn = ui.container()
                .clickable(true)
                .padding(5.0, 10.0, 5.0, 10.0)
                .radius_all(5.0)
                .bg(colors.highlight)
                .border(colors.border.with_alpha(0.5), 0.75)
                .show(|ui| {
                    ui.text("Edit config.json ↗")
                        .size(10.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                });
            if edit_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
            if edit_btn.clicked {
                if let Ok(path_str) = config_path.to_str().ok_or("") {
                    let _ = opener::open(path_str);
                }
            }
        });

    ui.spacing(4.0);

    // ── Status message ──
    if let Some(msg) = &state.ai_settings.api_key_save_msg.clone() {
        let is_err = msg.starts_with("Error");
        let accent = if is_err { Color::rgb(0.9, 0.3, 0.3) } else { colors.accent };
        ui.container()
            .full_width()
            .padding(6.0, 10.0, 6.0, 10.0)
            .radius_all(5.0)
            .bg(accent.with_alpha(0.12))
            .border(accent.with_alpha(0.4), 0.75)
            .show(|ui| {
                ui.text(msg).size(10.5).color(accent).weight(FontWeight::Medium).show();
            });
        ui.spacing(2.0);
    }

    // ── Known providers (dynamic) ──
    let known_provider_names: Vec<String> = KNOWN_PROVIDERS.iter().map(|p| p.name.to_string()).collect();

    for (i, kp) in KNOWN_PROVIDERS.iter().enumerate() {
        let current_key = cfg.get_provider_key(kp.name);
        let _is_set = current_key.is_some();
        let input_id = format!("api_key_input_{}", kp.name);

        // Separator
        if i > 0 {
            ui.container()
                .full_width()
                .height(1.0)
                .bg(colors.border.with_alpha(0.25))
                .show(|_| {});
        }

        // Provider row
        ui.container()
            .full_width()
            .padding(0.0, 0.0, 6.0, 0.0)
            .column()
            .gap(6.0)
            .show(|ui| {
                // Name + action row
                ui.container()
                    .full_width()
                    .row()
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .show(|ui| {
                        ui.text(kp.display_name)
                            .size(12.0)
                            .weight(FontWeight::Medium)
                            .color(colors.text_primary)
                            .show();
                    });

                // Key input/display row
                ui.container()
                    .full_width()
                    .row()
                    .valign(Align::Center)
                    .halign(Align::SpaceBetween)
                    .gap(8.0)
                    .show(|ui| {
                        if let Some(ref key) = current_key {
                            let masked = if key.len() > 8 {
                                format!("{}••••••••••••{}", &key[..4], &key[key.len() - 4..])
                            } else {
                                "••••••••".to_string()
                            };
                            ui.text(&masked)
                                .size(10.5)
                                .color(colors.text_muted)
                                .show();

                            ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                                let rm_btn = ui.container()
                                    .clickable(true)
                                    .padding(4.0, 9.0, 4.0, 9.0)
                                    .radius_all(5.0)
                                    .bg(colors.highlight)
                                    .border(colors.border.with_alpha(0.5), 0.75)
                                    .show(|ui| {
                                        ui.text("Remove")
                                            .size(10.0)
                                            .color(colors.text_primary)
                                            .show();
                                    });
                                if rm_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if rm_btn.clicked {
                                    let _ = agent::provider::remove_key(kp.name);
                                    state.ai_settings.api_key_save_msg =
                                        Some(format!("{} key removed from config.json", kp.display_name));
                                    ui.needs_redraw = true;
                                }
                            });
                        } else {
                            // Input box
                            let input_text = state.ai_settings.api_key_inputs
                                .entry(kp.name.to_string())
                                .or_insert_with(String::new);

                            ui.text_area(input_text, &input_id)
                                .width(240.0)
                                .height(28.0)
                                .size(10.5)
                                .color(colors.text_primary)
                                .bg(colors.bg_base.with_alpha(0.6))
                                .border(colors.border.with_alpha(0.5), 0.75)
                                .focus_border(colors.accent.with_alpha(0.8), 1.0)
                                .radius(4.0, 4.0, 4.0, 4.0)
                                .padding(4.0, 6.0, 4.0, 6.0)
                                .placeholder(&format!("Paste {} key...", kp.display_name))
                                .placeholder_color(colors.text_muted.with_alpha(0.5))
                                .show();

                            let key_val = state.ai_settings.api_key_inputs
                                .get(kp.name)
                                .map(|s| s.trim().to_string())
                                .unwrap_or_default();
                            let can_save = !key_val.is_empty();

                            let save_btn = ui.container()
                                .clickable(can_save)
                                .padding(5.0, 11.0, 5.0, 11.0)
                                .radius_all(5.0)
                                .bg(if can_save { colors.accent } else { colors.highlight })
                                .show(|ui| {
                                    ui.text("Save")
                                        .size(10.0)
                                        .weight(FontWeight::Medium)
                                        .color(if can_save { Color::WHITE } else { colors.text_muted })
                                        .show();
                                });
                            if save_btn.hovered && can_save { ui.cursor_icon = CursorIcon::Pointer; }
                            if save_btn.clicked && can_save {
                                let _ = agent::provider::set_key(kp.name, &key_val);
                                state.ai_settings.api_key_inputs.remove(kp.name);
                                state.ai_settings.api_key_save_msg =
                                    Some(format!("{} key saved to config.json", kp.display_name));
                                ui.needs_redraw = true;
                            }
                        }
                    });
            });
    }

    // ── Custom providers from config ──
    let custom_providers: Vec<(String, String)> = cfg.providers.iter()
        .filter(|(name, entry)| {
            !known_provider_names.contains(name) && !entry.key.is_empty()
        })
        .map(|(name, entry)| {
            let masked = if entry.key.len() > 8 {
                format!("{}••••••••••••{}", &entry.key[..4], &entry.key[entry.key.len() - 4..])
            } else {
                "••••••••".to_string()
            };
            (name.clone(), masked)
        })
        .collect();

    if !custom_providers.is_empty() {
        ui.container()
            .full_width()
            .height(1.0)
            .bg(colors.border.with_alpha(0.25))
            .show(|_| {});

        for (name, masked) in &custom_providers {
            ui.container()
                .full_width()
                .padding(0.0, 0.0, 6.0, 0.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container().column().gap(2.0).show(|ui| {
                        ui.text(name)
                            .size(12.0)
                            .weight(FontWeight::Medium)
                            .color(colors.text_primary)
                            .show();
                        ui.text(masked)
                            .size(10.0)
                            .color(colors.text_muted)
                            .show();
                    });

                    let rm_btn = ui.container()
                        .clickable(true)
                        .padding(4.0, 9.0, 4.0, 9.0)
                        .radius_all(5.0)
                        .bg(colors.highlight)
                        .border(colors.border.with_alpha(0.5), 0.75)
                        .show(|ui| {
                            ui.text("Remove").size(10.0).color(colors.text_primary).show();
                        });
                    if rm_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if rm_btn.clicked {
                        let _ = agent::provider::remove_key(name);
                        state.ai_settings.api_key_save_msg =
                            Some(format!("'{}' removed from config.json", name));
                        ui.needs_redraw = true;
                    }
                });
        }
    }

    // ── Divider before + Add ──
    ui.spacing(4.0);
    ui.container()
        .full_width()
        .height(1.0)
        .bg(colors.border.with_alpha(0.25))
        .show(|_| {});
    ui.spacing(4.0);

    // ── + Add Custom Provider ──
    if !state.ai_settings.custom_provider_form_open {
        let add_btn = ui.container()
            .clickable(true)
            .padding(5.0, 10.0, 5.0, 10.0)
            .radius_all(5.0)
            .bg(colors.highlight)
            .border(colors.border.with_alpha(0.4), 0.75)
            .show(|ui| {
                ui.text("+ Add Custom Provider")
                    .size(10.5)
                    .weight(FontWeight::Medium)
                    .color(colors.text_muted)
                    .show();
            });
        if add_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
        if add_btn.clicked {
            state.ai_settings.custom_provider_form_open = true;
            state.ai_settings.custom_provider_name.clear();
            state.ai_settings.custom_provider_key.clear();
            state.ai_settings.custom_provider_url.clear();
        }
    } else {
        // Inline form
        ui.container()
            .full_width()
            .padding(12.0, 14.0, 12.0, 14.0)
            .radius_all(7.0)
            .bg(colors.bg_base)
            .border(colors.accent.with_alpha(0.4), 1.0)
            .column()
            .gap(8.0)
            .show(|ui| {
                ui.text("Add Custom Provider")
                    .size(11.5)
                    .weight(FontWeight::SemiBold)
                    .color(colors.text_primary)
                    .show();

                // Name field
                ui.container().full_width().row().valign(Align::Center).gap(8.0).show(|ui| {
                    ui.text("Name").size(10.0).color(colors.text_muted).max_width(60.0).show();
                    ui.text_area(&mut state.ai_settings.custom_provider_name, "custom_prov_name")
                        .width(340.0)
                        .height(26.0)
                        .size(10.5)
                        .color(colors.text_primary)
                        .bg(colors.bg_base.with_alpha(0.6))
                        .border(colors.border.with_alpha(0.5), 0.75)
                        .focus_border(colors.accent.with_alpha(0.8), 1.0)
                        .radius(4.0, 4.0, 4.0, 4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .placeholder("e.g. openai")
                        .placeholder_color(colors.text_muted.with_alpha(0.5))
                        .show();
                });

                // API Key field
                ui.container().full_width().row().valign(Align::Center).gap(8.0).show(|ui| {
                    ui.text("API Key").size(10.0).color(colors.text_muted).max_width(60.0).show();
                    ui.text_area(&mut state.ai_settings.custom_provider_key, "custom_prov_key")
                        .width(340.0)
                        .height(26.0)
                        .size(10.5)
                        .color(colors.text_primary)
                        .bg(colors.bg_base.with_alpha(0.6))
                        .border(colors.border.with_alpha(0.5), 0.75)
                        .focus_border(colors.accent.with_alpha(0.8), 1.0)
                        .radius(4.0, 4.0, 4.0, 4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .placeholder("sk-...")
                        .placeholder_color(colors.text_muted.with_alpha(0.5))
                        .show();
                });

                // Base URL field
                ui.container().full_width().row().valign(Align::Center).gap(8.0).show(|ui| {
                    ui.text("Base URL").size(10.0).color(colors.text_muted).max_width(60.0).show();
                    ui.text_area(&mut state.ai_settings.custom_provider_url, "custom_prov_url")
                        .width(340.0)
                        .height(26.0)
                        .size(10.5)
                        .color(colors.text_primary)
                        .bg(colors.bg_base.with_alpha(0.6))
                        .border(colors.border.with_alpha(0.5), 0.75)
                        .focus_border(colors.accent.with_alpha(0.8), 1.0)
                        .radius(4.0, 4.0, 4.0, 4.0)
                        .padding(4.0, 6.0, 4.0, 6.0)
                        .placeholder("https://api.openai.com/v1")
                        .placeholder_color(colors.text_muted.with_alpha(0.5))
                        .show();
                });

                // Buttons
                ui.container().full_width().row().valign(Align::Center).halign(Align::Right).gap(6.0).show(|ui| {
                    let cancel_btn = ui.container()
                        .clickable(true)
                        .padding(5.0, 10.0, 5.0, 10.0)
                        .radius_all(5.0)
                        .bg(colors.highlight)
                        .border(colors.border.with_alpha(0.5), 0.75)
                        .show(|ui| {
                            ui.text("Cancel").size(10.0).color(colors.text_muted).show();
                        });
                    if cancel_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if cancel_btn.clicked {
                        state.ai_settings.custom_provider_form_open = false;
                    }

                    let name_ok = !state.ai_settings.custom_provider_name.trim().is_empty();
                    let key_ok = !state.ai_settings.custom_provider_key.trim().is_empty();
                    let url_ok = !state.ai_settings.custom_provider_url.trim().is_empty();
                    let can_add = name_ok && key_ok && url_ok;

                    let add_btn = ui.container()
                        .clickable(can_add)
                        .padding(5.0, 11.0, 5.0, 11.0)
                        .radius_all(5.0)
                        .bg(if can_add { colors.accent } else { colors.highlight })
                        .show(|ui| {
                            ui.text("Add")
                                .size(10.0)
                                .weight(FontWeight::Medium)
                                .color(if can_add { Color::WHITE } else { colors.text_muted })
                                .show();
                        });
                    if add_btn.hovered && can_add { ui.cursor_icon = CursorIcon::Pointer; }
                    if add_btn.clicked && can_add {
                        let name = state.ai_settings.custom_provider_name.trim().to_lowercase();
                        let key = state.ai_settings.custom_provider_key.trim().to_string();
                        let url = state.ai_settings.custom_provider_url.trim().to_string();
                        let mut c = ZenthreeConfig::load();
                        c.add_custom_provider(&name, &key, &url, vec![]);
                        let _ = c.save();
                        state.ai_settings.api_key_save_msg =
                            Some(format!("'{}' added to config.json", name));
                        state.ai_settings.custom_provider_form_open = false;
                        ui.needs_redraw = true;
                    }
                });
            });
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3b. External Agents & ACP Registry Tab
// ─────────────────────────────────────────────────────────────────────────────

fn draw_external_agents_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .full_width()
        .column()
        .gap(3.0)
        .show(|ui| {
            ui.text("External Agents & ACP")
                .size(14.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();
            ui.text("Connect autonomous external agents via Google ID OAuth or the Agent Client Protocol (ACP)")
                .size(11.0)
                .color(colors.text_muted)
                .show();
        });

    ui.spacing(4.0);

    let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
    let maybe_token = auth.load_token();
    let is_connected = maybe_token.is_some();
    let is_antigravity_active = state.selected_agent == SelectedAgent::Antigravity;
    let _is_light = colors.bg_base.r > 0.5;

    // ── 1. Google Antigravity Card ──
    ui.container()
        .full_width()
        .padding(14.0, 16.0, 14.0, 16.0)
        .radius_all(8.0)
        .bg(colors.bg_base)
        .border(if is_antigravity_active { colors.accent.with_alpha(0.7) } else { colors.border.with_alpha(0.6) }, 1.0)
        .column()
        .gap(12.0)
        .show(|ui| {
            // Header row: Logo + Name + Status Badge
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container().row().valign(Align::Center).gap(10.0).show(|ui| {
                        // Logo pill
                        ui.container()
                            .width(32.0)
                            .height(32.0)
                            .radius_all(6.0)
                            .bg(colors.accent.with_alpha(0.18))
                            .valign(Align::Center)
                            .halign(Align::Center)
                            .show(|ui| {
                                ui.text("⚡")
                                    .size(14.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.accent)
                                    .show();
                            });
                        ui.container().column().gap(2.0).show(|ui| {
                            ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                                ui.text("Google Antigravity")
                                    .size(13.0)
                                    .weight(FontWeight::SemiBold)
                                    .color(colors.text_primary)
                                    .show();
                                if is_antigravity_active {
                                    ui.container()
                                        .padding(1.5, 6.0, 1.5, 6.0)
                                        .radius_all(4.0)
                                        .bg(colors.accent.with_alpha(0.2))
                                        .show(|ui| {
                                            ui.text("ACTIVE IN CHAT")
                                                .size(8.5)
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                }
                            });
                            ui.text("Google Developer Infrastructure via Agent Client Protocol (ACP)")
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();
                        });
                    });

                    // Badge
                    let (badge_bg, badge_fg, badge_text) = if is_connected {
                        (Color::rgb(0.15, 0.55, 0.25), Color::rgb(0.2, 0.9, 0.4), "Connected")
                    } else {
                        (colors.highlight, colors.text_muted, "Not Connected")
                    };

                    ui.container()
                        .padding(3.0, 9.0, 3.0, 9.0)
                        .radius_all(10.0)
                        .bg(badge_bg.with_alpha(0.2))
                        .border(badge_bg.with_alpha(0.6), 0.75)
                        .show(|ui| {
                            ui.text(badge_text)
                                .size(10.0)
                                .weight(FontWeight::Medium)
                                .color(badge_fg)
                                .show();
                        });
                });

            // Description
            let description_width = ui.available_width.max(1.0);
            ui.text("Google Antigravity autonomous coding agent running via the Agent Client Protocol (ACP). Handles its own authentication and provides access to Gemini 3.8 Flash, Gemini 3.7, and Gemini 3.1 Pro.")
                .size(10.8)
                .color(colors.text_muted)
                .max_width(description_width)
                .auto_wrap()
                .show();

            // ── Controls row: Model Engine on Left, User & Actions on Right ──
            ui.container()
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .border(colors.border.with_alpha(0.5), 0.75)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // LEFT: Model Engine (fixed width so right side never overlaps)
                    ui.container()
                        .width(210.0)
                        .column()
                        .gap(5.0)
                        .show(|ui| {
                            ui.text("Model Engine")
                                .size(11.5)
                                .weight(FontWeight::Medium)
                                .color(colors.text_primary)
                                .show();

                            draw_antigravity_settings_model_button(ui, state, colors);
                        });

                    // RIGHT: User icon + email on top, Logout / Switch or Sign In below
                    ui.container()
                        .max_width(220.0)
                        .column()
                        .gap(8.0)
                        .halign(Align::Right)
                        .show(|ui| {
                            if let Some(token) = &maybe_token {
                                // User icon + email
                                ui.container().row().valign(Align::Center).gap(7.0).show(|ui| {
                                    ui.container()
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(11.0)
                                        .bg(colors.accent.with_alpha(0.18))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{f007}") // fa-user
                                                .size(9.5)
                                                .color(colors.accent)
                                                .show();
                                        });

                                    let email_str = token.account_email.as_deref().unwrap_or("Signed in via Google OAuth");
                                    ui.text(email_str)
                                        .size(10.5)
                                        .weight(FontWeight::SemiBold)
                                        .color(colors.text_primary)
                                        .max_width(170.0)
                                        .ellipsis(true)
                                        .show();
                                });

                                // Below: Logout and Switch buttons
                                ui.container().row().valign(Align::Center).gap(8.0).show(|ui| {
                                    let logout_btn = ui.container()
                                        .clickable(true)
                                        .padding(4.5, 12.0, 4.5, 12.0)
                                        .radius_all(5.0)
                                        .bg(colors.highlight)
                                        .border(colors.border.with_alpha(0.5), 0.75)
                                        .show(|ui| {
                                            ui.text("Log Out")
                                                .size(10.5)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if logout_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if logout_btn.clicked {
                                        auth.clear_token();
                                        if state.selected_agent == SelectedAgent::Antigravity {
                                            state.selected_agent = SelectedAgent::Zene;
                                        }
                                        ui.needs_redraw = true;
                                    }

                                    let switch_btn = ui.container()
                                        .clickable(!is_antigravity_active)
                                        .padding(4.5, 12.0, 4.5, 12.0)
                                        .radius_all(5.0)
                                        .bg(if is_antigravity_active { colors.accent.with_alpha(0.35) } else { colors.accent })
                                        .show(|ui| {
                                            ui.text(if is_antigravity_active { "Active in Chat" } else { "Switch to Agent" })
                                                .size(10.5)
                                                .weight(FontWeight::Medium)
                                                .color(Color::WHITE)
                                                .show();
                                        });
                                    if !is_antigravity_active && switch_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if !is_antigravity_active && switch_btn.clicked {
                                        state.selected_agent = SelectedAgent::Antigravity;
                                        ui.needs_redraw = true;
                                    }
                                });
                            } else {
                                let is_signing_in = ANTIGRAVITY_LOGIN_RUNNING
                                    .load(std::sync::atomic::Ordering::SeqCst);
                                let login_error = ANTIGRAVITY_LOGIN_ERROR
                                    .lock()
                                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                                    .clone();

                                ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                                    ui.container()
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(11.0)
                                        .bg(colors.border.with_alpha(0.3))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("\u{f007}") // fa-user
                                                .size(9.5)
                                                .color(colors.text_muted)
                                                .show();
                                        });

                                    ui.text("Not signed in")
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                                if let Some(error) = &login_error {
                                    ui.text(error)
                                        .size(9.0)
                                        .color(Color::from_hex(0xe65050))
                                        .show();
                                }

                                if is_signing_in {
                                    ui.request_redraw_after(std::time::Duration::from_millis(250));
                                }

                                let login_btn = ui.button(
                                    if is_signing_in { "Authenticating..." } else { "Sign In via Antigravity" },
                                )
                                    .id(Id::from_u64(999999834))
                                    .padding(5.0, 14.0, 5.0, 14.0)
                                    .radius_all(5.0)
                                    .bg(if is_signing_in { colors.accent.with_alpha(0.4) } else { colors.accent })
                                    .hover_bg(colors.accent.with_alpha(0.85))
                                    .text_color(Color::WHITE)
                                    .size(10.5)
                                    .show();
                                if login_btn.hovered && !is_signing_in {
                                    ui.cursor_icon = CursorIcon::Pointer;
                                }
                                if login_btn.clicked && !is_signing_in {
                                    ANTIGRAVITY_LOGIN_ERROR
                                        .lock()
                                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                                        .take();
                                    ANTIGRAVITY_LOGIN_RUNNING
                                        .store(true, std::sync::atomic::Ordering::SeqCst);
                                    std::thread::spawn(|| {
                                        let auth = agent::external_provider::AntigravityAuthManager::with_default_config();
                                        let result = auth.login_interactive().and_then(|()| {
                                            if auth.load_token().is_some() {
                                                Ok(())
                                            } else {
                                                Err("Antigravity login finished without saving credentials. Check that the Antigravity ACP agent is installed and try again.".to_string())
                                            }
                                        });
                                        if let Err(error) = result {
                                            *ANTIGRAVITY_LOGIN_ERROR
                                                .lock()
                                                .unwrap_or_else(std::sync::PoisonError::into_inner) =
                                                Some(error);
                                        }
                                        ANTIGRAVITY_LOGIN_RUNNING
                                            .store(false, std::sync::atomic::Ordering::SeqCst);
                                    });
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });
        });

    draw_settings_divider(ui, colors);

    // ── Extensible Registry: Other Agents (Future Ready) ──
    ui.container()
        .full_width()
        .column()
        .gap(3.0)
        .show(|ui| {
            ui.text("Additional External Agents")
                .size(12.0)
                .weight(FontWeight::SemiBold)
                .color(colors.text_primary)
                .show();
            ui.text("Add more autonomous agents via the open Agent Client Protocol")
                .size(10.0)
                .color(colors.text_muted)
                .show();
        });

    ui.spacing(2.0);

    // 2. GitHub Copilot Card (Coming Soon)
    ui.container()
        .full_width()
        .padding(12.0, 14.0, 12.0, 14.0)
        .radius_all(8.0)
        .bg(colors.bg_base)
        .border(colors.border.with_alpha(0.4), 0.75)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container().row().valign(Align::Center).gap(10.0).show(|ui| {
                ui.container()
                    .width(30.0)
                    .height(30.0)
                    .radius_all(6.0)
                    .bg(colors.highlight)
                    .valign(Align::Center)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text("🐙").size(13.0).show();
                    });
                ui.container().column().gap(2.0).show(|ui| {
                    ui.text("GitHub Copilot")
                        .size(12.0)
                        .weight(FontWeight::SemiBold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("GitHub Language Server & Workspace Agent (ACP)")
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                });
            });

            ui.container()
                .padding(3.0, 8.0, 3.0, 8.0)
                .radius_all(5.0)
                .bg(colors.highlight)
                .show(|ui| {
                    ui.text("Coming Soon")
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                });
        });

    ui.spacing(2.0);

    // 3. Claude Code Card (Coming Soon)
    ui.container()
        .full_width()
        .padding(12.0, 14.0, 12.0, 14.0)
        .radius_all(8.0)
        .bg(colors.bg_base)
        .border(colors.border.with_alpha(0.4), 0.75)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container().row().valign(Align::Center).gap(10.0).show(|ui| {
                ui.container()
                    .width(30.0)
                    .height(30.0)
                    .radius_all(6.0)
                    .bg(colors.highlight)
                    .valign(Align::Center)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text("🤖").size(13.0).show();
                    });
                ui.container().column().gap(2.0).show(|ui| {
                    ui.text("Claude Code")
                        .size(12.0)
                        .weight(FontWeight::SemiBold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Anthropic Claude Code autonomous agent via local ACP daemon")
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                });
            });

            ui.container()
                .padding(3.0, 8.0, 3.0, 8.0)
                .radius_all(5.0)
                .bg(colors.highlight)
                .show(|ui| {
                    ui.text("Coming Soon")
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                });
        });

    ui.spacing(2.0);

    // 4. Custom External Agent Card
    ui.container()
        .full_width()
        .padding(12.0, 14.0, 12.0, 14.0)
        .radius_all(8.0)
        .bg(colors.bg_base)
        .border(colors.border.with_alpha(0.4), 0.75)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container().row().valign(Align::Center).gap(10.0).show(|ui| {
                ui.container()
                    .width(30.0)
                    .height(30.0)
                    .radius_all(6.0)
                    .bg(colors.highlight)
                    .valign(Align::Center)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text("⚙").size(13.0).show();
                    });
                ui.container().column().gap(2.0).show(|ui| {
                    ui.text("Custom ACP Binary")
                        .size(12.0)
                        .weight(FontWeight::SemiBold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Launch any local executable supporting stdio Agent Client Protocol")
                        .size(9.5)
                        .color(colors.text_muted)
                        .show();
                });
            });

            ui.container()
                .clickable(true)
                .padding(4.0, 9.0, 4.0, 9.0)
                .radius_all(5.0)
                .bg(colors.highlight)
                .border(colors.border.with_alpha(0.5), 0.75)
                .show(|ui| {
                    ui.text("+ Add Binary")
                        .size(9.5)
                        .color(colors.text_primary)
                        .show();
                });
        });
}

fn draw_antigravity_settings_model_button(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: &ThemeColors,
) {
    if !state.selected_ai_model.is_antigravity() {
        state.selected_ai_model = AiModel::AntigravityGemini38FlashHigh;
    }

    let is_open = state.ai_settings.antigravity_model_dropdown_open;
    let model_btn = ui.container()
        .raw_id(AG_SETTINGS_MODEL_BTN_ID)
        .clickable(true)
        .width(200.0)
        .height(28.0)
        .padding(4.0, 10.0, 4.0, 10.0)
        .radius_all(5.0)
        .bg(if is_open { colors.highlight } else { colors.bg_base })
        .hover_bg(colors.highlight)
        .border(colors.border.with_alpha(0.6), 1.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.text(state.selected_ai_model.display_name())
                .size(10.5)
                .weight(FontWeight::Medium)
                .color(colors.text_primary)
                .show();
            ui.text(if is_open { "\u{f077}" } else { "\u{f078}" })
                .size(8.0)
                .color(colors.text_muted)
                .show();
        });

    if model_btn.hovered {
        ui.cursor_icon = CursorIcon::Pointer;
    }
    if model_btn.clicked {
        ui.clicked = false;
        state.ai_settings.antigravity_model_dropdown_open = !state.ai_settings.antigravity_model_dropdown_open;
        ui.needs_redraw = true;
    }
}

pub fn draw_antigravity_settings_model_popover(
    ui: &mut Ui,
    state: &mut ZenthreeState,
) {
    if !state.ai_settings.antigravity_model_dropdown_open {
        return;
    }

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    if !state.selected_ai_model.is_antigravity() {
        state.selected_ai_model = AiModel::AntigravityGemini38FlashHigh;
    }

    let menu_w = 200.0;
    let models = AiModel::ANTIGRAVITY_MODELS;
    let menu_h = (models.len() as f32 * 26.0 + 8.0).min(240.0);

    let is_light = colors.bg_base.r > 0.5;
    let menu_bg = if state.glassmorphism {
        if is_light {
            Color::rgba(250.0 / 255.0, 250.0 / 255.0, 252.0 / 255.0, 0.95)
        } else {
            Color::rgba(22.0 / 255.0, 22.0 / 255.0, 25.0 / 255.0, 0.96)
        }
    } else {
        if is_light {
            Color::rgb(250.0 / 255.0, 250.0 / 255.0, 252.0 / 255.0)
        } else {
            Color::rgb(22.0 / 255.0, 22.0 / 255.0, 25.0 / 255.0)
        }
    };
    let menu_border = if is_light {
        Color::rgb(200.0 / 255.0, 200.0 / 255.0, 208.0 / 255.0)
    } else {
        Color::rgb(45.0 / 255.0, 45.0 / 255.0, 54.0 / 255.0)
    };
    let menu_shadow = if is_light {
        Color::rgba(0.0, 0.0, 0.0, 0.18)
    } else {
        Color::rgba(0.0, 0.0, 0.0, 0.65)
    };

    if let Some(anchor) = ui.next_screen_layout_cache.get(&AG_SETTINGS_MODEL_BTN_ID)
        .or_else(|| ui.screen_layout_cache.get(&AG_SETTINGS_MODEL_BTN_ID)).copied()
    {
        let (mx, my) = zenthra_widgets::compute_placement_pos(
            anchor,
            menu_w,
            menu_h,
            Placement::Bottom,
            PopoverAlign::Start,
            4.0,
            true,
            ui.width,
            ui.height,
        );
        let mouse_in_menu = ui.mouse_in_rect(mx, my, menu_w, menu_h);
        let mouse_in_trigger = ui.mouse_in_rect(anchor.origin.x, anchor.origin.y, anchor.size.width, anchor.size.height);
        if ui.clicked && !mouse_in_menu && !mouse_in_trigger {
            state.ai_settings.antigravity_model_dropdown_open = false;
            ui.needs_redraw = true;
            return;
        }
    }

    let mut select_model: Option<AiModel> = None;
    let mut popover_builder = ui.popover(AG_SETTINGS_MODEL_BTN_ID)
        .placement(Placement::Bottom)
        .align(PopoverAlign::Start)
        .gap(4.0)
        .auto_flip(true)
        .width(menu_w)
        .height(menu_h)
        .bg(menu_bg)
        .border(menu_border, 1.0)
        .radius_all(6.0)
        .shadow(menu_shadow, 0.0, 4.0, 16.0)
        .padding(4.0, 4.0, 4.0, 4.0);

    if state.glassmorphism {
        popover_builder = popover_builder.backdrop_blur(state.blur_strength.max(4.0));
    }

    popover_builder.show(|ui| {
            ui.container()
                .full_width()
                .height(menu_h - 8.0)
                .column()
                .gap(1.5)
                .scroll_y(true)
                .show(|ui| {
                    // Models only, NO title/heading, active model highlighted, NO ticks
                    for model in models {
                        let is_selected = state.selected_ai_model == model;
                        let row_id = ui.id_from(&format!("pop_ag_settings_model_{}", model.model_id()));
                        let row = ui.container()
                            .id(row_id)
                            .full_width()
                            .height(26.0)
                            .padding(2.0, 8.0, 2.0, 8.0)
                            .radius_all(4.0)
                            .bg(if is_selected { colors.accent.with_alpha(0.12) } else { Color::TRANSPARENT })
                            .hover_bg(colors.highlight)
                            .row()
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text(model.display_name())
                                    .size(10.0)
                                    .weight(if is_selected { FontWeight::Medium } else { FontWeight::Regular })
                                    .color(if is_selected { colors.accent } else { colors.text_primary })
                                    .show();
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
        state.selected_agent = SelectedAgent::Antigravity;
        state.ai_settings.antigravity_model_dropdown_open = false;
        state.save_persistent_state();
        ui.needs_redraw = true;
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 3c. MCP (Model Context Protocol) Tab (UI Demo)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_mcp_tab(ui: &mut Ui, _state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container().column().gap(3.0).show(|ui| {
                ui.text("Model Context Protocol (MCP)")
                    .size(14.0)
                    .weight(FontWeight::Bold)
                    .color(colors.text_primary)
                    .show();
                ui.text("Extend agents with external databases, CLI tools, and workspace context")
                    .size(11.0)
                    .color(colors.text_muted)
                    .show();
            });

            ui.container()
                .padding(4.5, 10.0, 4.5, 10.0)
                .radius_all(5.0)
                .bg(colors.highlight)
                .border(colors.border.with_alpha(0.5), 0.75)
                .show(|ui| {
                    ui.text("+ Add Server")
                        .size(10.5)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                });
        });

    ui.spacing(4.0);

    // MCP Server Card 1: sqlite-memory
    ui.container()
        .full_width()
        .padding(12.0, 14.0, 12.0, 14.0)
        .radius_all(8.0)
        .bg(colors.bg_base)
        .border(colors.border.with_alpha(0.5), 1.0)
        .column()
        .gap(8.0)
        .show(|ui| {
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container().row().valign(Align::Center).gap(8.0).show(|ui| {
                        ui.text("🔌").size(12.0).show();
                        ui.text("sqlite-memory")
                            .size(12.5)
                            .weight(FontWeight::SemiBold)
                            .color(colors.text_primary)
                            .show();
                        ui.container()
                            .padding(2.0, 6.0, 2.0, 6.0)
                            .radius_all(4.0)
                            .bg(Color::rgb(0.15, 0.55, 0.25).with_alpha(0.2))
                            .show(|ui| {
                                ui.text("Active • 3 tools")
                                    .size(9.0)
                                    .color(Color::rgb(0.2, 0.9, 0.4))
                                    .weight(FontWeight::Medium)
                                    .show();
                            });
                    });

                    ui.container()
                        .padding(2.5, 8.0, 2.5, 8.0)
                        .radius_all(10.0)
                        .bg(colors.accent.with_alpha(0.2))
                        .show(|ui| {
                            ui.text("ON").size(9.5).weight(FontWeight::Bold).color(colors.accent).show();
                        });
                });

            ui.text("stdio: npx -y @modelcontextprotocol/server-sqlite --db ./dev.db")
                .size(10.0)
                .color(colors.text_muted)
                .show();

            ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                for tool in &["read_query", "write_query", "list_tables"] {
                    ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(3.0)
                        .bg(colors.highlight)
                        .show(|ui| {
                            ui.text(tool)
                                .size(9.0)
                                .color(colors.text_muted)
                                .show();
                        });
                }
            });
        });

    ui.spacing(2.0);

    // MCP Server Card 2: github-context
    ui.container()
        .full_width()
        .padding(12.0, 14.0, 12.0, 14.0)
        .radius_all(8.0)
        .bg(colors.bg_base)
        .border(colors.border.with_alpha(0.5), 1.0)
        .column()
        .gap(8.0)
        .show(|ui| {
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container().row().valign(Align::Center).gap(8.0).show(|ui| {
                        ui.text("🐙").size(12.0).show();
                        ui.text("github-context")
                            .size(12.5)
                            .weight(FontWeight::SemiBold)
                            .color(colors.text_primary)
                            .show();
                        ui.container()
                            .padding(2.0, 6.0, 2.0, 6.0)
                            .radius_all(4.0)
                            .bg(Color::rgb(0.15, 0.55, 0.25).with_alpha(0.2))
                            .show(|ui| {
                                ui.text("Active • 5 tools")
                                    .size(9.0)
                                    .color(Color::rgb(0.2, 0.9, 0.4))
                                    .weight(FontWeight::Medium)
                                    .show();
                            });
                    });

                    ui.container()
                        .padding(2.5, 8.0, 2.5, 8.0)
                        .radius_all(10.0)
                        .bg(colors.accent.with_alpha(0.2))
                        .show(|ui| {
                            ui.text("ON").size(9.5).weight(FontWeight::Bold).color(colors.accent).show();
                        });
                });

            ui.text("stdio: docker run -i mcp/github")
                .size(10.0)
                .color(colors.text_muted)
                .show();

            ui.container().row().valign(Align::Center).gap(6.0).show(|ui| {
                for tool in &["get_issue", "list_prs", "search_code", "+2 more"] {
                    ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(3.0)
                        .bg(colors.highlight)
                        .show(|ui| {
                            ui.text(tool)
                                .size(9.0)
                                .color(colors.text_muted)
                                .show();
                        });
                }
            });
        });

    ui.spacing(4.0);

    // Popular Presets
    ui.text("Popular MCP Presets (1-Click Install)")
        .size(11.5)
        .weight(FontWeight::SemiBold)
        .color(colors.text_primary)
        .show();

    ui.container().row().gap(8.0).show(|ui| {
        let presets = [
            ("🧠 Memory", "Cross-session memory graph"),
            ("🌐 Fetch", "Read & parse live web pages"),
            ("📂 Git Diff", "Rich working-tree inspection"),
        ];
        for (name, desc) in presets {
            ui.container()
                .width(180.0)
                .padding(8.0, 10.0, 8.0, 10.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .border(colors.border.with_alpha(0.4), 0.75)
                .column()
                .gap(4.0)
                .show(|ui| {
                    ui.text(name)
                        .size(11.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text(desc)
                        .size(9.0)
                        .color(colors.text_muted)
                        .show();
                });
        }
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// 4. Customization Tab (UI Only)
// ─────────────────────────────────────────────────────────────────────────────

fn draw_customization_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .full_width()
        .column()
        .gap(3.0)
        .show(|ui| {
            ui.text("Agent Customization")
                .size(14.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();
            ui.text("Configure persona roles and global system instructions")
                .size(11.0)
                .color(colors.text_muted)
                .show();
        });

    ui.spacing(2.0);

    // Row 1: Agent Persona Mode
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(300.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Agent Persona")
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Specialization mode for autonomous coding assistance")
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });

            ui.container()
                .row()
                .gap(4.0)
                .padding(3.0, 4.0, 3.0, 4.0)
                .radius_all(6.0)
                .bg(colors.bg_base)
                .show(|ui| {
                    for mode in AgentMode::ALL {
                        let is_active = state.agent_mode == mode;
                        let btn = ui.container()
                            .clickable(true)
                            .padding(5.0, 11.0, 5.0, 11.0)
                            .radius_all(4.0)
                            .bg(if is_active { colors.accent } else { Color::TRANSPARENT })
                            .hover_bg(if is_active { colors.accent } else { colors.highlight })
                            .show(|ui| {
                                ui.text(mode.display_name())
                                    .size(10.5)
                                    .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                    .color(if is_active { Color::WHITE } else { colors.text_muted })
                                    .show();
                            });
                        if btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if btn.clicked {
                            state.agent_mode = mode;
                            ui.needs_redraw = true;
                        }
                    }
                });
        });

    draw_settings_divider(ui, colors);

    // Row 2: Custom Instructions Header
    ui.container()
        .full_width()
        .column()
        .gap(6.0)
        .show(|ui| {
            ui.text("Custom System Instructions")
                .size(12.0)
                .weight(FontWeight::Medium)
                .color(colors.text_primary)
                .show();
            ui.text("Injected into the system prompt at the beginning of each session")
                .size(10.5)
                .color(colors.text_muted)
                .show();

            ui.spacing(2.0);

            ui.text_area(&mut state.ai_settings.custom_instructions, "ai_custom_instructions_textarea")
                .width(490.0)
                .height(110.0)
                .scrollable(true)
                .size(11.0)
                .color(colors.text_primary)
                .bg(colors.bg_base)
                .border(colors.border.with_alpha(0.5), 0.75)
                .focus_border(colors.accent.with_alpha(0.8), 1.0)
                .radius(6.0, 6.0, 6.0, 6.0)
                .padding(8.0, 10.0, 8.0, 10.0)
                .placeholder("e.g. Adhere to strict idiomatic Rust, never use unwrap() in production code.")
                .placeholder_color(colors.text_muted.with_alpha(0.5))
                .wrap(zenthra_text::prelude::TextWrap::Word)
                .show();
        });
}

// ─────────────────────────────────────────────────────────────────────────────
// Reusable Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn draw_toggle_row_inline(
    ui: &mut Ui,
    id_seed: u64,
    title: &str,
    description: &str,
    val: &mut bool,
    is_locked: bool,
    label_alpha: f32,
    sub_alpha: f32,
    colors: &ThemeColors,
) -> bool {
    let mut toggled = false;
    ui.container()
        .full_width()
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .width(330.0)
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text(title)
                        .size(12.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_primary.with_alpha(label_alpha))
                        .show();
                    ui.text(description)
                        .size(10.5)
                        .color(colors.text_muted.with_alpha(sub_alpha))
                        .show();
                });

            ui.container()
                .padding(2.0, 4.0, 2.0, 4.0)
                .row()
                .valign(Align::Center)
                .show(|ui| {
                    let resp = ui.toggle(val, None)
                        .id(Id::from_u64(id_seed))
                        .size(38.0, 20.0)
                        .pill()
                        .colors(colors.accent, colors.border.with_alpha(0.6), Color::WHITE)
                        .disabled(is_locked)
                        .show();
                    if resp.clicked {
                        toggled = true;
                    }
                });
        });
    toggled
}

fn draw_settings_divider(ui: &mut Ui, colors: &ThemeColors) {
    ui.container()
        .full_width()
        .height(1.0)
        .bg(colors.border.with_alpha(0.35))
        .show(|_| {});
}
