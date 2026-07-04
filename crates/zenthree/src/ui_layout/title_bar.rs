use crate::state::ZenthreeState;
use crate::theme::ThemeMode;
use zenthra::{Color, Ui, FontWeight, WindowAction, Align, Id};

pub fn draw_title_bar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();

    // Reset hover flag for menus in title bar since we do not use menu_bar wrapper
    let hover_flag_key = Id::from_u64(999999902);
    ui.interaction_state.insert(hover_flag_key, 0.0);

    const BTN_W: f32 = 20.0;
    const BAR_H: f32 = 36.0;
    const RIGHT_W: f32 = BTN_W * 3.0 + 36.0; // 96.0

    ui.container()
        .full_width()
        .height(BAR_H)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .row()
        .valign(Align::Center)
        .no_wrap()
        .show(|ui| {
            let left_start = ui.cursor_x;

            // LEFT: compact icon  +  inline menu items
            ui.container()
                .row()
                .gap(12.0)
                .valign(Align::Center)
                .padding_left(12.0)
                .show(|ui| {
                    // App Icon & Name
                    ui.container()
                        .row()
                        .gap(8.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(zenthra::icons::NF_FA_CODE)
                                .size(14.0)
                                .color(colors.accent)
                                .show();
                            ui.text("Zenthree")
                                .size(12.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    // Dropdown menus
                    ui.menu("File").show(|ui| {
                        if ui.menu_item("New File").shortcut("Ctrl+N").show().clicked {
                            state.new_untitled_file();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Open File...").shortcut("Ctrl+O").show().clicked {
                            if let Some(path) = rfd::FileDialog::new().pick_file() {
                                state.open_file(&path);
                            }
                            ui.request_redraw();
                        }
                        if ui.menu_item("Open Folder...").shortcut("Ctrl+Shift+O").show().clicked {
                            if let Some(path) = rfd::FileDialog::new().pick_folder() {
                                state.current_dir = path;
                                state.expanded_dirs.clear();
                                state.rebuild_tree();
                                state.status_message = format!("Opened workspace: {}", state.current_dir.display());
                            }
                            ui.request_redraw();
                        }
                        if ui.menu_item("Save").shortcut("Ctrl+S").show().clicked {
                            state.save_active_tab();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Close Tab").shortcut("Ctrl+W").show().clicked {
                            if let Some(idx) = state.active_tab {
                                state.close_tab(idx);
                            }
                            ui.request_redraw();
                        }
                        ui.spacing(4.0);
                        if ui.menu_item("Exit").show().clicked {
                            ui.window_actions.push(WindowAction::Close);
                        }
                    });

                    ui.menu("Edit").show(|ui| {
                        if ui.menu_item("Undo").shortcut("Ctrl+Z").show().clicked {
                            state.status_message = "Undo requested".to_string();
                        }
                        if ui.menu_item("Redo").shortcut("Ctrl+Y").show().clicked {
                            state.status_message = "Redo requested".to_string();
                        }
                        ui.spacing(4.0);
                        if ui.menu_item("Cut").shortcut("Ctrl+X").show().clicked {
                            state.status_message = "Cut".to_string();
                        }
                        if ui.menu_item("Copy").shortcut("Ctrl+C").show().clicked {
                            state.status_message = "Copy".to_string();
                        }
                        if ui.menu_item("Paste").shortcut("Ctrl+V").show().clicked {
                            state.status_message = "Paste".to_string();
                        }
                    });

                    ui.menu("View").show(|ui| {
                        let sidebar_label = if state.sidebar_visible { "Hide Sidebar" } else { "Show Sidebar" };
                        if ui.menu_item(sidebar_label).shortcut("Ctrl+B").show().clicked {
                            state.sidebar_visible = !state.sidebar_visible;
                            ui.request_redraw();
                        }
                    });

                    ui.menu("Theme").show(|ui| {
                        let dark_mark = if state.theme == ThemeMode::Dark { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Dark Theme", dark_mark)).show().clicked {
                            state.theme = ThemeMode::Dark;
                            ui.request_redraw();
                        }
                        let light_mark = if state.theme == ThemeMode::Light { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Light Theme", light_mark)).show().clicked {
                            state.theme = ThemeMode::Light;
                            ui.request_redraw();
                        }
                    });

                    ui.menu("Help").show(|ui| {
                        if ui.menu_item("About Zenthree").show().clicked {
                            state.about_open = true;
                            ui.request_redraw();
                        }
                    });
                });

            let left_w = ui.cursor_x - left_start;
            let drag_w = (ui.available_width - left_w - RIGHT_W).max(0.0);

            // CENTER: drag zone
            let drag_resp = ui.container()
                .width(drag_w)
                .height(BAR_H)
                .show(|_| {});

            if drag_resp.pressed {
                ui.window_actions.push(WindowAction::Drag);
            }

            // RIGHT: window controls
            ui.container()
                .width(RIGHT_W)
                .height(BAR_H)
                .row()
                .valign(Align::Center)
                .padding_right(12.0)
                .gap(12.0)
                .show(|ui| {
                    // Minimize
                    let min_key = Id::from_u64(999999903);
                    let min_hov = ui.interaction_state.get(&min_key).copied().unwrap_or(0.0) > 0.5;
                    let min_resp = ui.container()
                        .width(BTN_W).height(BTN_W)
                        .radius_all(BTN_W / 2.0)
                        .bg(if min_hov {
                            match state.theme {
                                ThemeMode::Dark => Color::rgba(1.0, 1.0, 1.0, 0.08),
                                ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.06),
                            }
                        } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f068}").size(8.0).color(colors.text_muted).show();
                        });
                    let h = min_resp.hovered;
                    let prev = ui.interaction_state.insert(min_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if min_resp.clicked {
                        ui.window_actions.push(WindowAction::Minimize);
                    }

                    // Maximize
                    let max_key = Id::from_u64(999999904);
                    let max_hov = ui.interaction_state.get(&max_key).copied().unwrap_or(0.0) > 0.5;
                    let max_resp = ui.container()
                        .width(BTN_W).height(BTN_W)
                        .radius_all(BTN_W / 2.0)
                        .bg(if max_hov {
                            match state.theme {
                                ThemeMode::Dark => Color::rgba(1.0, 1.0, 1.0, 0.08),
                                ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.06),
                            }
                        } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f0c8}").size(7.5).color(colors.text_muted).show();
                        });
                    let h = max_resp.hovered;
                    let prev = ui.interaction_state.insert(max_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if max_resp.clicked {
                        ui.window_actions.push(WindowAction::Maximize);
                    }

                    // Close
                    let cls_key = Id::from_u64(999999905);
                    let cls_hov = ui.interaction_state.get(&cls_key).copied().unwrap_or(0.0) > 0.5;
                    let cls_resp = ui.container()
                        .width(BTN_W).height(BTN_W)
                        .radius_all(BTN_W / 2.0)
                        .bg(if cls_hov { Color::rgb(0.85, 0.15, 0.15) } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let fg = if cls_hov { Color::WHITE } else { colors.text_muted };
                            ui.text("\u{f00d}").size(8.0).color(fg).show();
                        });
                    let h = cls_resp.hovered;
                    let prev = ui.interaction_state.insert(cls_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if cls_resp.clicked {
                        ui.window_actions.push(WindowAction::Close);
                    }
                });
        });

    // Click outside (light dismiss) handling for menus
    let active_menu_key = Id::from_u64(999999900);
    let active_submenu_key = Id::from_u64(999999901);
    let hover_flag_key = Id::from_u64(999999902);

    let clicked = ui.clicked;
    let active_menu_id = ui.interaction_state.get(&active_menu_key).copied().map(|v| v as u64).unwrap_or(0);
    let hover_flag = ui.interaction_state.get(&hover_flag_key).copied().unwrap_or(0.0) > 0.5;

    if active_menu_id != 0 && clicked && !hover_flag {
        ui.interaction_state.insert(active_menu_key, 0.0);
        ui.interaction_state.insert(active_submenu_key, 0.0);
        ui.request_redraw();
    }
}
