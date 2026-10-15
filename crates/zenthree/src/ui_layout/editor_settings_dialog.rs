use crate::state::{EditorSettingsTab, ZenthreeState};
use crate::state::types::{CursorStyle, CursorColorMode};
use crate::theme::{ThemeColors, ACCENT_COLOR_OPTIONS, named_color};
use zenthra::{use_context, Align, Color, CursorIcon, FontWeight, Id, Style, Ui, Wrap};

pub fn draw_editor_settings_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.editor_settings_open {
        return;
    }

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    // Generous, minimalist floating window dimensions
    let win_w = 900.0;
    let win_h = 630.0;
    let win_radius = 8.0;

    let viewport = ui.current_viewport;
    let center_x = viewport.origin.x + (viewport.size.width - win_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height - win_h) / 2.0;

    // 1. Semi-transparent dark backdrop
    ui.overlays.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [viewport.origin.x, viewport.origin.y],
            size: [viewport.size.width, viewport.size.height],
            color: [0.0, 0.0, 0.0, 0.50],
            radius: [0.0; 4],
            ..Default::default()
        },
    }));

    // Escape key handling
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

    let modal_id = Id::from_u64(999999811);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_window_id = Some(modal_id);
    let prev_win_id = ui.current_window_id;
    ui.current_window_id = Some(modal_id);
    ui.record_layout(modal_id, zenthra_core::Rect::new(center_x, center_y, win_w, win_h));

    if close_on_esc {
        state.editor_settings_open = false;
        ui.interaction_state.remove(&modal_key);
        state.save_persistent_state();
        ui.current_window_id = prev_win_id;
        ui.needs_redraw = true;
        return;
    }

    // Outside click detection
    if state.editor_settings_just_opened {
        state.editor_settings_just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, win_w, win_h);
        if ui.clicked && !mouse_in_dialog {
            state.editor_settings_open = false;
            ui.interaction_state.remove(&modal_key);
            ui.consume_click();
            state.save_persistent_state();
            ui.current_window_id = prev_win_id;
            ui.needs_redraw = true;
            return;
        }
    }

    // 2. Centered Floating Dialog Card (reacts dynamically to glass_opacity and blur_strength)
    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 1.2).clamp(0.40, 0.95))
    } else {
        colors.bg_sidebar
    };

    let mut card = ui.container()
        .id(Id::from_u64(999999811))
        .pos(center_x, center_y)
        .overlay()
        .width(win_w)
        .height(win_h)
        .bg(card_bg)
        .border((colors.border.with_alpha(0.40), 1.0))
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
                .fill_x()
                .height(46.0)
                .padding(0.0, 20.0, 0.0, 20.0)
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
                            ui.text("\u{f013}") // cog
                                .size(14.0)
                                .color(colors.accent)
                                .show();
                            ui.text("Preferences")
                                .size(13.5)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("/")
                                .size(13.0)
                                .color(colors.text_dim)
                                .show();
                            ui.text(state.editor_settings_tab.display_name())
                                .size(13.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    let close_btn = ui.container()
                        .interactive(true)
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .radius_all(6.0)
                        .hover(Style::bg(colors.highlight))
                        .show(|ui| {
                            ui.text("\u{f00d}") // times
                                .size(13.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    if close_btn.hovered {
                        ui.cursor_icon = CursorIcon::Pointer;
                    }
                    if close_btn.clicked {
                        state.editor_settings_open = false;
                        state.save_persistent_state();
                        ui.needs_redraw = true;
                    }
                });

            // Header bottom border
            ui.container()
                .fill_x()
                .height(1.0)
                .bg(colors.border.with_alpha(0.25))
                .show(|_| {});

            // Body: Left Sidebar + Right Scrollable Content Area
            let body_h = win_h - 47.0;
            let sidebar_w = 205.0;
            let content_w = win_w - sidebar_w - 1.0;

            ui.container()
                .fill_x()
                .height(body_h)
                .row()
                .show(|ui| {
                    // Left navigation tabs sidebar
                    ui.container()
                        .width(sidebar_w)
                        .height(body_h)
                        .bg(colors.bg_base.with_alpha(0.35))
                        .radius_bottom_left(win_radius)
                        .padding(14.0, 10.0, 14.0, 10.0)
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            for tab in &EditorSettingsTab::ALL {
                                let is_active = state.editor_settings_tab == *tab;
                                let tab_btn = ui.container()
                                    .fill_x()
                                    .interactive(true)
                                    .padding(5.5, 10.0, 5.5, 10.0)
                                    .radius_all(5.0)
                                    .bg(if is_active { colors.highlight.with_alpha(0.06) } else { Color::TRANSPARENT })
                                    .hover(Style::bg(colors.highlight))
                                    .row()
                                    .valign(Align::Center)
                                    .gap(9.0)
                                    .show(|ui| {
                                        ui.text(tab.icon())
                                            .size(12.0)
                                            .color(if is_active { colors.accent } else { colors.text_muted })
                                            .show();
                                        ui.text(tab.display_name())
                                            .size(12.0)
                                            .font_weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_active { colors.text_primary } else { colors.text_muted })
                                            .show();
                                    });

                                if tab_btn.hovered {
                                    ui.cursor_icon = CursorIcon::Pointer;
                                }
                                if tab_btn.clicked {
                                    state.editor_settings_tab = *tab;
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

                    // Right Scrollable Content Area
                    let scroll_id = Id::from_u64(999999820 + state.editor_settings_tab as u64);
                    ui.container()
                        .id(scroll_id)
                        .width(content_w)
                        .height(body_h)
                        .scroll_y(true)
                        .clip(true)
                        .radius_bottom_right(win_radius)
                        .padding(16.0, 20.0, 16.0, 20.0)
                        .column()
                        .gap(12.0)
                        .show(|ui| {
                            match state.editor_settings_tab {
                                EditorSettingsTab::Appearance => {
                                    draw_appearance_tab(ui, state, &colors);
                                }
                                EditorSettingsTab::Editor => {
                                    draw_editor_tab(ui, state, &colors);
                                }
                                EditorSettingsTab::Terminal => {
                                    draw_terminal_tab(ui, state, &colors);
                                }
                                EditorSettingsTab::Agent => {
                                    draw_agent_tab(ui, state, &colors);
                                }
                                EditorSettingsTab::Keymaps => {
                                    draw_keymaps_tab(ui, state, &colors);
                                }
                            }
                        });
                });
        });

    ui.current_window_id = prev_win_id;
}

fn draw_appearance_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    let card_panel_bg = if state.glassmorphism {
        colors.bg_base.with_alpha((state.glass_opacity * 0.65).clamp(0.20, 0.70))
    } else {
        colors.bg_base.with_alpha(0.50)
    };

    ui.container()
        .column()
        .gap(10.0)
        .show(|ui| {
            // Header with Reset Defaults button
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.text("Appearance")
                                .size(15.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Blur, opacity, and editor font size.")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    let reset_all = ui.container()
                        .interactive(true)
                        .padding(4.0, 10.0, 4.0, 10.0)
                        .radius_all(5.0)
                        .bg(colors.accent.with_alpha(0.10))
                        .hover(Style::bg(colors.accent.with_alpha(0.20)))
                        .border((colors.accent.with_alpha(0.35), 1.0))
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .show(|ui| {
                            ui.text("↺")
                                .size(12.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.accent)
                                .show();
                            ui.text("Reset Defaults")
                                .size(11.0)
                                .font_weight(FontWeight::Medium)
                                .color(colors.text_primary)
                                .show();
                        });

                    if reset_all.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if reset_all.clicked {
                        state.set_blur_strength(24.0);
                        state.set_glass_opacity(0.55);
                        state.set_font_size(13.0);
                        state.reset_zoom();
                        state.glassmorphism = true;
                        ui.set_blur(true);
                        ui.needs_redraw = true;
                    }
                });

            // 1. Frosted Glassmorphism Switch (using Zenthra Toggle Widget)
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border((colors.border.with_alpha(0.20), 1.0))
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.text("Window Blur")
                                .size(13.0)
                                .font_weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Blur behind the app window (if supported by your OS).")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    let mut is_glass = state.glassmorphism;
                    let toggle_resp = ui.toggle(&mut is_glass, None)
                        .width(40.0)
                        .height(20.0)
                        .pill()
                        .colors(colors.accent, colors.highlight, Color::WHITE)
                        .show();

                    if toggle_resp.clicked {
                        state.glassmorphism = is_glass;
                        ui.set_blur(state.glassmorphism);
                        state.save_persistent_state();
                        ui.needs_redraw = true;
                    }
                });

            // 2. Blur Strength / Radius Slider with Steppers & Presets
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border((colors.border.with_alpha(0.20), 1.0))
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .show(|ui| {
                            ui.container()
                                .column()
                                .gap(2.0)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(7.0)
                                        .show(|ui| {
                                            ui.text("Panel Blur")
                                                .size(13.0)
                                                .font_weight(FontWeight::SemiBold)
                                                .color(colors.text_primary)
                                                .show();
                                            if (state.blur_strength - 24.0).abs() < 0.5 {
                                                ui.container()
                                                    .padding(2.0, 5.0, 2.0, 5.0)
                                                    .radius_all(4.0)
                                                    .bg(colors.accent.with_alpha(0.15))
                                                    .show(|ui| {
                                                        ui.text("Default")
                                                            .size(9.0)
                                                            .font_weight(FontWeight::SemiBold)
                                                            .color(colors.accent)
                                                            .show();
                                                    });
                                            }
                                        });
                                    ui.text("Blur dialogs, cards, and floating panels. Default: 24px")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .wrap(true)
                                        .show();
                                });

                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    // Decrement stepper
                                    let dec_btn = ui.container()
                                        .interactive(true)
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(4.0)
                                        .bg(colors.highlight)
                                        .border((colors.border.with_alpha(0.30), 1.0))
                                        .hover(Style::bg(colors.accent.with_alpha(0.18)))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("-")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if dec_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if dec_btn.clicked {
                                        state.set_blur_strength((state.blur_strength - 2.0).max(4.0));
                                        ui.needs_redraw = true;
                                    }

                                    // Slider
                                    let mut blur_val = state.blur_strength;
                                    let slider_resp = ui.slider(&mut blur_val, "blur_slider")
                                        .range(4.0, 64.0)
                                        .step(2.0)
                                        .track_size(120.0, 4.0)
                                        .track_radius(2.0, 2.0, 2.0, 2.0)
                                        .track_color(colors.highlight)
                                        .thumb_size(12.0, 12.0)
                                        .thumb_radius(6.0, 6.0, 6.0, 6.0)
                                        .thumb_color(colors.accent)
                                        .show();

                                    if (blur_val - state.blur_strength).abs() > 0.001 || slider_resp.clicked || slider_resp.pressed {
                                        state.set_blur_strength(blur_val);
                                        ui.needs_redraw = true;
                                    }

                                    // Increment stepper
                                    let inc_btn = ui.container()
                                        .interactive(true)
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(4.0)
                                        .bg(colors.highlight)
                                        .border((colors.border.with_alpha(0.30), 1.0))
                                        .hover(Style::bg(colors.accent.with_alpha(0.18)))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("+")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if inc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if inc_btn.clicked {
                                        state.set_blur_strength((state.blur_strength + 2.0).min(64.0));
                                        ui.needs_redraw = true;
                                    }

                                    ui.container()
                                        .width(42.0)
                                        .halign(Align::Right)
                                        .show(|ui| {
                                            ui.text(&format!("{:.0}px", state.blur_strength))
                                                .size(11.5)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                });
                        });

                    // Preset chips + Reset button (wraps gracefully if narrow)
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .wrap(Wrap::Wrap)
                        .show(|ui| {
                            let presets = [
                                ("Subtle", 12.0f32, false),
                                ("Balanced", 24.0f32, true),
                                ("Strong", 36.0f32, false),
                                ("Ultra", 48.0f32, false),
                            ];
                            for (label, val, is_default) in presets {
                                let is_sel = (state.blur_strength - val).abs() < 1.0;
                                let chip_text = if is_default {
                                    format!("{} ({:.0}px) • Def", label, val)
                                } else {
                                    format!("{} ({:.0}px)", label, val)
                                };
                                let chip = ui.container()
                                    .interactive(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                    .border((if is_sel { colors.accent } else { colors.border.with_alpha(0.30) }, 1.0))
                                    .hover(Style::bg(if is_sel { colors.accent.with_alpha(0.28) } else { colors.accent.with_alpha(0.10) }))
                                    .show(|ui| {
                                        ui.text(&chip_text)
                                            .size(10.0)
                                            .font_weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_sel { colors.accent } else { colors.text_muted })
                                            .show();
                                    });
                                if chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if chip.clicked {
                                    state.set_blur_strength(val);
                                    ui.needs_redraw = true;
                                }
                            }

                            if (state.blur_strength - 24.0).abs() >= 1.0 {
                                let reset_chip = ui.container()
                                    .interactive(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(colors.accent.with_alpha(0.12))
                                    .border((colors.accent.with_alpha(0.40), 1.0))
                                    .hover(Style::bg(colors.accent.with_alpha(0.24)))
                                    .row()
                                    .valign(Align::Center)
                                    .gap(4.0)
                                    .show(|ui| {
                                        ui.text("↺")
                                            .size(10.5)
                                            .font_weight(FontWeight::Bold)
                                            .color(colors.accent)
                                            .show();
                                        ui.text("24px")
                                            .size(10.0)
                                            .font_weight(FontWeight::SemiBold)
                                            .color(colors.text_primary)
                                            .show();
                                    });
                                if reset_chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if reset_chip.clicked {
                                    state.set_blur_strength(24.0);
                                    ui.needs_redraw = true;
                                }
                            }
                        });

                    // Informative note regarding internal UI blur vs native OS compositor desktop blur
                    ui.container()
                        .fill_x()
                        .padding(8.0, 12.0, 8.0, 12.0)
                        .radius_all(5.0)
                        .bg(colors.highlight.with_alpha(0.04))
                        .border((colors.border.with_alpha(0.12), 1.0))
                        .row()
                        .valign(Align::Top)
                        .gap(10.0)
                        .show(|ui| {
                            ui.container()
                                .padding_top(1.0)
                                .show(|ui| {
                                    ui.text(zenthra::icons::NF_FA_INFO)
                                        .size(11.5)
                                        .color(colors.accent)
                                        .show();
                                });
                            ui.container()
                                .column()
                                .gap(3.0)
                                .show(|ui| {
                                    ui.text("Controls blur inside the app. Window blur is a separate OS setting.")
                                        .size(10.5)
                                        .color(colors.text_primary)
                                        .wrap(true)
                                        .show();
                                });
                        });
                });

            // 3. Surface Opacity / Frost Depth Slider with Steppers & Presets
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border((colors.border.with_alpha(0.20), 1.0))
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .show(|ui| {
                            ui.container()
                                .column()
                                .gap(2.0)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(7.0)
                                        .show(|ui| {
                                            ui.text("Panel Opacity")
                                                .size(13.0)
                                                .font_weight(FontWeight::SemiBold)
                                                .color(colors.text_primary)
                                                .show();
                                            if (state.glass_opacity - 0.55).abs() < 0.02 {
                                                ui.container()
                                                    .padding(2.0, 5.0, 2.0, 5.0)
                                                    .radius_all(4.0)
                                                    .bg(colors.accent.with_alpha(0.15))
                                                    .show(|ui| {
                                                        ui.text("Default")
                                                            .size(9.0)
                                                            .font_weight(FontWeight::SemiBold)
                                                            .color(colors.accent)
                                                            .show();
                                                    });
                                            }
                                        });
                                    ui.text("Editor and sidebar transparency. Default: 55%")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .wrap(true)
                                        .show();
                                });

                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    // Decrement stepper
                                    let dec_btn = ui.container()
                                        .interactive(true)
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(4.0)
                                        .bg(colors.highlight)
                                        .border((colors.border.with_alpha(0.30), 1.0))
                                        .hover(Style::bg(colors.accent.with_alpha(0.18)))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("-")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if dec_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if dec_btn.clicked {
                                        state.set_glass_opacity((state.glass_opacity - 0.05).max(0.15));
                                        ui.needs_redraw = true;
                                    }

                                    // Slider
                                    let mut op_val = state.glass_opacity * 100.0;
                                    let slider_resp = ui.slider(&mut op_val, "opacity_slider")
                                        .range(15.0, 95.0)
                                        .step(5.0)
                                        .track_size(120.0, 4.0)
                                        .track_radius(2.0, 2.0, 2.0, 2.0)
                                        .track_color(colors.highlight)
                                        .thumb_size(12.0, 12.0)
                                        .thumb_radius(6.0, 6.0, 6.0, 6.0)
                                        .thumb_color(colors.accent)
                                        .show();

                                    if (op_val / 100.0 - state.glass_opacity).abs() > 0.001 || slider_resp.clicked || slider_resp.pressed {
                                        state.set_glass_opacity(op_val / 100.0);
                                        ui.needs_redraw = true;
                                    }

                                    // Increment stepper
                                    let inc_btn = ui.container()
                                        .interactive(true)
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(4.0)
                                        .bg(colors.highlight)
                                        .border((colors.border.with_alpha(0.30), 1.0))
                                        .hover(Style::bg(colors.accent.with_alpha(0.18)))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("+")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if inc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if inc_btn.clicked {
                                        state.set_glass_opacity((state.glass_opacity + 0.05).min(0.95));
                                        ui.needs_redraw = true;
                                    }

                                    ui.container()
                                        .width(42.0)
                                        .halign(Align::Right)
                                        .show(|ui| {
                                            ui.text(&format!("{:.0}%", state.glass_opacity * 100.0))
                                                .size(11.5)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                });
                        });

                    // Preset chips + Reset button (wraps gracefully if narrow)
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .wrap(Wrap::Wrap)
                        .show(|ui| {
                            let presets = [
                                ("Light Frost", 0.35f32, false),
                                ("Balanced", 0.55f32, true),
                                ("Frosted", 0.70f32, false),
                                ("Deep Tint", 0.85f32, false),
                            ];
                            for (label, val, is_default) in presets {
                                let is_sel = (state.glass_opacity - val).abs() < 0.03;
                                let chip_text = if is_default {
                                    format!("{} ({:.0}%) • Def", label, val * 100.0)
                                } else {
                                    format!("{} ({:.0}%)", label, val * 100.0)
                                };
                                let chip = ui.container()
                                    .interactive(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                    .border((if is_sel { colors.accent } else { colors.border.with_alpha(0.30) }, 1.0))
                                    .hover(Style::bg(if is_sel { colors.accent.with_alpha(0.28) } else { colors.accent.with_alpha(0.10) }))
                                    .show(|ui| {
                                        ui.text(&chip_text)
                                            .size(10.0)
                                            .font_weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_sel { colors.accent } else { colors.text_muted })
                                            .show();
                                    });
                                if chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if chip.clicked {
                                    state.set_glass_opacity(val);
                                    ui.needs_redraw = true;
                                }
                            }

                            if (state.glass_opacity - 0.55).abs() >= 0.03 {
                                let reset_chip = ui.container()
                                    .interactive(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(colors.accent.with_alpha(0.12))
                                    .border((colors.accent.with_alpha(0.40), 1.0))
                                    .hover(Style::bg(colors.accent.with_alpha(0.24)))
                                    .row()
                                    .valign(Align::Center)
                                    .gap(4.0)
                                    .show(|ui| {
                                        ui.text("↺")
                                            .size(10.5)
                                            .font_weight(FontWeight::Bold)
                                            .color(colors.accent)
                                            .show();
                                        ui.text("55%")
                                            .size(10.0)
                                            .font_weight(FontWeight::SemiBold)
                                            .color(colors.text_primary)
                                            .show();
                                    });
                                if reset_chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if reset_chip.clicked {
                                    state.set_glass_opacity(0.55);
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });

            // 4. Editor Font Size & Live Typography Preview
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border((colors.border.with_alpha(0.20), 1.0))
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .show(|ui| {
                            ui.container()
                                .column()
                                .gap(2.0)
                                .show(|ui| {
                                    ui.container()
                                        .row()
                                        .valign(Align::Center)
                                        .gap(7.0)
                                        .show(|ui| {
                                            ui.text("Editor Font Size")
                                                .size(13.0)
                                                .font_weight(FontWeight::SemiBold)
                                                .color(colors.text_primary)
                                                .show();
                                            if (state.font_size - 13.0).abs() < 0.1 {
                                                ui.container()
                                                    .padding(2.0, 5.0, 2.0, 5.0)
                                                    .radius_all(4.0)
                                                    .bg(colors.accent.with_alpha(0.15))
                                                    .show(|ui| {
                                                        ui.text("Default")
                                                            .size(9.0)
                                                            .font_weight(FontWeight::SemiBold)
                                                            .color(colors.accent)
                                                            .show();
                                                    });
                                            }
                                        });
                                    ui.text("Editor text size. Default: 13px")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .wrap(true)
                                        .show();
                                });

                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    // Decrement stepper
                                    let dec_btn = ui.container()
                                        .interactive(true)
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(4.0)
                                        .bg(colors.highlight)
                                        .border((colors.border.with_alpha(0.30), 1.0))
                                        .hover(Style::bg(colors.accent.with_alpha(0.18)))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("-")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if dec_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if dec_btn.clicked {
                                        state.set_font_size((state.font_size - 1.0).max(10.0));
                                        ui.needs_redraw = true;
                                    }

                                    // Slider
                                    let mut fs_val = state.font_size;
                                    let slider_resp = ui.slider(&mut fs_val, "fontsize_slider")
                                        .range(10.0, 26.0)
                                        .step(1.0)
                                        .track_size(120.0, 4.0)
                                        .track_radius(2.0, 2.0, 2.0, 2.0)
                                        .track_color(colors.highlight)
                                        .thumb_size(12.0, 12.0)
                                        .thumb_radius(6.0, 6.0, 6.0, 6.0)
                                        .thumb_color(colors.accent)
                                        .show();

                                    if (fs_val - state.font_size).abs() > 0.001 || slider_resp.clicked || slider_resp.pressed {
                                        state.set_font_size(fs_val);
                                        ui.needs_redraw = true;
                                    }

                                    // Increment stepper
                                    let inc_btn = ui.container()
                                        .interactive(true)
                                        .width(22.0)
                                        .height(22.0)
                                        .radius_all(4.0)
                                        .bg(colors.highlight)
                                        .border((colors.border.with_alpha(0.30), 1.0))
                                        .hover(Style::bg(colors.accent.with_alpha(0.18)))
                                        .valign(Align::Center)
                                        .halign(Align::Center)
                                        .show(|ui| {
                                            ui.text("+")
                                                .size(13.0)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.text_primary)
                                                .show();
                                        });
                                    if inc_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                    if inc_btn.clicked {
                                        state.set_font_size((state.font_size + 1.0).min(26.0));
                                        ui.needs_redraw = true;
                                    }

                                    ui.container()
                                        .width(42.0)
                                        .halign(Align::Right)
                                        .show(|ui| {
                                            ui.text(&format!("{:.0}px", state.font_size))
                                                .size(11.5)
                                                .font_weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                });
                        });

                    // Preset chips + Reset button (wraps gracefully if narrow)
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .wrap(Wrap::Wrap)
                        .show(|ui| {
                            let sizes = [
                                (11.0f32, false),
                                (12.0, false),
                                (13.0, true),
                                (14.0, false),
                                (16.0, false),
                                (18.0, false),
                                (20.0, false),
                            ];
                            for (s, is_default) in sizes {
                                let is_sel = (state.font_size - s).abs() < 0.1;
                                let chip_text = if is_default {
                                    format!("{:.0}px • Def", s)
                                } else {
                                    format!("{:.0}px", s)
                                };
                                let chip = ui.container()
                                    .interactive(true)
                                    .padding(3.0, 7.0, 3.0, 7.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                    .border((if is_sel { colors.accent } else { colors.border.with_alpha(0.30) }, 1.0))
                                    .hover(Style::bg(if is_sel { colors.accent.with_alpha(0.28) } else { colors.accent.with_alpha(0.10) }))
                                    .show(|ui| {
                                        ui.text(&chip_text)
                                            .size(10.0)
                                            .font_weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_sel { colors.accent } else { colors.text_muted })
                                            .show();
                                    });
                                if chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if chip.clicked {
                                    state.set_font_size(s);
                                    ui.needs_redraw = true;
                                }
                            }

                            if (state.font_size - 13.0).abs() >= 0.1 {
                                let reset_chip = ui.container()
                                    .interactive(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(colors.accent.with_alpha(0.12))
                                    .border((colors.accent.with_alpha(0.40), 1.0))
                                    .hover(Style::bg(colors.accent.with_alpha(0.24)))
                                    .row()
                                    .valign(Align::Center)
                                    .gap(4.0)
                                    .show(|ui| {
                                        ui.text("↺")
                                            .size(10.5)
                                            .font_weight(FontWeight::Bold)
                                            .color(colors.accent)
                                            .show();
                                        ui.text("13px")
                                            .size(10.0)
                                            .font_weight(FontWeight::SemiBold)
                                            .color(colors.text_primary)
                                            .show();
                                    });
                                if reset_chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if reset_chip.clicked {
                                    state.set_font_size(13.0);
                                    ui.needs_redraw = true;
                                }
                            }
                        });

                    // Live Typography Preview Card
                    ui.container()
                        .fill_x()
                        .padding(8.0, 12.0, 8.0, 12.0)
                        .radius_all(5.0)
                        .bg(colors.bg_editor)
                        .border((colors.border.with_alpha(0.25), 1.0))
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    ui.text("fn")
                                        .size(state.font_size)
                                        .color(Color::rgb(0.75, 0.45, 0.95))
                                        .show();
                                    ui.text("render_buffer")
                                        .size(state.font_size)
                                        .font_weight(FontWeight::SemiBold)
                                        .color(Color::rgb(0.40, 0.75, 1.0))
                                        .show();
                                    ui.text("() -> Result<()> {")
                                        .size(state.font_size)
                                        .color(colors.text_primary)
                                        .show();
                                });
                            ui.container()
                                .padding(0.0, 0.0, 0.0, 18.0)
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    ui.text("// Live typography preview at current scale")
                                        .size(state.font_size)
                                        .color(colors.text_dim)
                                        .wrap(true)
                                        .show();
                                });
                            ui.text("}")
                                .size(state.font_size)
                                .color(colors.text_primary)
                                .show();
                        });
                });
        });
}

fn draw_editor_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .column()
        .gap(16.0)
        .show(|ui| {
            ui.container()
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Editor Configurations")
                        .size(16.0)
                        .font_weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Configure editing canvas behavior, gutter indicators, and layout scale.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .wrap(true)
                        .show();
                });

            // Line numbers toggle (using Zenthra Toggle Widget)
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border((colors.border.with_alpha(0.20), 1.0))
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            ui.text("Line Numbers Gutter")
                                .size(13.0)
                                .font_weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Display row index numbers along the left edge of the buffer")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    let mut ln = state.line_numbers_visible;
                    let toggle_resp = ui.toggle(&mut ln, None)
                        .width(42.0)
                        .height(22.0)
                        .pill()
                        .colors(colors.accent, colors.highlight, Color::WHITE)
                        .show();

                    if toggle_resp.clicked {
                        state.line_numbers_visible = ln;
                        state.save_persistent_state();
                        ui.needs_redraw = true;
                    }
                });

            // Cursor Style (Block, Line, Underline)
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border((colors.border.with_alpha(0.20), 1.0))
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            ui.text("Cursor Style")
                                .size(13.0)
                                .font_weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Choose cursor appearance for both code buffer and terminal")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    // Segmented pill control
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(2.0)
                        .padding(2.0, 3.0, 2.0, 3.0)
                        .radius_all(6.0)
                        .bg(colors.highlight)
                        .border((colors.border.with_alpha(0.35), 1.0))
                        .show(|ui| {
                            for style in CursorStyle::ALL {
                                let is_sel = state.cursor_style == style;
                                let chip = ui.container()
                                    .interactive(true)
                                    .padding(4.0, 12.0, 4.0, 12.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent } else { Color::TRANSPARENT })
                                    .hover(Style::bg(if is_sel { colors.accent } else { colors.accent.with_alpha(0.12) }))
                                    .show(|ui| {
                                        ui.text(style.display_name())
                                            .size(11.5)
                                            .font_weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
                                            .color(if is_sel { Color::WHITE } else { colors.text_muted })
                                            .show();
                                    });
                                if chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if chip.clicked {
                                    state.cursor_style = style;
                                    state.save_persistent_state();
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });

            // Cursor Color (White, Match Accent, Custom Palette)
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border((colors.border.with_alpha(0.20), 1.0))
                .column()
                .gap(10.0)
                .show(|ui| {
                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .show(|ui| {
                            ui.container()
                                .column()
                                .gap(3.0)
                                .show(|ui| {
                                    ui.text("Cursor Color")
                                        .size(13.0)
                                        .font_weight(FontWeight::SemiBold)
                                        .color(colors.text_primary)
                                        .show();
                                    ui.text("Default is white, match theme accent, or choose a custom accent")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .wrap(true)
                                        .show();
                                });

                            // Live cursor indicator preview matching exact style and color
                            let cur_color = state.get_cursor_color(colors.accent);
                            ui.container()
                                .padding(4.0, 10.0, 4.0, 10.0)
                                .radius_all(5.0)
                                .bg(colors.bg_base)
                                .border((colors.border.with_alpha(0.35), 1.0))
                                .row()
                                .valign(Align::Center)
                                .gap(8.0)
                                .show(|ui| {
                                    match state.cursor_style {
                                        CursorStyle::Block => {
                                            ui.container()
                                                .width(8.0)
                                                .height(14.0)
                                                .radius_all(1.0)
                                                .bg(cur_color)
                                                .show(|_| {});
                                        }
                                        CursorStyle::Line => {
                                            ui.container()
                                                .width(2.0)
                                                .height(14.0)
                                                .radius_all(0.5)
                                                .bg(cur_color)
                                                .show(|_| {});
                                        }
                                        CursorStyle::Underline => {
                                            ui.container()
                                                .width(10.0)
                                                .height(3.0)
                                                .radius_all(0.5)
                                                .bg(cur_color)
                                                .show(|_| {});
                                        }
                                    }
                                    ui.text("Preview")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                        });

                    ui.container()
                        .fill_x()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .wrap(Wrap::Wrap)
                        .show(|ui| {
                            // 1. Default White / Auto High-Contrast
                            let is_white = state.cursor_color_mode == CursorColorMode::White;
                            let def_color = if state.theme == crate::theme::ThemeMode::Light {
                                Color::rgb(0.12, 0.12, 0.14)
                            } else {
                                Color::WHITE
                            };
                            let def_label = if state.theme == crate::theme::ThemeMode::Light {
                                "Default (Dark)"
                            } else {
                                "White (Default)"
                            };

                            let white_chip = ui.container()
                                .interactive(true)
                                .padding(5.0, 12.0, 5.0, 12.0)
                                .radius_all(5.0)
                                .bg(if is_white { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                .border((if is_white { colors.accent } else { colors.border.with_alpha(0.30) }, 1.0))
                                .hover(Style::bg(if is_white { colors.accent.with_alpha(0.30) } else { colors.accent.with_alpha(0.10) }))
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    ui.container()
                                        .width(9.0)
                                        .height(9.0)
                                        .radius_all(4.5)
                                        .bg(def_color)
                                        .show(|_| {});
                                    ui.text(def_label)
                                        .size(11.5)
                                        .font_weight(if is_white { FontWeight::SemiBold } else { FontWeight::Regular })
                                        .color(if is_white { colors.text_primary } else { colors.text_muted })
                                        .show();
                                });
                            if white_chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                            if white_chip.clicked {
                                state.cursor_color_mode = CursorColorMode::White;
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }

                            // 2. Match Accent
                            let is_accent = state.cursor_color_mode == CursorColorMode::Accent;
                            let accent_chip = ui.container()
                                .interactive(true)
                                .padding(5.0, 12.0, 5.0, 12.0)
                                .radius_all(5.0)
                                .bg(if is_accent { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                .border((if is_accent { colors.accent } else { colors.border.with_alpha(0.30) }, 1.0))
                                .hover(Style::bg(if is_accent { colors.accent.with_alpha(0.30) } else { colors.accent.with_alpha(0.10) }))
                                .row()
                                .valign(Align::Center)
                                .gap(6.0)
                                .show(|ui| {
                                    ui.container()
                                        .width(9.0)
                                        .height(9.0)
                                        .radius_all(4.5)
                                        .bg(colors.accent)
                                        .show(|_| {});
                                    ui.text("Match Accent")
                                        .size(11.5)
                                        .font_weight(if is_accent { FontWeight::SemiBold } else { FontWeight::Regular })
                                        .color(if is_accent { colors.text_primary } else { colors.text_muted })
                                        .show();
                                });
                            if accent_chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                            if accent_chip.clicked {
                                state.cursor_color_mode = CursorColorMode::Accent;
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }

                            // Subtle vertical separator
                            ui.container()
                                .width(1.0)
                                .height(20.0)
                                .bg(colors.border.with_alpha(0.30))
                                .show(|_| {});

                            // 3. Custom color palette using theme provider options
                            for (id, _name) in ACCENT_COLOR_OPTIONS {
                                let col = named_color(id, state.theme);
                                let is_active = match &state.cursor_color_mode {
                                    CursorColorMode::Custom(c) => c == *id,
                                    _ => false,
                                };

                                let swatch = ui.container()
                                    .interactive(true)
                                    .width(22.0)
                                    .height(22.0)
                                    .radius_all(11.0)
                                    .bg(col)
                                    .border((
                                        if is_active {
                                            colors.text_primary
                                        } else {
                                            colors.border.with_alpha(0.25)
                                        },
                                        if is_active { 2.0 } else { 1.0 },
                                    ))
                                    .valign(Align::Center)
                                    .halign(Align::Center)
                                    .show(|ui| {
                                        if is_active {
                                            ui.container()
                                                .width(5.0)
                                                .height(5.0)
                                                .radius_all(2.5)
                                                .bg(if col.r * 0.299 + col.g * 0.587 + col.b * 0.114 > 0.6 {
                                                    Color::BLACK
                                                } else {
                                                    Color::WHITE
                                                })
                                                .show(|_| {});
                                        }
                                    });
                                if swatch.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if swatch.clicked {
                                    state.cursor_color_mode = CursorColorMode::Custom(id.to_string());
                                    state.save_persistent_state();
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });

            // Zoom level slider (using Zenthra Slider Widget)
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border((colors.border.with_alpha(0.20), 1.0))
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(7.0)
                                .show(|ui| {
                                    ui.text("Interface Zoom Level")
                                        .size(13.0)
                                        .font_weight(FontWeight::SemiBold)
                                        .color(colors.text_primary)
                                        .show();
                                    if (state.zoom_level - 1.0).abs() < 0.01 {
                                        ui.container()
                                            .padding(2.0, 5.0, 2.0, 5.0)
                                            .radius_all(4.0)
                                            .bg(colors.accent.with_alpha(0.15))
                                            .show(|ui| {
                                                ui.text("Default")
                                                    .size(9.0)
                                                    .font_weight(FontWeight::SemiBold)
                                                    .color(colors.accent)
                                                    .show();
                                            });
                                    }
                                });
                            ui.text("Overall editor scaling factor (Ctrl + / - / 0)")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            let mut zoom_val = state.zoom_level * 100.0;
                            let slider_resp = ui.slider(&mut zoom_val, "zoom_slider")
                                .range(50.0, 200.0)
                                .step(5.0)
                                .track_size(150.0, 5.0)
                                .track_radius(2.5, 2.5, 2.5, 2.5)
                                .track_color(colors.highlight)
                                .thumb_size(14.0, 14.0)
                                .thumb_radius(7.0, 7.0, 7.0, 7.0)
                                .thumb_color(colors.accent)
                                .show();

                            if (zoom_val - state.zoom_level * 100.0).abs() > 0.1 || slider_resp.clicked || slider_resp.pressed {
                                state.set_zoom_level((zoom_val / 100.0).clamp(0.5, 2.0));
                                ui.needs_redraw = true;
                            }

                            ui.container()
                                .width(44.0)
                                .halign(Align::Right)
                                .show(|ui| {
                                    ui.text(&format!("{:.0}%", state.zoom_level * 100.0))
                                        .size(12.0)
                                        .font_weight(FontWeight::Bold)
                                        .color(colors.accent)
                                        .show();
                                });

                            // Reset button beside the zoom slider
                            let is_not_default = (state.zoom_level - 1.0).abs() >= 0.01;
                            let reset_btn = ui.container()
                                .interactive(is_not_default)
                                .padding(3.0, 8.0, 3.0, 8.0)
                                .radius_all(4.0)
                                .bg(if is_not_default {
                                    colors.accent.with_alpha(0.14)
                                } else {
                                    colors.bg_base.with_alpha(0.35)
                                })
                                .border((
                                    if is_not_default {
                                        colors.accent.with_alpha(0.40)
                                    } else {
                                        colors.border.with_alpha(0.20)
                                    },
                                    1.0,
                                ))
                                .hover(Style::bg(if is_not_default {
                                    colors.accent.with_alpha(0.26)
                                } else {
                                    colors.bg_base.with_alpha(0.35)
                                }))
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .show(|ui| {
                                    ui.text("↺")
                                        .size(11.0)
                                        .font_weight(FontWeight::Bold)
                                        .color(if is_not_default {
                                            colors.accent
                                        } else {
                                            colors.text_muted.with_alpha(0.40)
                                        })
                                        .show();
                                    ui.text("Reset")
                                        .size(10.5)
                                        .font_weight(FontWeight::SemiBold)
                                        .color(if is_not_default {
                                            colors.text_primary
                                        } else {
                                            colors.text_muted.with_alpha(0.40)
                                        })
                                        .show();
                                });

                            if is_not_default && reset_btn.hovered {
                                ui.cursor_icon = CursorIcon::Pointer;
                            }
                            if is_not_default && reset_btn.clicked {
                                state.reset_zoom();
                                ui.needs_redraw = true;
                            }
                        });
                });
        });
}

fn draw_terminal_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .column()
        .gap(16.0)
        .show(|ui| {
            ui.container()
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Integrated Terminal")
                        .size(16.0)
                        .font_weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Configure native terminal panel defaults and height.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .wrap(true)
                        .show();
                });

            // Default height slider (using Zenthra Slider Widget)
            ui.container()
                .fill_x()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border((colors.border.with_alpha(0.20), 1.0))
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.text("Terminal Panel Height")
                                .size(13.0)
                                .font_weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Default opened height in pixels")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            let mut th_val = state.terminal_height;
                            let slider_resp = ui.slider(&mut th_val, "terminal_h_slider")
                                .range(100.0, 450.0)
                                .step(10.0)
                                .track_size(160.0, 5.0)
                                .track_radius(2.5, 2.5, 2.5, 2.5)
                                .track_color(colors.highlight)
                                .thumb_size(14.0, 14.0)
                                .thumb_radius(7.0, 7.0, 7.0, 7.0)
                                .thumb_color(colors.accent)
                                .show();

                            if slider_resp.clicked || slider_resp.pressed {
                                state.terminal_height = th_val;
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }

                            ui.container()
                                .width(48.0)
                                .halign(Align::Right)
                                .show(|ui| {
                                    ui.text(&format!("{:.0}px", state.terminal_height))
                                        .size(12.0)
                                        .font_weight(FontWeight::Bold)
                                        .color(colors.accent)
                                        .show();
                                });
                        });
                });
        });
}

fn draw_agent_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .column()
        .gap(16.0)
        .show(|ui| {
            ui.container()
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("AI & Agent Integration")
                        .size(16.0)
                        .font_weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Configure AI assistant models, autonomous agents, and tool execution.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .wrap(true)
                        .show();
                });

            ui.container()
                .fill_x()
                .padding(12.0, 16.0, 12.0, 16.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border((colors.border.with_alpha(0.20), 1.0))
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            ui.text("Advanced AI & Agent Settings")
                                .size(13.0)
                                .font_weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Provider API keys, thinking budget, system prompts, and custom tools")
                                .size(11.0)
                                .color(colors.text_muted)
                                .wrap(true)
                                .show();
                        });

                    let open_btn = ui.container()
                        .interactive(true)
                        .padding(7.0, 16.0, 7.0, 16.0)
                        .radius_all(6.0)
                        .bg(colors.accent)
                        .show(|ui| {
                            ui.text("Open AI Dialog")
                                .size(11.5)
                                .font_weight(FontWeight::Bold)
                                .color(Color::WHITE)
                                .show();
                        });

                    if open_btn.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if open_btn.clicked {
                        state.editor_settings_open = false;
                        state.ai_settings.open = true;
                        state.ai_settings.just_opened = true;
                        ui.needs_redraw = true;
                    }
                });
        });
}

fn draw_keymaps_tab(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    let card_panel_bg = if state.glassmorphism {
        colors.bg_base.with_alpha((state.glass_opacity * 0.65).clamp(0.20, 0.70))
    } else {
        colors.bg_base.with_alpha(0.50)
    };

    ui.container()
        .column()
        .gap(14.0)
        .show(|ui| {
            ui.container()
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Keyboard Shortcuts")
                        .size(16.0)
                        .font_weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Complete keybindings reference for fast navigation and productivity. VSCode & Neovim friendly.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .wrap(true)
                        .show();
                });

            let sections: [(&str, &str, &[(&str, &str)]); 6] = [
                (
                    "\u{ebeb}",
                    "Panels & View Modes",
                    &[
                        ("Toggle Explorer (File Tree)", "Ctrl + E"),
                        ("Toggle Integrated Terminal", "Ctrl + T"),
                        ("Toggle Git Panel", "Ctrl + Shift + G"),
                        ("Toggle AI Agent Panel", "Ctrl + Shift + A"),
                        ("Toggle Focus Mode (Zen)", "F11"),
                        ("Zoom In / Out / Reset", "Ctrl + = / - / 0"),
                    ],
                ),
                (
                    "\u{f04e9}",
                    "Buffer Tabs & Navigation",
                    &[
                        ("Jump to Active Editor", "Ctrl + 1"),
                        ("Unfocus Panel → Return to Editor", "Escape"),
                        ("Previous Tab", "Alt + ←"),
                        ("Next Tab", "Alt + →"),
                        ("Close Current Tab", "Alt + W"),
                        ("Open Preferences / Settings", "Ctrl + ,"),
                    ],
                ),
                (
                    "\u{f120}",
                    "Command Palette & LazyVim",
                    &[
                        ("Open Command Palette / Dialog", "Ctrl + Shift + P"),
                        ("Open LazyVim Command Mode", "Shift + : or :"),
                        ("Directory Change with Autocomplete", "cd <path>"),
                        ("New File / New Folder", "touch <f> / mkdir <d>"),
                        ("Save File / Close Tab", ":w / :q / :wq"),
                        ("Execute Shell Command", "!<command>"),
                    ],
                ),
                (
                    "\u{ec2b}",
                    "Code Editor",
                    &[
                        ("Save Active File", "Ctrl + S"),
                        ("Save As", "Ctrl + Shift + S"),
                        ("Undo / Redo", "Ctrl + Z / Ctrl + Y"),
                        ("Cut / Copy / Paste", "Ctrl + X / C / V"),
                        ("Select All", "Ctrl + A"),
                        ("Select Word Under Cursor", "Ctrl + D"),
                        ("Duplicate Line Down", "Ctrl + Shift + D"),
                        ("Delete Entire Line", "Ctrl + Shift + K"),
                        ("Delete Word Left / Right", "Ctrl + Backspace / Delete"),
                        ("Word Navigation Left / Right", "Ctrl + ← / →"),
                        ("Find in File", "Ctrl + F"),
                        ("Global / File Search", "Ctrl + Shift + F"),
                        ("Trigger Autocomplete", "Ctrl + Space"),
                    ],
                ),
                (
                    "\u{ef81}",
                    "File Tree (When Focused)",
                    &[
                        ("Navigate Down / Up", "j / k (or ↓ / ↑)"),
                        ("Open File / Expand Folder", "l / Enter (or →)"),
                        ("Collapse Folder / Parent", "h (or ←)"),
                        ("Create File / Folder (Dialog)", "a / A"),
                        ("Rename Selected", "r or F2"),
                        ("Delete Selected (Confirm)", "d or Delete"),
                        ("Refresh File Tree", "R"),
                        ("Back to Editor", "q or Escape"),
                    ],
                ),
                (
                    "\u{f489}",
                    "Integrated Terminal (When Focused)",
                    &[
                        ("Word Navigation Left / Right", "Ctrl + ← / →"),
                        ("Scroll History (15 Lines)", "Shift + PgUp / PgDn"),
                        ("Scroll History (1 Line)", "Shift + ↑ / ↓"),
                        ("Scroll to Top / Bottom", "Shift + Home / End"),
                        ("Copy Output to Clipboard", "Ctrl + Shift + C"),
                        ("Paste from Clipboard", "Ctrl + Shift + V / Ctrl + V"),
                        ("Clear Terminal & Scrollback", "Ctrl + Shift + K"),
                        ("Interrupt (SIGINT)", "Ctrl + C"),
                        ("EOF / Exit Session", "Ctrl + D"),
                        ("Clear Screen", "Ctrl + L"),
                        ("Reverse History Search", "Ctrl + R"),
                        ("Previous / Next History", "Ctrl + P / Ctrl + N"),
                    ],
                ),
            ];

            for (icon, section_title, items) in sections {
                ui.container()
                    .column()
                    .gap(6.0)
                    .show(|ui| {
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text(icon)
                                    .size(13.0)
                                    .color(colors.accent)
                                    .show();
                                ui.text(section_title)
                                    .size(13.0)
                                    .font_weight(FontWeight::SemiBold)
                                    .color(colors.text_primary)
                                    .show();
                            });

                        ui.container()
                            .fill_x()
                            .radius_all(6.0)
                            .bg(card_panel_bg)
                            .border((colors.border.with_alpha(0.20), 1.0))
                            .padding(4.0, 12.0, 4.0, 12.0)
                            .column()
                            .gap(2.0)
                            .show(|ui| {
                                for (i, (action, keys)) in items.iter().enumerate() {
                                    if i > 0 {
                                        ui.container()
                                            .fill_x()
                                            .height(1.0)
                                            .bg(colors.border.with_alpha(0.10))
                                            .show(|_| {});
                                    }

                                    ui.container()
                                        .fill_x()
                                        .padding(5.0, 4.0, 5.0, 4.0)
                                        .row()
                                        .valign(Align::Center)
                                        .halign(Align::SpaceBetween)
                                        .show(|ui| {
                                            ui.text(*action)
                                                .size(12.0)
                                                .color(colors.text_primary)
                                                .show();

                                            ui.container()
                                                .padding(2.5, 8.0, 2.5, 8.0)
                                                .radius_all(4.0)
                                                .bg(colors.highlight)
                                                .border((colors.border.with_alpha(0.25), 1.0))
                                                .show(|ui| {
                                                    ui.text(*keys)
                                                        .size(11.0)
                                                        .font_weight(FontWeight::SemiBold)
                                                        .color(colors.accent)
                                                        .show();
                                                });
                                        });
                                }
                            });
                    });
            }
        });
}
