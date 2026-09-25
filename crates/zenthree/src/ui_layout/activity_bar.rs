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
                    // 1. Files / Explorer (\u{f0645})
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
                                    ui.text("\u{f0645}")
                                        .size(17.0)
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
                            let prev = state.sidebar_view;
                            state.sidebar_view = SidebarView::Explorer;
                            if prev != SidebarView::Explorer {
                                state.rebuild_tree();
                            }
                        }
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 2. Git Panel (\u{ea68})
                    let is_git = state.sidebar_visible && state.sidebar_view == SidebarView::Git;
                    let git_key = Id::from_u64(999997204);
                    let git_hov = ui.interaction_state.get(&git_key).copied().unwrap_or(0.0) > 0.5;
                    let total_changes = state.git_state.staged.len() + state.git_state.unstaged.len();
                    let git_btn_id = Id::from_u64(999997204);

                    let git_btn = ui.container()
                        .raw_id(git_btn_id)
                        .clickable(true)
                        .width(40.0)
                        .height(38.0)
                        .radius_all(4.0)
                        .bg(if is_git {
                            colors.accent.with_alpha(0.16)
                        } else if git_hov {
                            colors.highlight
                        } else {
                            Color::TRANSPARENT
                        })
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.container()
                                .width(2.5)
                                .height(22.0)
                                .radius_all(1.25)
                                .bg(if is_git { colors.accent } else { Color::TRANSPARENT })
                                .show(|_| {});

                            ui.container()
                                .width(37.5)
                                .height(38.0)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let icon_color = if is_git {
                                        colors.accent
                                    } else if git_hov {
                                        colors.text_primary
                                    } else {
                                        colors.text_muted
                                    };
                                    ui.text("\u{ea68}")
                                        .size(16.0)
                                        .color(icon_color)
                                        .show();
                                });
                        });

                    if total_changes > 0 {
                        let btn_rect = ui.screen_layout_cache.get(&git_btn_id)
                            .copied()
                            .or_else(|| ui.get_recorded_layout(git_btn_id).map(|(r, _)| r));

                        if let Some(rect) = btn_rect {
                            let badge_text = if total_changes > 99 { "99+".to_string() } else { format!("{}", total_changes) };
                            let badge_x = rect.origin.x + rect.size.width - 15.0;
                            let badge_y = rect.origin.y + rect.size.height - 14.0;
                            ui.overlay(|ui| {
                                ui.container()
                                    .raw_id(Id::from_u64(999997205))
                                    .absolute(badge_x, badge_y)
                                    .overlay()
                                    .height(13.0)
                                    .padding(0.0, 3.5, 0.0, 3.5)
                                    .radius_all(6.5)
                                    .bg(colors.accent)
                                    .halign(Align::Center)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text(&badge_text)
                                            .size(8.5)
                                            .weight(zenthra::FontWeight::Bold)
                                            .color(Color::WHITE)
                                            .show();
                                    });
                            });
                        }
                    }

                    let h = git_btn.hovered;
                    let prev = ui.interaction_state.insert(git_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                    if git_btn.clicked {
                        if state.sidebar_visible && state.sidebar_view == SidebarView::Git {
                            state.sidebar_visible = false;
                        } else {
                            state.sidebar_visible = true;
                            state.sidebar_view = SidebarView::Git;
                            state.refresh_git_status();
                        }
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 3. Notes Panel (\u{f1782})
                    let is_notes = state.sidebar_visible && state.sidebar_view == SidebarView::Notes;
                    let notes_key = Id::from_u64(999997203);
                    let notes_hov = ui.interaction_state.get(&notes_key).copied().unwrap_or(0.0) > 0.5;

                    let notes_btn = ui.container()
                        .clickable(true)
                        .width(40.0)
                        .height(38.0)
                        .radius_all(4.0)
                        .bg(if is_notes {
                            colors.accent.with_alpha(0.16)
                        } else if notes_hov {
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
                                .bg(if is_notes { colors.accent } else { Color::TRANSPARENT })
                                .show(|_| {});

                            // Centered icon container
                            ui.container()
                                .width(37.5)
                                .height(38.0)
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    let icon_color = if is_notes {
                                        colors.accent
                                    } else if notes_hov {
                                        colors.text_primary
                                    } else {
                                        colors.text_muted
                                    };
                                    ui.text("\u{f1782}")
                                        .size(16.0)
                                        .color(icon_color)
                                        .show();
                                });
                        });

                    let h = notes_btn.hovered;
                    let prev = ui.interaction_state.insert(notes_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                    if notes_btn.clicked {
                        if state.sidebar_visible && state.sidebar_view == SidebarView::Notes {
                            state.sidebar_visible = false;
                        } else {
                            state.sidebar_visible = true;
                            let prev = state.sidebar_view;
                            state.sidebar_view = SidebarView::Notes;
                            if prev != SidebarView::Notes {
                                state.rebuild_tree();
                            }
                        }
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 4. Extensions / Language Tools (\u{f0431})
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
                                    ui.text("\u{f0431}")
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

            // Bottom Section: Settings / About (\u{f013})
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
                            ui.text("\u{f013}")
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
