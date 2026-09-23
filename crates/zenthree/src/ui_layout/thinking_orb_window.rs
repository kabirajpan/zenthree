use crate::state::ZenthreeState;
use crate::theme::ThemeColors;
use zenthra::{Align, Color, CursorIcon, FontWeight, Id, Ui};
use zenthra_state::use_context;

pub fn draw_thinking_orb_window(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.orb_window_open {
        return;
    }

    let colors = use_context::<ThemeColors>()
        .map(|c| *c)
        .unwrap_or_else(|| state.colors());

    let win_w = 210.0;
    let win_h = 224.0;
    let win_radius = 12.0;

    // Initialize position on first open
    if state.orb_window_pos.0 <= 1.0 && state.orb_window_pos.1 <= 1.0 {
        state.orb_window_pos = ((ui.width - win_w - 24.0).max(20.0), 64.0);
    }

    let mut pos_x = state.orb_window_pos.0;
    let mut pos_y = state.orb_window_pos.1;

    // Clamp inside viewport
    pos_x = pos_x.clamp(10.0, (ui.width - win_w - 10.0).max(10.0));
    pos_y = pos_y.clamp(10.0, (ui.height - win_h - 10.0).max(10.0));
    state.orb_window_pos = (pos_x, pos_y);

    // Card background: frosted glass or solid sidebar
    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 1.15).clamp(0.45, 0.95))
    } else {
        colors.bg_sidebar
    };

    let mut card = ui.container()
        .id(Id::from_u64(999999880))
        .absolute(pos_x, pos_y)
        .overlay()
        .width(win_w)
        .height(win_h)
        .bg(card_bg)
        .border(colors.border.with_alpha(0.45), 1.0)
        .radius_all(win_radius)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.40), 0.0, 8.0, 24.0);

    if state.glassmorphism {
        card = card.backdrop_blur(state.blur_strength.max(4.0));
    }

    let mut do_close = false;
    let mut do_open_ai_panel = false;

    card.column()
        .clip(true)
        .show(|ui| {
            // ── Top Header / Drag Handle ──────────────────────────────────────
            const HEADER_H: f32 = 30.0;
            let header_bg = colors.highlight.with_alpha(0.02);

            let header_resp = ui.container()
                .id(Id::from_u64(999999881))
                .clickable(true)
                .full_width()
                .height(HEADER_H)
                .radius_top(win_radius)
                .bg(header_bg)
                .padding_left(10.0)
                .padding_right(8.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left: Pulsing status indicator dot + Title
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            let is_working = state.is_ai_working();
                            let dot_col = if is_working {
                                Color::rgb(250.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0) // Amber pulse
                            } else {
                                colors.accent
                            };
                            ui.container()
                                .width(7.0)
                                .height(7.0)
                                .radius_all(3.5)
                                .bg(dot_col)
                                .show(|_| {});

                            ui.text("Thinking Orb")
                                .size(11.0)
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Right: Close button
                    let close_btn = ui.container()
                        .id(Id::from_u64(999999882))
                        .clickable(true)
                        .width(18.0)
                        .height(18.0)
                        .radius_all(3.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(colors.highlight)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("✕")
                                .size(9.5)
                                .color(colors.text_muted)
                                .show();
                        });
                    if close_btn.hovered {
                        ui.cursor_icon = CursorIcon::Pointer;
                    }
                    if close_btn.clicked {
                        do_close = true;
                    }
                });

            // Thin separator replacing border_bottom
            ui.container()
                .full_width()
                .height(1.0)
                .bg(colors.border.with_alpha(0.25))
                .show(|_| {});

            // Dragging interaction
            if header_resp.hovered {
                ui.cursor_icon = CursorIcon::Grab;
            }
            if header_resp.pressed && ui.mouse_down {
                if !state.orb_window_dragging {
                    state.orb_window_dragging = true;
                    state.orb_window_drag_offset = (ui.mouse_x - pos_x, ui.mouse_y - pos_y);
                }
            }
            if !ui.mouse_down {
                state.orb_window_dragging = false;
            }
            if state.orb_window_dragging {
                ui.cursor_icon = CursorIcon::Grabbing;
                let new_x = (ui.mouse_x - state.orb_window_drag_offset.0).clamp(10.0, (ui.width - win_w - 10.0).max(10.0));
                let new_y = (ui.mouse_y - state.orb_window_drag_offset.1).clamp(10.0, (ui.height - win_h - 10.0).max(10.0));
                state.orb_window_pos = (new_x, new_y);
                ui.needs_redraw = true;
            }

            // ── Orb Animation Stage ──────────────────────────────────────────
            // NOTE: .absolute() inside nested .show() closures resolves against
            // prev_global_ox which is 0 at deep nesting → layers were invisible.
            // Fix: reserve a fixed-height flow slot, then draw all orb layers as
            // overlay containers with real screen coords (pos_x + offset).
            let t = state.orb_animation_start.elapsed().as_secs_f32();
            let is_working = state.is_ai_working();
            let speed = if is_working { 3.2 } else { 1.2 };
            let pulse = (t * speed).sin() * 0.5 + 0.5;
            let pulse_slow = (t * (speed * 0.5)).cos() * 0.5 + 0.5;

            // Stage dimensions and center in screen space
            // Header (30) + separator (1) = 31px from card top
            const STAGE_H: f32 = 130.0;
            const STAGE_W: f32 = 210.0; // == win_w
            let stage_screen_cx = pos_x + STAGE_W / 2.0;
            let stage_screen_cy = pos_y + 31.0 + STAGE_H / 2.0;

            // Reserve layout space so subsequent flow items position correctly
            let orb_click_resp = ui.container()
                .id(Id::from_u64(999999889))
                .clickable(true)
                .full_width()
                .height(STAGE_H)
                .bg(Color::TRANSPARENT)
                .show(|_| {});

            if orb_click_resp.hovered {
                ui.cursor_icon = CursorIcon::Pointer;
            }
            if orb_click_resp.clicked {
                do_open_ai_panel = true;
            }

            // ── Layer 1: Outer Ambient Aura (overlay, real screen coords) ────
            let aura_size = 72.0 + pulse * 14.0;
            let aura_color = if is_working {
                Color::rgba(245.0 / 255.0, 140.0 / 255.0, 30.0 / 255.0, 0.16 + pulse * 0.14)
            } else {
                colors.accent.with_alpha(0.09 + pulse * 0.08)
            };
            ui.container()
                .id(Id::from_u64(999999891))
                .overlay()
                .absolute(stage_screen_cx - aura_size / 2.0, stage_screen_cy - aura_size / 2.0)
                .width(aura_size)
                .height(aura_size)
                .radius_all(aura_size / 2.0)
                .bg(aura_color)
                .show(|_| {});

            // ── Layer 2: Harmonic Halo ────────────────────────────────────────
            let halo_size = 56.0 + pulse_slow * 8.0;
            let halo_color = if is_working {
                Color::rgba(230.0 / 255.0, 70.0 / 255.0, 160.0 / 255.0, 0.25 + pulse * 0.15)
            } else {
                Color::rgba(130.0 / 255.0, 90.0 / 255.0, 240.0 / 255.0, 0.18 + pulse * 0.10)
            };
            ui.container()
                .id(Id::from_u64(999999892))
                .overlay()
                .absolute(stage_screen_cx - halo_size / 2.0, stage_screen_cy - halo_size / 2.0)
                .width(halo_size)
                .height(halo_size)
                .radius_all(halo_size / 2.0)
                .bg(halo_color)
                .show(|_| {});

            // ── Layer 3: Core Luminous Sphere ─────────────────────────────────
            let core_size = 42.0;
            let core_color = if is_working {
                Color::rgb(0.95, 0.40 + pulse * 0.35, 0.15 + pulse_slow * 0.30)
            } else {
                colors.accent
            };
            let core_glow = if is_working {
                Color::rgba(1.0, 140.0 / 255.0, 20.0 / 255.0, 0.65)
            } else {
                colors.accent.with_alpha(0.55)
            };
            ui.container()
                .id(Id::from_u64(999999893))
                .overlay()
                .absolute(stage_screen_cx - core_size / 2.0, stage_screen_cy - core_size / 2.0)
                .width(core_size)
                .height(core_size)
                .radius_all(core_size / 2.0)
                .bg(core_color)
                .border(Color::rgba(1.0, 1.0, 1.0, 0.45 + pulse * 0.30), 1.5)
                .shadow(core_glow, 0.0, 0.0, 16.0 + pulse * 10.0)
                .show(|_| {});

            // ── Layer 3b: Specular Glint (top-left highlight) ─────────────────
            ui.container()
                .id(Id::from_u64(999999894))
                .overlay()
                .absolute(
                    stage_screen_cx - core_size / 2.0 + 6.0,
                    stage_screen_cy - core_size / 2.0 + 6.0,
                )
                .width(11.0)
                .height(11.0)
                .radius_all(5.5)
                .bg(Color::rgba(1.0, 1.0, 1.0, 0.70))
                .show(|_| {});

            // ── Layer 4: Orbiting Energy Spark ────────────────────────────────
            let orbit_speed = if is_working { 4.5 } else { 1.6 };
            let angle = t * orbit_speed;
            let orbit_r = 30.0 + pulse * 4.0;
            let dot_size = 5.5;
            let dot_screen_x = stage_screen_cx + angle.cos() * orbit_r - dot_size / 2.0;
            let dot_screen_y = stage_screen_cy + angle.sin() * orbit_r - dot_size / 2.0;
            let dot_color = if is_working {
                Color::rgb(1.0, 0.90, 0.30)
            } else {
                Color::rgba(1.0, 1.0, 1.0, 0.85)
            };
            ui.container()
                .id(Id::from_u64(999999895))
                .overlay()
                .absolute(dot_screen_x, dot_screen_y)
                .width(dot_size)
                .height(dot_size)
                .radius_all(dot_size / 2.0)
                .bg(dot_color)
                .shadow(dot_color, 0.0, 0.0, 6.0)
                .show(|_| {});

            // ── Status Readout & Clickable Action ────────────────────────────
            ui.container()
                .full_width()
                .halign(Align::Center)
                .valign(Align::Center)
                .column()
                .gap(5.0)
                .show(|ui| {
                    let status_pill = ui.container()
                        .clickable(true)
                        .padding(3.5, 10.0, 3.5, 10.0)
                        .radius_all(10.0)
                        .bg(colors.highlight)
                        .hover_bg(colors.highlight.with_alpha(0.08))
                        .border(colors.border.with_alpha(0.35), 0.75)
                        .row()
                        .valign(Align::Center)
                        .gap(5.0)
                        .show(|ui| {
                            let label = if is_working {
                                "Thinking..."
                            } else {
                                "AI Ready"
                            };
                            let icon_color = if is_working {
                                Color::rgb(250.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0)
                            } else {
                                colors.accent
                            };

                            ui.text("●")
                                .size(7.0)
                                .color(icon_color)
                                .show();

                            ui.text(label)
                                .size(10.5)
                                .weight(FontWeight::Medium)
                                .color(colors.text_primary)
                                .show();
                        });

                    if status_pill.hovered {
                        ui.cursor_icon = CursorIcon::Pointer;
                    }
                    if status_pill.clicked {
                        do_open_ai_panel = true;
                    }

                    // Secondary helper text
                    ui.text("Click orb to open AI chat")
                        .size(9.0)
                        .color(colors.text_muted.with_alpha(0.70))
                        .show();
                });
        });

    if do_close {
        state.orb_window_open = false;
        state.orb_window_dragging = false;
        ui.needs_redraw = true;
    }

    if do_open_ai_panel {
        state.ai_panel_visible = true;
        state.save_persistent_state();
        ui.needs_redraw = true;
    }

    // Smooth ~30 FPS animation frame request while window is open
    ui.request_redraw_after(std::time::Duration::from_millis(33));
}

/// Renders the orb as a fixed-width side panel (normal in-flow layout, no
/// absolute/overlay tricks). Call it in a `row()` context next to the AI panel
/// to preview the orb without floating-window coordinate issues.
pub fn draw_orb_side_panel(ui: &mut Ui, state: &mut ZenthreeState, height: f32) {
    let colors = use_context::<ThemeColors>()
        .map(|c| *c)
        .unwrap_or_else(|| state.colors());

    const PANEL_W: f32 = 80.0;

    let panel_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 1.1).clamp(0.4, 0.95))
    } else {
        colors.bg_sidebar
    };

    let mut panel = ui.container()
        .id(Id::from_u64(999999870))
        .width(PANEL_W)
        .height(height)
        .bg(panel_bg)
        .border(colors.border.with_alpha(0.4), 1.0);

    if state.glassmorphism {
        panel = panel.backdrop_blur(state.blur_strength.max(4.0));
    }

    panel.column()
        .halign(Align::Center)
        .valign(Align::Center)
        .gap(10.0)
        .show(|ui| {
            let t = state.orb_animation_start.elapsed().as_secs_f32();
            let is_working = state.is_ai_working();
            let speed = if is_working { 3.2 } else { 1.2 };
            let pulse = (t * speed).sin() * 0.5 + 0.5;
            let pulse_slow = (t * speed * 0.5).cos() * 0.5 + 0.5;

            // ── Stage: fixed 68×68 area, layers as in-flow centered circles ─
            // We draw from outermost (aura) inward so each one paints on top.
            // Using a stack container with centered children.
            let stage_size = 68.0;

            ui.container()
                .id(Id::from_u64(999999871))
                .width(stage_size)
                .height(stage_size)
                .halign(Align::Center)
                .valign(Align::Center)
                .show(|ui| {
                    // Aura — uses padding trick: negative margin via centering
                    let aura_size = 52.0 + pulse * 10.0;
                    let aura_color = if is_working {
                        Color::rgba(245.0 / 255.0, 140.0 / 255.0, 30.0 / 255.0, 0.18 + pulse * 0.14)
                    } else {
                        colors.accent.with_alpha(0.10 + pulse * 0.09)
                    };
                    ui.container()
                        .width(aura_size)
                        .height(aura_size)
                        .radius_all(aura_size / 2.0)
                        .bg(aura_color)
                        .halign(Align::Center)
                        .show(|_| {});

                    // Halo
                    let halo_size = 40.0 + pulse_slow * 6.0;
                    let halo_color = if is_working {
                        Color::rgba(230.0 / 255.0, 70.0 / 255.0, 160.0 / 255.0, 0.28 + pulse * 0.16)
                    } else {
                        Color::rgba(130.0 / 255.0, 90.0 / 255.0, 240.0 / 255.0, 0.20 + pulse * 0.12)
                    };
                    ui.container()
                        .width(halo_size)
                        .height(halo_size)
                        .radius_all(halo_size / 2.0)
                        .bg(halo_color)
                        .halign(Align::Center)
                        .show(|_| {});

                    // Core
                    let core_size = 28.0;
                    let core_color = if is_working {
                        Color::rgb(0.95, 0.40 + pulse * 0.35, 0.15 + pulse_slow * 0.30)
                    } else {
                        colors.accent
                    };
                    let core_glow = if is_working {
                        Color::rgba(1.0, 140.0 / 255.0, 20.0 / 255.0, 0.65)
                    } else {
                        colors.accent.with_alpha(0.55)
                    };
                    ui.container()
                        .width(core_size)
                        .height(core_size)
                        .radius_all(core_size / 2.0)
                        .bg(core_color)
                        .border(Color::rgba(1.0, 1.0, 1.0, 0.40 + pulse * 0.30), 1.5)
                        .shadow(core_glow, 0.0, 0.0, 12.0 + pulse * 8.0)
                        .halign(Align::Center)
                        .show(|_| {});
                });

            // Status label
            let label = if is_working { "Thinking" } else { "Ready" };
            let label_color = if is_working {
                Color::rgb(250.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0)
            } else {
                colors.accent
            };
            ui.container()
                .row()
                .halign(Align::Center)
                .valign(Align::Center)
                .gap(4.0)
                .show(|ui| {
                    ui.container()
                        .width(5.0)
                        .height(5.0)
                        .radius_all(2.5)
                        .bg(label_color)
                        .show(|_| {});
                    ui.text(label)
                        .size(9.0)
                        .weight(FontWeight::Medium)
                        .color(colors.text_muted)
                        .show();
                });
        });

    // Keep animating
    ui.request_redraw_after(std::time::Duration::from_millis(33));
}
