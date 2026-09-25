use crate::state::{ZenthreeState, SidebarView};
use zenthra::{Color, Ui, Align, Id, use_context};
use crate::theme::ThemeColors;

pub const ACTIVITY_BAR_WIDTH: f32 = 44.0;

pub fn draw_activity_bar(ui: &mut Ui, state: &mut ZenthreeState, height: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    let bar_x = ui.cursor_x;
    let bar_y = ui.cursor_y;

    ui.container()
        .width(ACTIVITY_BAR_WIDTH)
        .height(height)
        .bg(colors.bg_base)
        .column()
        .halign(Align::Center)
        .valign(Align::SpaceBetween)
        .padding(6.0, 0.0, 8.0, 0.0)
        .show(|ui| {
            // Top Section: Primary navigation icons
            ui.container()
                .column()
                .halign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    // 1. Files / Explorer
                    let is_explorer = state.sidebar_visible && state.sidebar_view == SidebarView::Explorer;
                    let exp_key = Id::from_u64(999997200);
                    let exp_hov = ui.interaction_state.get(&exp_key).copied().unwrap_or(0.0) > 0.5;

                    let exp_btn = ui.container()
                        .clickable(true)
                        .width(40.0)
                        .height(38.0)
                        .radius_all(4.0)
                        .bg(if is_explorer {
                            colors.accent.with_alpha(0.16)
                        } else if exp_hov {
                            colors.highlight
                        } else {
                            Color::TRANSPARENT
                        })
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            // Left active indicator strip (2.5px wide)
                            ui.container()
                                .width(2.5)
                                .height(22.0)
                                .radius_all(1.25)
                                .bg(if is_explorer { colors.accent } else { Color::TRANSPARENT })
                                .show(|_| {});

                            // Centered icon container
                            ui.container()
                                .width(37.5)
                                .height(38.0)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let icon_color = if is_explorer {
                                        colors.accent
                                    } else if exp_hov {
                                        colors.text_primary
                                    } else {
                                        colors.text_muted
                                    };
                                    // NF_FA_FILES_O (\u{f0c5}) - iconic files/explorer icon
                                    ui.text("\u{f0c5}")
                                        .size(16.0)
                                        .color(icon_color)
                                        .show();
                                });
                        });

                    let h = exp_btn.hovered;
                    let prev = ui.interaction_state.insert(exp_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                    if exp_btn.clicked {
                        if state.sidebar_visible && state.sidebar_view == SidebarView::Explorer {
                            state.sidebar_visible = false;
                        } else {
                            state.sidebar_visible = true;
                            state.sidebar_view = SidebarView::Explorer;
                        }
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 2. Extensions / Language Tools
                    let is_ext = state.sidebar_visible && state.sidebar_view == SidebarView::Extensions;
                    let ext_key = Id::from_u64(999997201);
                    let ext_hov = ui.interaction_state.get(&ext_key).copied().unwrap_or(0.0) > 0.5;

                    let ext_btn = ui.container()
                        .clickable(true)
                        .width(40.0)
                        .height(38.0)
                        .radius_all(4.0)
                        .bg(if is_ext {
                            colors.accent.with_alpha(0.16)
                        } else if ext_hov {
                            colors.highlight
                        } else {
                            Color::TRANSPARENT
                        })
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            // Left active indicator strip (2.5px wide)
                            ui.container()
                                .width(2.5)
                                .height(22.0)
                                .radius_all(1.25)
                                .bg(if is_ext { colors.accent } else { Color::TRANSPARENT })
                                .show(|_| {});

                            // Centered icon container
                            ui.container()
                                .width(37.5)
                                .height(38.0)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let icon_color = if is_ext {
                                        colors.accent
                                    } else if ext_hov {
                                        colors.text_primary
                                    } else {
                                        colors.text_muted
                                    };
                                    // NF_FA_PUZZLE_PIECE (\u{f12e}) - iconic puzzle piece extension icon
                                    ui.text("\u{f12e}")
                                        .size(16.0)
                                        .color(icon_color)
                                        .show();
                                });
                        });

                    let h = ext_btn.hovered;
                    let prev = ui.interaction_state.insert(ext_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                    if ext_btn.clicked {
                        if state.sidebar_visible && state.sidebar_view == SidebarView::Extensions {
                            state.sidebar_visible = false;
                        } else {
                            state.sidebar_visible = true;
                            state.sidebar_view = SidebarView::Extensions;
                        }
                        state.save_persistent_state();
                        ui.request_redraw();
                    }
                });

            // Bottom Section: Settings / About
            ui.container()
                .column()
                .halign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    let set_key = Id::from_u64(999997202);
                    let set_hov = ui.interaction_state.get(&set_key).copied().unwrap_or(0.0) > 0.5;

                    let set_btn = ui.container()
                        .clickable(true)
                        .width(40.0)
                        .height(38.0)
                        .radius_all(4.0)
                        .bg(if state.about_open {
                            colors.accent.with_alpha(0.16)
                        } else if set_hov {
                            colors.highlight
                        } else {
                            Color::TRANSPARENT
                        })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let icon_color = if state.about_open {
                                colors.accent
                            } else if set_hov {
                                colors.text_primary
                            } else {
                                colors.text_muted
                            };
                            // NF_FA_SETTINGS (\u{f013}) gear icon
                            ui.text(zenthra::icons::NF_FA_SETTINGS)
                                .size(16.0)
                                .color(icon_color)
                                .show();
                        });

                    let h = set_btn.hovered;
                    let prev = ui.interaction_state.insert(set_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                    if set_btn.clicked {
                        state.about_open = !state.about_open;
                        ui.request_redraw();
                    }
                });
        });

    // Right 1px divider line separating activity bar from sidebar or workspace
    ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [bar_x + ACTIVITY_BAR_WIDTH - 1.0, bar_y],
            size: [1.0, height],
            color: colors.border.to_array(),
            radius: [0.0; 4],
            border_width: 0.0,
            border_color: [0.0; 4],
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));
}
