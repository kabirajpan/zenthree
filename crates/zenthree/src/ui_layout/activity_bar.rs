use crate::state::{ZenthreeState, SidebarView};
use zenthra::{Color, Ui, Align, Id};

pub const ACTIVITY_BAR_WIDTH: f32 = 44.0;

pub fn draw_activity_bar(ui: &mut Ui, state: &mut ZenthreeState, height: f32) {
    let colors = zenthra::use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    ui.container()
        .width(ACTIVITY_BAR_WIDTH)
        .height(height)
        .bg(colors.bg_base)
        .border(colors.border, 1.0)
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
                            state.set_sidebar_visible(false);
                        } else {
                            state.set_sidebar_view(SidebarView::Explorer);
                            state.set_sidebar_visible(true);
                        }
                        ui.request_redraw();
                    }

                    // 2. Git / Source Control
                    let is_git = state.sidebar_visible && state.sidebar_view == SidebarView::SourceControl;
                    let git_key = Id::from_u64(999997203);
                    let git_hov = ui.interaction_state.get(&git_key).copied().unwrap_or(0.0) > 0.5;
                    let total_changes = state.git_state.staged.len() + state.git_state.unstaged.len();

                    let git_btn_id = Id::from_u64(999997203);

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
                            // Left active indicator strip (2.5px wide)
                            ui.container()
                                .width(2.5)
                                .height(22.0)
                                .radius_all(1.25)
                                .bg(if is_git { colors.accent } else { Color::TRANSPARENT })
                                .show(|_| {});

                            // Centered icon container - perfectly centered, never shifts
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
                                    // Codicon source control (\u{ea68})
                                    ui.text("\u{ea68}")
                                        .size(16.0)
                                        .color(icon_color)
                                        .show();
                                });
                        });

                    // Badge count for changed files pinned on bottom-right of tab icon (just like VS Code)
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
                                    .raw_id(Id::from_u64(999997204))
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
                        if state.sidebar_visible && state.sidebar_view == SidebarView::SourceControl {
                            state.set_sidebar_visible(false);
                        } else {
                            state.set_sidebar_view(SidebarView::SourceControl);
                            state.set_sidebar_visible(true);
                            state.refresh_git_status();
                        }
                        ui.request_redraw();
                    }

                    // 3. Extensions / Language Tools
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
                            state.set_sidebar_visible(false);
                        } else {
                            state.set_sidebar_view(SidebarView::Extensions);
                            state.set_sidebar_visible(true);
                        }
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
}
