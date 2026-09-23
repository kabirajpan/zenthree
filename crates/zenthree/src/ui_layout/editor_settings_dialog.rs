use crate::state::{EditorSettingsTab, ZenthreeState};
use crate::theme::ThemeColors;
use zenthra::{Align, Color, CursorIcon, FontWeight, Id, Ui};

pub fn draw_editor_settings_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.editor_settings_open {
        return;
    }

    let colors = state.colors();

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

    if close_on_esc {
        state.editor_settings_open = false;
        state.save_persistent_state();
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
            state.save_persistent_state();
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
                                .weight(FontWeight::Bold)
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
                        .clickable(true)
                        .padding(6.0, 10.0, 6.0, 10.0)
                        .radius_all(6.0)
                        .hover_bg(colors.highlight)
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
                .full_width()
                .height(1.0)
                .bg(colors.border.with_alpha(0.25))
                .show(|_| {});

            // Body: Left Sidebar + Right Scrollable Content Area
            let body_h = win_h - 47.0;
            let sidebar_w = 205.0;
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
                        .padding(14.0, 10.0, 14.0, 10.0)
                        .column()
                        .gap(3.0)
                        .show(|ui| {
                            for tab in &EditorSettingsTab::ALL {
                                let is_active = state.editor_settings_tab == *tab;
                                let tab_btn = ui.container()
                                    .full_width()
                                    .clickable(true)
                                    .padding(5.5, 10.0, 5.5, 10.0)
                                    .radius_all(5.0)
                                    .bg(if is_active { colors.highlight.with_alpha(0.06) } else { Color::TRANSPARENT })
                                    .hover_bg(colors.highlight)
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
                                            .weight(if is_active { FontWeight::SemiBold } else { FontWeight::Regular })
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
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.text("Appearance & Frosted Glass")
                                .size(15.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Wayland/OS frosted glassmorphism, blur strength, panel opacity, and editor typography.")
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    let reset_all = ui.container()
                        .clickable(true)
                        .padding(4.0, 10.0, 4.0, 10.0)
                        .radius_all(5.0)
                        .bg(colors.highlight)
                        .hover_bg(colors.accent.with_alpha(0.12))
                        .border(colors.border.with_alpha(0.30), 1.0)
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .show(|ui| {
                            ui.text("↺")
                                .size(12.0)
                                .color(colors.accent)
                                .show();
                            ui.text("Reset Defaults")
                                .size(11.0)
                                .weight(FontWeight::Medium)
                                .color(colors.text_primary)
                                .show();
                        });

                    if reset_all.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                    if reset_all.clicked {
                        state.set_blur_strength(24.0);
                        state.set_glass_opacity(0.55);
                        state.set_font_size(13.0);
                        state.glassmorphism = true;
                        ui.set_blur(true);
                        ui.needs_redraw = true;
                    }
                });

            // 1. Frosted Glassmorphism Switch (using Zenthra Toggle Widget)
            ui.container()
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border(colors.border.with_alpha(0.20), 1.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.text("Frosted Glassmorphism")
                                .size(13.0)
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Enable Wayland ext-background-effect frosted blur & translucent surfaces")
                                .size(11.0)
                                .color(colors.text_muted)
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
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border(colors.border.with_alpha(0.20), 1.0)
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
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
                                            ui.text("Frosted Blur Strength")
                                                .size(13.0)
                                                .weight(FontWeight::SemiBold)
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
                                                            .weight(FontWeight::SemiBold)
                                                            .color(colors.accent)
                                                            .show();
                                                    });
                                            }
                                        });
                                    ui.text("Background blur dispersion radius (Default: 24px)")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    // Decrement stepper
                                    let dec_btn = ui.container()
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
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                });
                        });

                    // Preset chips + Reset button
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
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
                                    .clickable(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                    .border(if is_sel { colors.accent } else { Color::TRANSPARENT }, 1.0)
                                    .show(|ui| {
                                        ui.text(&chip_text)
                                            .size(10.0)
                                            .weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
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
                                    .clickable(true)
                                    .padding(3.0, 7.0, 3.0, 7.0)
                                    .radius_all(4.0)
                                    .bg(colors.highlight.with_alpha(0.5))
                                    .border(colors.border.with_alpha(0.3), 1.0)
                                    .show(|ui| {
                                        ui.text("↺ 24px")
                                            .size(9.5)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                                if reset_chip.hovered { ui.cursor_icon = CursorIcon::Pointer; }
                                if reset_chip.clicked {
                                    state.set_blur_strength(24.0);
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });

            // 3. Surface Opacity / Frost Depth Slider with Steppers & Presets
            ui.container()
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border(colors.border.with_alpha(0.20), 1.0)
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
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
                                            ui.text("Surface Opacity (Frost Depth)")
                                                .size(13.0)
                                                .weight(FontWeight::SemiBold)
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
                                                            .weight(FontWeight::SemiBold)
                                                            .color(colors.accent)
                                                            .show();
                                                    });
                                            }
                                        });
                                    ui.text("Translucency of editor and sidebar panels (Default: 55%)")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    // Decrement stepper
                                    let dec_btn = ui.container()
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
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                });
                        });

                    // Preset chips + Reset button
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
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
                                    .clickable(true)
                                    .padding(3.0, 8.0, 3.0, 8.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                    .border(if is_sel { colors.accent } else { Color::TRANSPARENT }, 1.0)
                                    .show(|ui| {
                                        ui.text(&chip_text)
                                            .size(10.0)
                                            .weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
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
                                    .clickable(true)
                                    .padding(3.0, 7.0, 3.0, 7.0)
                                    .radius_all(4.0)
                                    .bg(colors.highlight.with_alpha(0.5))
                                    .border(colors.border.with_alpha(0.3), 1.0)
                                    .show(|ui| {
                                        ui.text("↺ 55%")
                                            .size(9.5)
                                            .color(colors.text_muted)
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
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(card_panel_bg)
                .border(colors.border.with_alpha(0.20), 1.0)
                .column()
                .gap(8.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
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
                                                .weight(FontWeight::SemiBold)
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
                                                            .weight(FontWeight::SemiBold)
                                                            .color(colors.accent)
                                                            .show();
                                                    });
                                            }
                                        });
                                    ui.text("Editor buffer text size and line height (Default: 13px)")
                                        .size(11.0)
                                        .color(colors.text_muted)
                                        .show();
                                });

                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(5.0)
                                .show(|ui| {
                                    // Decrement stepper
                                    let dec_btn = ui.container()
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
                                                .weight(FontWeight::Bold)
                                                .color(colors.accent)
                                                .show();
                                        });
                                });
                        });

                    // Preset chips + Reset button
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
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
                                    .clickable(true)
                                    .padding(3.0, 7.0, 3.0, 7.0)
                                    .radius_all(4.0)
                                    .bg(if is_sel { colors.accent.with_alpha(0.20) } else { colors.highlight })
                                    .border(if is_sel { colors.accent } else { Color::TRANSPARENT }, 1.0)
                                    .show(|ui| {
                                        ui.text(&chip_text)
                                            .size(10.0)
                                            .weight(if is_sel { FontWeight::SemiBold } else { FontWeight::Regular })
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
                                    .clickable(true)
                                    .padding(3.0, 7.0, 3.0, 7.0)
                                    .radius_all(4.0)
                                    .bg(colors.highlight.with_alpha(0.5))
                                    .border(colors.border.with_alpha(0.3), 1.0)
                                    .show(|ui| {
                                        ui.text("↺ 13px")
                                            .size(9.5)
                                            .color(colors.text_muted)
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
                        .full_width()
                        .padding(8.0, 12.0, 8.0, 12.0)
                        .radius_all(5.0)
                        .bg(colors.bg_editor)
                        .border(colors.border.with_alpha(0.25), 1.0)
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
                                        .weight(FontWeight::SemiBold)
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
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Configure editing canvas behavior, gutter indicators, and layout scale.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });

            // Line numbers toggle (using Zenthra Toggle Widget)
            ui.container()
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border(colors.border.with_alpha(0.20), 1.0)
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
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Display row index numbers along the left edge of the buffer")
                                .size(11.0)
                                .color(colors.text_muted)
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

            // Zoom level slider (using Zenthra Slider Widget)
            ui.container()
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border(colors.border.with_alpha(0.20), 1.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .column()
                        .gap(2.0)
                        .show(|ui| {
                            ui.text("Interface Zoom Level")
                                .size(13.0)
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Overall editor scaling factor")
                                .size(11.0)
                                .color(colors.text_muted)
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
                                .track_size(160.0, 5.0)
                                .track_radius(2.5, 2.5, 2.5, 2.5)
                                .track_color(colors.highlight)
                                .thumb_size(14.0, 14.0)
                                .thumb_radius(7.0, 7.0, 7.0, 7.0)
                                .thumb_color(colors.accent)
                                .show();

                            if slider_resp.clicked || slider_resp.pressed {
                                state.zoom_level = (zoom_val / 100.0).clamp(0.5, 2.0);
                                state.save_persistent_state();
                                ui.needs_redraw = true;
                            }

                            ui.container()
                                .width(48.0)
                                .halign(Align::Right)
                                .show(|ui| {
                                    ui.text(&format!("{:.0}%", state.zoom_level * 100.0))
                                        .size(12.0)
                                        .weight(FontWeight::Bold)
                                        .color(colors.accent)
                                        .show();
                                });
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
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Configure native terminal panel defaults and height.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });

            // Default height slider (using Zenthra Slider Widget)
            ui.container()
                .full_width()
                .padding(10.0, 14.0, 10.0, 14.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border(colors.border.with_alpha(0.20), 1.0)
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
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Default opened height in pixels")
                                .size(11.0)
                                .color(colors.text_muted)
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
                                        .weight(FontWeight::Bold)
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
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Configure AI assistant models, autonomous agents, and tool execution.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });

            ui.container()
                .full_width()
                .padding(12.0, 16.0, 12.0, 16.0)
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border(colors.border.with_alpha(0.20), 1.0)
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
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                            ui.text("Provider API keys, thinking budget, system prompts, and custom tools")
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    let open_btn = ui.container()
                        .clickable(true)
                        .padding(7.0, 16.0, 7.0, 16.0)
                        .radius_all(6.0)
                        .bg(colors.accent)
                        .show(|ui| {
                            ui.text("Open AI Dialog")
                                .size(11.5)
                                .weight(FontWeight::Bold)
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

fn draw_keymaps_tab(ui: &mut Ui, _state: &mut ZenthreeState, colors: &ThemeColors) {
    ui.container()
        .column()
        .gap(16.0)
        .show(|ui| {
            ui.container()
                .column()
                .gap(3.0)
                .show(|ui| {
                    ui.text("Keyboard Shortcuts")
                        .size(16.0)
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                    ui.text("Essential shortcuts for fast navigation and productivity.")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });

            let shortcuts = [
                ("Preferences / Settings", "Ctrl + ,"),
                ("New File", "Ctrl + N"),
                ("Open File", "Ctrl + O"),
                ("Open Workspace Folder", "Ctrl + Shift + O"),
                ("Save Active File", "Ctrl + S"),
                ("Save As", "Ctrl + Shift + S"),
                ("Close Active Tab", "Ctrl + W"),
                ("Format Buffer", "Ctrl + Shift + F"),
                ("Find in Buffer", "Ctrl + F"),
                ("Toggle Integrated Terminal", "Ctrl + `"),
                ("Toggle AI Agent Panel", "Ctrl + K"),
            ];

            ui.container()
                .full_width()
                .radius_all(6.0)
                .bg(colors.bg_base.with_alpha(0.40))
                .border(colors.border.with_alpha(0.20), 1.0)
                .padding(8.0, 16.0, 8.0, 16.0)
                .column()
                .gap(6.0)
                .show(|ui| {
                    for (action, keys) in shortcuts {
                        ui.container()
                            .full_width()
                            .padding(6.0, 4.0, 6.0, 4.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.text(action)
                                    .size(12.5)
                                    .color(colors.text_primary)
                                    .show();

                                ui.container()
                                    .padding(3.0, 9.0, 3.0, 9.0)
                                    .radius_all(4.0)
                                    .bg(colors.highlight)
                                    .border(colors.border.with_alpha(0.25), 1.0)
                                    .show(|ui| {
                                        ui.text(keys)
                                            .size(11.0)
                                            .weight(FontWeight::SemiBold)
                                            .color(colors.accent)
                                            .show();
                                    });
                            });
                    }
                });
        });
}
