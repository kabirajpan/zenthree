use crate::state::{
    AgentMode, AiInputTone, AiMessageBg, AiModel, AiPresetMode, AiSettingsTab, AiTextSize,
    AiThinkingBudget, ZenthreeState,
};
use crate::theme::ThemeColors;
use zenthra::{use_context, Align, Color, CursorIcon, FontWeight, Id, Ui};

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
                        (AiTextSize::Compact, "10.5px"),
                        (AiTextSize::Regular, "11.5px"),
                        (AiTextSize::Large, "12.5px"),
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

