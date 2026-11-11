use crate::state::ZenthreeState;
use crate::theme::ThemeColors;
use crate::ui_layout::thinking_orb_engine::OrbMode;
use zenthra::{Align, Color, CursorIcon, FontWeight, Id, Ui};
use zenthra_state::use_context;

#[allow(dead_code)]
pub fn draw_thinking_orb_window(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.orb_window_open {
        return;
    }

    let colors = use_context::<ThemeColors>()
        .map(|c| *c)
        .unwrap_or_else(|| state.colors());

    let win_w = 268.0;
    let win_h = 265.0;
    let win_radius = 14.0;

    // Initialize position on first open
    if state.orb_window_pos.0 <= 1.0 && state.orb_window_pos.1 <= 1.0 {
        state.orb_window_pos = ((ui.width - win_w - 24.0).max(20.0), 64.0);
    }

    let mut pos_x = state.orb_window_pos.0.round();
    let mut pos_y = state.orb_window_pos.1.round();

    const HEADER_H: f32 = 32.0;

    // ── Dragging Interaction (handled directly on window coordinates) ──
    let in_header_drag_area = ui.mouse_in_rect(pos_x, pos_y, win_w - 36.0, HEADER_H);
    if in_header_drag_area && !state.orb_window_dragging {
        ui.cursor_icon = CursorIcon::Grab;
        if ui.mouse_down {
            state.orb_window_dragging = true;
            state.orb_window_drag_offset = (ui.mouse_x - pos_x, ui.mouse_y - pos_y);
        }
    }

    if state.orb_window_dragging {
        if ui.mouse_down {
            ui.cursor_icon = CursorIcon::Grabbing;
            ui.consume_pointer();
            pos_x = (ui.mouse_x - state.orb_window_drag_offset.0).clamp(10.0, (ui.width - win_w - 10.0).max(10.0)).round();
            pos_y = (ui.mouse_y - state.orb_window_drag_offset.1).clamp(10.0, (ui.height - win_h - 10.0).max(10.0)).round();
            state.orb_window_pos = (pos_x, pos_y);
            ui.needs_redraw = true;
        } else {
            state.orb_window_dragging = false;
        }
    }

    // Card background: frosted glass with high opacity for sharp contrast
    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.92)
    } else {
        colors.bg_sidebar
    };

    let card_id = Id::from_u64(999999880);
    let prev_win_id = ui.current_window_id;
    ui.current_window_id = Some(card_id);
    ui.record_layout(card_id, zenthra_core::Rect::new(pos_x, pos_y, win_w, win_h));
    let overlay_key = Id::from_u64((card_id.raw() << 8) | 99);
    ui.interaction_state.insert(overlay_key, 1.0);
    ui.active_overlays.push(card_id);
    ui.active_overlay_stack.push(card_id);

    let mut card = ui.container()
        .id(card_id)
        .absolute(pos_x, pos_y)
        .overlay()
        .width(win_w)
        .height(win_h)
        .bg(card_bg)
        .border(colors.border.with_alpha(0.75), 1.0)
        .radius_all(win_radius)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.45), 0.0, 10.0, 30.0);

    if state.glassmorphism {
        card = card.backdrop_blur(state.blur_strength.max(6.0));
    }

    let mut do_close = false;
    let mut do_open_ai_panel = false;

    // Get current mode
    let current_mode = match state.orb_mode_index % 4 {
        1 => OrbMode::Thinking,
        2 => OrbMode::Searching,
        3 => OrbMode::Analyzing,
        _ => OrbMode::Default,
    };

    card.column()
        .show(|ui| {
            // ── Top Header / Drag Handle ──────────────────────────────────────
            let header_bg = colors.highlight.with_alpha(0.04);

            ui.container()
                .id(Id::from_u64(999999881))
                .full_width()
                .height(HEADER_H)
                .radius_top(win_radius)
                .bg(header_bg)
                .padding_left(12.0)
                .padding_right(10.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    // Left: Pulsing status indicator dot + Title
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(7.0)
                        .show(|ui| {
                            let is_working = state.is_ai_working();
                            let dot_col = if is_working {
                                Color::rgb(250.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0)
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
                                .size(12.0)
                                .weight(FontWeight::SemiBold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Right: Close button
                    let close_btn = ui.button("✕")
                        .id(Id::from_u64(999999882))
                        .width(20.0)
                        .height(20.0)
                        .padding_all(0.0)
                        .radius_all(4.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(colors.highlight)
                        .text_color(colors.text_muted)
                        .size(11.0)
                        .show();

                    if close_btn.hovered {
                        ui.cursor_icon = CursorIcon::Pointer;
                    }
                    if close_btn.clicked {
                        do_close = true;
                    }
                });

            // Thin separator
            ui.container()
                .full_width()
                .height(1.0)
                .bg(colors.border.with_alpha(0.35))
                .show(|_| {});

            // ── Orb Animation Stage (Spherical Polyhedra GPU Shader) ─────────
            const STAGE_H: f32 = 125.0;

            let orb_stage_y = pos_y + HEADER_H + 1.0;
            let in_orb_stage = ui.mouse_in_rect(pos_x, orb_stage_y, win_w, STAGE_H);
            if in_orb_stage {
                ui.cursor_icon = CursorIcon::Pointer;
                if ui.clicked {
                    do_open_ai_panel = true;
                    ui.consume_click();
                }
            }

            let is_working = state.is_ai_working();

            // Modes: 0=Default (Pure Sphere), 1=Thinking, 2=Searching, 3=Analyzing
            let (sel_mode, tint) = match current_mode {
                OrbMode::Default => (0.0, [1.0, 1.0, 1.0]),
                OrbMode::Thinking => (1.0, [1.0, 0.78, 0.28]),
                OrbMode::Searching => (2.0, [0.15, 0.92, 1.0]),
                OrbMode::Analyzing => (3.0, [0.75, 0.38, 1.0]),
            };

            // Dynamic thinking/searching/analyzing animation intensity & speed multipliers
            let is_active_mode = is_working || current_mode != OrbMode::Default;
            let active_intensity = if is_active_mode { 1.0 } else { 0.0 };

            let speed_mult = match current_mode {
                OrbMode::Thinking => 1.5,
                OrbMode::Searching => 1.3,
                OrbMode::Analyzing => 1.1,
                OrbMode::Default => 1.0,
            };

            // Compute interactive mouse tilt when hovering over orb stage
            let stage_screen_cx = pos_x + win_w / 2.0;
            let stage_screen_cy = orb_stage_y + STAGE_H / 2.0;
            let (mouse_x_norm, mouse_y_norm) = if in_orb_stage {
                (
                    ((ui.mouse_x - stage_screen_cx) / (win_w * 0.5)).clamp(-1.0, 1.0),
                    ((ui.mouse_y - stage_screen_cy) / (STAGE_H * 0.5)).clamp(-1.0, 1.0),
                )
            } else {
                (0.0, 0.0)
            };

            let shader_params = [
                sel_mode,
                tint[0],
                tint[1],
                tint[2],
                active_intensity,
                speed_mult,
                mouse_x_norm,
                mouse_y_norm,
            ];

            // Custom spherical polyhedra raymarching container
            ui.container()
                .id(Id::from_u64(999999883))
                .full_width()
                .height(STAGE_H)
                .post_process_shader_with_params("spherical_polyhedra", shader_params)
                .show(|_| {});

            // ── Mode Switcher Selector Pills ─────────────────────────────────
            ui.container()
                .full_width()
                .padding_left(8.0)
                .padding_right(8.0)
                .padding_bottom(6.0)
                .row()
                .halign(Align::Center)
                .valign(Align::Center)
                .gap(4.0)
                .show(|ui| {
                    for (idx, mode) in OrbMode::ALL.iter().enumerate() {
                        let is_active = state.orb_mode_index % OrbMode::ALL.len() == idx;
                        let pill_bg = if is_active {
                            colors.accent.with_alpha(0.25)
                        } else {
                            colors.highlight.with_alpha(0.06)
                        };
                        let pill_text_col = if is_active {
                            colors.accent
                        } else {
                            colors.text_muted
                        };
                        let pill_border = if is_active {
                            colors.accent.with_alpha(0.70)
                        } else {
                            colors.border.with_alpha(0.30)
                        };

                        let pill_btn = ui.button(mode.label())
                            .id(Id::from_u64(999999910 + idx as u64))
                            .height(24.0)
                            .padding_x(7.5)
                            .padding_y(3.0)
                            .radius_all(6.0)
                            .bg(pill_bg)
                            .hover_bg(if is_active { pill_bg } else { colors.highlight.with_alpha(0.14) })
                            .text_color(pill_text_col)
                            .border(pill_border, 1.0)
                            .size(10.0)
                            .show();

                        if pill_btn.hovered {
                            ui.cursor_icon = CursorIcon::Pointer;
                        }
                        if pill_btn.clicked {
                            state.orb_mode_index = idx;
                            ui.needs_redraw = true;
                        }
                    }
                });

            // ── Status Readout & Clickable Action ────────────────────────────
            ui.container()
                .full_width()
                .halign(Align::Center)
                .valign(Align::Center)
                .column()
                .gap(4.0)
                .padding_bottom(8.0)
                .show(|ui| {
                    let label = match current_mode {
                        OrbMode::Default => "Default · Ready",
                        OrbMode::Thinking => "Thinking...",
                        OrbMode::Searching => "Searching...",
                        OrbMode::Analyzing => "Analyzing neural web...",
                    };
                    let bullet = if is_active_mode { "● " } else { "● " };
                    let status_label = format!("{}{}", bullet, label);

                    let status_btn = ui.button(&status_label)
                        .id(Id::from_u64(999999890))
                        .height(24.0)
                        .padding_x(12.0)
                        .padding_y(3.0)
                        .radius_all(12.0)
                        .bg(colors.highlight)
                        .hover_bg(colors.highlight.with_alpha(0.20))
                        .text_color(if is_working {
                            Color::rgb(250.0 / 255.0, 160.0 / 255.0, 20.0 / 255.0)
                        } else {
                            colors.text_primary
                        })
                        .border(colors.border.with_alpha(0.50), 1.0)
                        .size(10.5)
                        .show();

                    if status_btn.hovered {
                        ui.cursor_icon = CursorIcon::Pointer;
                    }
                    if status_btn.clicked {
                        do_open_ai_panel = true;
                    }

                    // Secondary helper text
                    ui.text("Click orb to open AI chat")
                        .size(9.5)
                        .color(colors.text_muted.with_alpha(0.80))
                        .show();
                });
        });

    if ui.clicked && ui.mouse_in_rect(pos_x, pos_y, win_w, win_h) {
        ui.consume_click();
    }

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

    // Restore window ID & overlays
    ui.active_overlay_stack.retain(|&id| id != card_id);
    ui.current_window_id = prev_win_id;

    // Hardware V-Sync synchronized animation loop
    ui.request_redraw();
}

/// Renders the orb inside the fixed-width side panel next to the AI chat
#[allow(dead_code)]
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

    let current_mode = match state.orb_mode_index % 4 {
        1 => OrbMode::Thinking,
        2 => OrbMode::Searching,
        3 => OrbMode::Analyzing,
        _ => OrbMode::Default,
    };

    panel.column()
        .halign(Align::Center)
        .valign(Align::Center)
        .gap(8.0)
        .show(|ui| {
            let is_working = state.is_ai_working();
            let stage_size = 54.0;
            let (sel_mode, tint) = match current_mode {
                OrbMode::Default => (0.0, [1.0, 1.0, 1.0]),
                OrbMode::Thinking => (1.0, [1.0, 0.78, 0.28]),
                OrbMode::Searching => (2.0, [0.15, 0.92, 1.0]),
                OrbMode::Analyzing => (3.0, [0.75, 0.38, 1.0]),
            };

            let is_active_mode = is_working || current_mode != OrbMode::Default;
            let active_intensity = if is_active_mode { 1.0 } else { 0.0 };
            let speed_mult = match current_mode {
                OrbMode::Thinking => 1.5,
                OrbMode::Searching => 1.3,
                OrbMode::Analyzing => 1.1,
                OrbMode::Default => 1.0,
            };
            let shader_params = [
                sel_mode, tint[0], tint[1], tint[2],
                active_intensity, speed_mult, 0.0, 0.0,
            ];

            ui.container()
                .id(Id::from_u64(999999871))
                .width(stage_size)
                .height(stage_size)
                .post_process_shader_with_params("spherical_polyhedra", shader_params)
                .show(|_| {});

            // Status label
            let label = current_mode.label();
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
                        .width(4.5)
                        .height(4.5)
                        .radius_all(2.25)
                        .bg(label_color)
                        .show(|_| {});
                    ui.text(label)
                        .size(8.5)
                        .weight(FontWeight::Medium)
                        .color(colors.text_muted)
                        .show();
                });
        });

    ui.request_redraw_after(std::time::Duration::from_millis(16));
}
