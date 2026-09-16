use crate::state::ZenthreeState;
use crate::theme::{ThemeMode, ACCENT_COLOR_OPTIONS, ICON_THEME_OPTIONS};
use zenthra::{Color, Ui, FontWeight, Align, Id};

fn get_animated_app_title() -> (&'static str, std::time::Duration, bool) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let millis = (now.as_millis() % 12640) as u64;

    if millis < 5000 {
        // Hold ZENTHREE for 5 seconds
        ("ZENTHREE", std::time::Duration::from_millis(5000 - millis), false)
    } else if millis < 5240 {
        ("ZENTHRE", std::time::Duration::from_millis(5240 - millis), true)
    } else if millis < 5480 {
        ("ZENTHE", std::time::Duration::from_millis(5480 - millis), true)
    } else if millis < 5720 {
        ("ZENTE", std::time::Duration::from_millis(5720 - millis), true)
    } else if millis < 5960 {
        // Arrived at ZENE with normal E
        ("ZENE", std::time::Duration::from_millis(5960 - millis), true)
    } else if millis < 6200 {
        // Flip the E -> ZENƎ
        ("ZEN\u{018e}", std::time::Duration::from_millis(6200 - millis), true)
    } else if millis < 11200 {
        // Hold ZENƎ for 5 seconds
        ("ZEN\u{018e}", std::time::Duration::from_millis(11200 - millis), false)
    } else if millis < 11440 {
        // Flip back to normal E -> ZENE
        ("ZENE", std::time::Duration::from_millis(11440 - millis), true)
    } else if millis < 11680 {
        ("ZENTE", std::time::Duration::from_millis(11680 - millis), true)
    } else if millis < 11920 {
        ("ZENTHE", std::time::Duration::from_millis(11920 - millis), true)
    } else if millis < 12160 {
        ("ZENTHRE", std::time::Duration::from_millis(12160 - millis), true)
    } else if millis < 12400 {
        ("ZENTHREE", std::time::Duration::from_millis(12400 - millis), true)
    } else {
        ("ZENTHREE", std::time::Duration::from_millis(12640 - millis), true)
    }
}

pub fn draw_title_bar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();

    // Reset hover flag for menus in title bar since we do not use menu_bar wrapper
    let hover_flag_key = Id::from_u64(999999902);
    ui.interaction_state.insert(hover_flag_key, 0.0);

    const BTN_W: f32 = 20.0;
    const BAR_H: f32 = 36.0;
    const RIGHT_W: f32 = 226.0;

    ui.container()
        .full_width()
        .height(BAR_H)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween) // Use SpaceBetween: Left menus vs Right controls
        .draggable_window(true)
        .no_wrap()
        .show(|ui| {
            // LEFT: compact icon  +  inline menu items
            ui.container()
                .row()
                .gap(12.0)
                .valign(Align::Center)
                .padding_left(12.0)
                .show(|ui| {
                    // App Icon & Animated Name (ZENTHREE <-> ZENƎ with flipped E)
                    let (app_title, next_redraw, is_animating) = get_animated_app_title();
                    ui.request_redraw_after(next_redraw);
                    if is_animating {
                        ui.needs_redraw = true;
                    }

                    ui.container()
                        .row()
                        .gap(8.0)
                        .min_width(96.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(zenthra::icons::NF_FA_CODE)
                                .size(14.0)
                                .color(colors.accent)
                                .show();

                            let mut title_builder = ui.text(app_title)
                                .family("MODERNIZ")
                                .size(12.0)
                                .bold()
                                .color(colors.accent);

                            if is_animating {
                                title_builder = title_builder.continuous();
                            }
                            title_builder.show();
                        });

                    // Dropdown menus with cascading submenus
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
                                state.open_workspace(path);
                            }
                            ui.request_redraw();
                        }
                        if state.has_workspace {
                            if ui.menu_item("Close Folder").show().clicked {
                                state.close_workspace();
                                ui.request_redraw();
                            }
                        }

                        // Cascading Submenu: Open Recent
                        ui.sub_menu("Open Recent").show(|ui| {
                            let recent = state.recent_files.clone();
                            if recent.is_empty() {
                                ui.menu_item("No Recent Files").show();
                            } else {
                                for path in recent {
                                    let label = path.file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("Unknown");
                                    if ui.menu_item(label).show().clicked {
                                        state.open_file(&path);
                                        ui.request_redraw();
                                    }
                                }
                                ui.spacing(4.0);
                                if ui.menu_item("Clear History").show().clicked {
                                    state.recent_files.clear();
                                    ui.request_redraw();
                                }
                            }
                        });

                        ui.spacing(4.0);

                        if ui.menu_item("Save").shortcut("Ctrl+S").show().clicked {
                            state.save_active_tab();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Save As...").shortcut("Ctrl+Shift+S").show().clicked {
                            state.save_active_tab_as();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Close Tab").shortcut("Ctrl+W").show().clicked {
                            if let Some(idx) = state.active_tab {
                                state.close_tab(idx);
                            }
                            ui.request_redraw();
                        }

                        ui.spacing(4.0);

                        // Cascading Submenu: Line Endings
                        ui.sub_menu("Line Endings").show(|ui| {
                            if ui.menu_item("● LF (Unix)").show().clicked {
                                state.status_message = "Line endings set to LF".to_string();
                                ui.request_redraw();
                            }
                            if ui.menu_item("  CRLF (Windows)").show().clicked {
                                state.status_message = "Line endings set to CRLF".to_string();
                                ui.request_redraw();
                            }
                        });

                        ui.spacing(4.0);
                        if ui.menu_item("Exit").show().clicked {
                            state.save_persistent_state();
                            ui.close();
                        }
                    });

                    ui.menu("Edit").show(|ui| {
                        if ui.menu_item("Undo").shortcut("Ctrl+Z").show().clicked {
                            state.undo_active_tab();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Redo").shortcut("Ctrl+Y").show().clicked {
                            state.redo_active_tab();
                            ui.request_redraw();
                        }
                        ui.spacing(4.0);
                        if ui.menu_item("Cut").shortcut("Ctrl+X").show().clicked {
                            state.cut_selection();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Copy").shortcut("Ctrl+C").show().clicked {
                            state.copy_selection();
                            ui.request_redraw();
                        }
                        if ui.menu_item("Paste").shortcut("Ctrl+V").show().clicked {
                            state.paste_clipboard();
                            ui.request_redraw();
                        }
                        ui.spacing(4.0);

                        // Cascading Submenu: Text Transform
                        ui.sub_menu("Transform").show(|ui| {
                            if ui.menu_item("UPPERCASE").show().clicked {
                                state.status_message = "Transformed to UPPERCASE".to_string();
                                ui.request_redraw();
                            }
                            if ui.menu_item("lowercase").show().clicked {
                                state.status_message = "Transformed to lowercase".to_string();
                                ui.request_redraw();
                            }
                            if ui.menu_item("Title Case").show().clicked {
                                state.status_message = "Transformed to Title Case".to_string();
                                ui.request_redraw();
                            }
                        });

                        // Cascading Submenu: Selection
                        ui.sub_menu("Selection").show(|ui| {
                            if ui.menu_item("Select All").shortcut("Ctrl+A").show().clicked {
                                if let Some(idx) = state.active_tab {
                                    if idx < state.tabs.len() {
                                        state.tabs[idx].editor.select_all();
                                    }
                                }
                                state.status_message = "Selected all".to_string();
                                ui.request_redraw();
                            }
                            if ui.menu_item("Expand Selection").show().clicked {
                                state.status_message = "Expand Selection".to_string();
                                ui.request_redraw();
                            }
                            if ui.menu_item("Shrink Selection").show().clicked {
                                state.status_message = "Shrink Selection".to_string();
                                ui.request_redraw();
                            }
                        });

                        ui.spacing(4.0);
                        if ui.menu_item("Format Document").shortcut("Ctrl+Shift+F").show().clicked {
                            state.format_active_tab();
                            ui.request_redraw();
                        }
                    });

                    ui.menu("View").show(|ui| {
                        let ext_mark = if state.sidebar_visible && state.sidebar_view == crate::state::SidebarView::Extensions { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Extensions / Language Packs", ext_mark)).shortcut("Ctrl+Shift+X").show().clicked {
                            if state.sidebar_visible && state.sidebar_view == crate::state::SidebarView::Extensions {
                                state.sidebar_view = crate::state::SidebarView::Explorer;
                            } else {
                                state.sidebar_visible = true;
                                state.sidebar_view = crate::state::SidebarView::Extensions;
                            }
                            state.save_persistent_state();
                            ui.request_redraw();
                        }

                        ui.spacing(4.0);

                        // Cascading Submenu: Appearance
                        ui.sub_menu("Appearance").show(|ui| {
                            let sb_mark = if state.sidebar_visible { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Sidebar", sb_mark)).shortcut("Ctrl+B").show().clicked {
                                state.sidebar_visible = !state.sidebar_visible;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let st_mark = if state.status_bar_visible { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Status Bar", st_mark)).show().clicked {
                                state.status_bar_visible = !state.status_bar_visible;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let ln_mark = if state.line_numbers_visible { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Line Numbers", ln_mark)).show().clicked {
                                state.line_numbers_visible = !state.line_numbers_visible;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                        });

                        // Cascading Submenu: File Explorer
                        ui.sub_menu("File Explorer").show(|ui| {
                            // Presets
                            let p_modern = if state.explorer_preset == crate::state::ExplorerPreset::ModernMinimalist { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Minimal", p_modern)).show().clicked {
                                state.explorer_preset = crate::state::ExplorerPreset::ModernMinimalist;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let p_snacks = if state.explorer_preset == crate::state::ExplorerPreset::NeovimSnacks { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Framed Box", p_snacks)).show().clicked {
                                state.explorer_preset = crate::state::ExplorerPreset::NeovimSnacks;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let p_classic = if state.explorer_preset == crate::state::ExplorerPreset::ClassicTree { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Indented Tree", p_classic)).show().clicked {
                                state.explorer_preset = crate::state::ExplorerPreset::ClassicTree;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }

                            ui.spacing(4.0);

                            let guide_mark = if state.explorer_tree_guides { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Indent Guides", guide_mark)).show().clicked {
                                state.explorer_tree_guides = !state.explorer_tree_guides;
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let git_mark = if state.explorer_git_status { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Git Badges", git_mark)).show().clicked {
                                state.explorer_git_status = !state.explorer_git_status;
                                state.rebuild_tree();
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let hidden_mark = if state.explorer_show_hidden { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Hidden Files", hidden_mark)).show().clicked {
                                state.explorer_show_hidden = !state.explorer_show_hidden;
                                state.rebuild_tree();
                                state.save_persistent_state();
                                ui.request_redraw();
                            }

                            ui.spacing(4.0);

                            if ui.menu_item("Collapse All Folders").show().clicked {
                                state.collapse_all_folders();
                                ui.request_redraw();
                            }
                            if ui.menu_item("Expand All Folders").show().clicked {
                                state.expand_all_folders();
                                ui.request_redraw();
                            }
                            if ui.menu_item("Refresh Explorer").shortcut("F5").show().clicked {
                                state.rebuild_tree();
                                ui.request_redraw();
                            }
                        });

                        // Cascading Submenu: Zoom
                        ui.sub_menu("Zoom").show(|ui| {
                            let z = state.zoom_level;
                            let z100 = if (z - 1.0).abs() < 0.01 { "● " } else { "  " };
                            let z125 = if (z - 1.25).abs() < 0.01 { "● " } else { "  " };
                            let z150 = if (z - 1.5).abs() < 0.01 { "● " } else { "  " };

                            if ui.menu_item("Zoom In").shortcut("Ctrl +").show().clicked {
                                state.zoom_level = (state.zoom_level + 0.1).min(2.0);
                                ui.request_redraw();
                            }
                            if ui.menu_item("Zoom Out").shortcut("Ctrl -").show().clicked {
                                state.zoom_level = (state.zoom_level - 0.1).max(0.6);
                                ui.request_redraw();
                            }
                            ui.spacing(4.0);
                            if ui.menu_item(&format!("{}100% (Reset)", z100)).show().clicked {
                                state.zoom_level = 1.0;
                                ui.request_redraw();
                            }
                            if ui.menu_item(&format!("{}125%", z125)).show().clicked {
                                state.zoom_level = 1.25;
                                ui.request_redraw();
                            }
                            if ui.menu_item(&format!("{}150%", z150)).show().clicked {
                                state.zoom_level = 1.5;
                                ui.request_redraw();
                            }
                        });
                    });

                    ui.menu("Theme").show(|ui| {
                        // Cascading Submenu: Color Mode
                        ui.sub_menu("Color Mode").show(|ui| {
                            let dark_mark = if state.theme == ThemeMode::Dark { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Dark Theme", dark_mark)).show().clicked {
                                state.theme = ThemeMode::Dark;
                                state.highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::dark());
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let light_mark = if state.theme == ThemeMode::Light { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Light Theme", light_mark)).show().clicked {
                                state.theme = ThemeMode::Light;
                                state.highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::light());
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                        });

                        // Cascading Submenu: Accent Color
                        ui.sub_menu("Accent Color").show(|ui| {
                            for (color_id, color_name) in ACCENT_COLOR_OPTIONS {
                                let is_active = state.accent_color == *color_id;
                                let label = if is_active {
                                    format!("● {}", color_name)
                                } else {
                                    format!("  {}", color_name)
                                };
                                if ui.menu_item(&label).show().clicked {
                                    state.accent_color = color_id.to_string();
                                    state.save_persistent_state();
                                    ui.request_redraw();
                                }
                            }
                        });

                        // Cascading Submenu: File Icon Theme
                        ui.sub_menu("File Icon Theme").show(|ui| {
                            for (theme_id, theme_name) in ICON_THEME_OPTIONS {
                                let is_active = state.icon_theme == *theme_id;
                                let label = if is_active {
                                    format!("● {}", theme_name)
                                } else {
                                    format!("  {}", theme_name)
                                };
                                if ui.menu_item(&label).show().clicked {
                                    state.icon_theme = theme_id.to_string();
                                    state.status_message = format!("Icon theme: {}", theme_name);
                                    state.save_persistent_state();
                                    ui.request_redraw();
                                }
                            }
                        });

                        // Cascading Submenu: Syntax Theme
                        ui.sub_menu("Syntax Highlighting").show(|ui| {
                            let is_dark = state.theme == ThemeMode::Dark;
                            let dark_s = if is_dark { "● " } else { "  " };
                            let light_s = if !is_dark { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Dark Highlighting", dark_s)).show().clicked {
                                state.highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::dark());
                                ui.request_redraw();
                            }
                            if ui.menu_item(&format!("{}Light Highlighting", light_s)).show().clicked {
                                state.highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::light());
                                ui.request_redraw();
                            }
                        });
                    });

                    ui.menu("Help").show(|ui| {
                        if ui.menu_item("About Zenthree").show().clicked {
                            state.about_open = true;
                            ui.request_redraw();
                        }
                    });
                });

            // RIGHT: window controls & panel toggles
            // Sequence: left, bottom, right, user icon, gap, minimize, floating window, cross
            ui.container()
                .width(RIGHT_W)
                .height(BAR_H)
                .row()
                .valign(Align::Center)
                .padding_right(12.0)
                .gap(6.0)
                .show(|ui| {
                    // 1. Left Panel (File Tree) Toggle
                    let left_key = Id::from_u64(999999910);
                    let left_hov = ui.interaction_state.get(&left_key).copied().unwrap_or(0.0) > 0.5;
                    let left_resp = ui.container()
                        .clickable(true)
                        .width(22.0).height(20.0)
                        .radius_all(3.0)
                        .bg(if left_hov {
                            colors.highlight
                        } else if state.sidebar_visible {
                            colors.accent.with_alpha(0.15)
                        } else {
                            Color::TRANSPARENT
                        })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let border_color = if state.sidebar_visible {
                                colors.accent
                            } else if left_hov {
                                colors.text_primary.with_alpha(0.8)
                            } else {
                                colors.text_muted.with_alpha(0.6)
                            };
                            let fill_color = if state.sidebar_visible {
                                colors.accent
                            } else if left_hov {
                                colors.text_muted.with_alpha(0.6)
                            } else {
                                colors.text_muted.with_alpha(0.3)
                            };
                            ui.container()
                                .width(14.0).height(11.0)
                                .border(border_color, 1.0)
                                .radius_all(2.0)
                                .row()
                                .show(|ui| {
                                    ui.container()
                                        .width(4.0).height(9.0)
                                        .bg(fill_color)
                                        .show(|_| {});
                                });
                        });
                    let h = left_resp.hovered;
                    let prev = ui.interaction_state.insert(left_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if left_resp.clicked {
                        state.sidebar_visible = !state.sidebar_visible;
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 2. Bottom Panel (Terminal) Toggle
                    let btm_key = Id::from_u64(999999911);
                    let btm_hov = ui.interaction_state.get(&btm_key).copied().unwrap_or(0.0) > 0.5;
                    let btm_resp = ui.container()
                        .clickable(true)
                        .width(22.0).height(20.0)
                        .radius_all(3.0)
                        .bg(if btm_hov {
                            colors.highlight
                        } else if state.terminal_visible {
                            colors.accent.with_alpha(0.15)
                        } else {
                            Color::TRANSPARENT
                        })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let border_color = if state.terminal_visible {
                                colors.accent
                            } else if btm_hov {
                                colors.text_primary.with_alpha(0.8)
                            } else {
                                colors.text_muted.with_alpha(0.6)
                            };
                            let fill_color = if state.terminal_visible {
                                colors.accent
                            } else if btm_hov {
                                colors.text_muted.with_alpha(0.6)
                            } else {
                                colors.text_muted.with_alpha(0.3)
                            };
                            ui.container()
                                .width(14.0).height(11.0)
                                .border(border_color, 1.0)
                                .radius_all(2.0)
                                .column()
                                .valign(Align::Bottom)
                                .show(|ui| {
                                    ui.container()
                                        .full_width().height(3.0)
                                        .bg(fill_color)
                                        .show(|_| {});
                                });
                        });
                    let h = btm_resp.hovered;
                    let prev = ui.interaction_state.insert(btm_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if btm_resp.clicked {
                        state.terminal_visible = !state.terminal_visible;
                        if state.terminal_visible {
                            state.ensure_terminal_session();
                        }
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 3. Right Panel (AI Agent) Toggle
                    let right_key = Id::from_u64(999999912);
                    let right_hov = ui.interaction_state.get(&right_key).copied().unwrap_or(0.0) > 0.5;
                    let right_resp = ui.container()
                        .clickable(true)
                        .width(22.0).height(20.0)
                        .radius_all(3.0)
                        .bg(if right_hov {
                            colors.highlight
                        } else if state.ai_panel_visible {
                            colors.accent.with_alpha(0.15)
                        } else {
                            Color::TRANSPARENT
                        })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let border_color = if state.ai_panel_visible {
                                colors.accent
                            } else if right_hov {
                                colors.text_primary.with_alpha(0.8)
                            } else {
                                colors.text_muted.with_alpha(0.6)
                            };
                            let fill_color = if state.ai_panel_visible {
                                colors.accent
                            } else if right_hov {
                                colors.text_muted.with_alpha(0.6)
                            } else {
                                colors.text_muted.with_alpha(0.3)
                            };
                            ui.container()
                                .width(14.0).height(11.0)
                                .border(border_color, 1.0)
                                .radius_all(2.0)
                                .row()
                                .halign(Align::Right)
                                .show(|ui| {
                                    ui.container()
                                        .width(4.0).height(9.0)
                                        .bg(fill_color)
                                        .show(|_| {});
                                });
                        });
                    let h = right_resp.hovered;
                    let prev = ui.interaction_state.insert(right_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if right_resp.clicked {
                        state.ai_panel_visible = !state.ai_panel_visible;
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 4. User Icon
                    let user_key = Id::from_u64(999999913);
                    let user_hov = ui.interaction_state.get(&user_key).copied().unwrap_or(0.0) > 0.5;
                    let user_resp = ui.container()
                        .clickable(true)
                        .width(22.0).height(22.0)
                        .radius_all(11.0)
                        .bg(if user_hov {
                            colors.highlight
                        } else if state.user_profile_open {
                            colors.accent.with_alpha(0.25)
                        } else {
                            Color::TRANSPARENT
                        })
                        .border(if state.user_profile_open { colors.accent } else { colors.border }, 1.0)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(zenthra::icons::NF_FA_USER)
                                .size(9.5)
                                .color(if state.user_profile_open { colors.accent } else { colors.text_primary })
                                .show();
                        });
                    let h = user_resp.hovered;
                    let prev = ui.interaction_state.insert(user_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if user_resp.clicked {
                        state.user_profile_open = !state.user_profile_open;
                        ui.request_redraw();
                    }

                    // 5. Gap / Divider
                    ui.container()
                        .width(10.0)
                        .height(BAR_H)
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.container()
                                .width(1.0)
                                .height(14.0)
                                .bg(colors.border)
                                .show(|_| {});
                        });

                    // 6. Minimize
                    let min_key = Id::from_u64(999999903);
                    let min_hov = ui.interaction_state.get(&min_key).copied().unwrap_or(0.0) > 0.5;
                    let min_resp = ui.container()
                        .clickable(true)
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
                        ui.minimize();
                    }

                    // 7. Floating Window (Maximize/Restore)
                    let max_key = Id::from_u64(999999904);
                    let max_hov = ui.interaction_state.get(&max_key).copied().unwrap_or(0.0) > 0.5;
                    let max_resp = ui.container()
                        .clickable(true)
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
                        ui.maximize();
                    }

                    // 8. Cross (Close)
                    let cls_key = Id::from_u64(999999905);
                    let cls_hov = ui.interaction_state.get(&cls_key).copied().unwrap_or(0.0) > 0.5;
                    let cls_resp = ui.container()
                        .clickable(true)
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
                        state.save_persistent_state();
                        ui.close();
                    }
                });
        });

    // User Profile Popup Overlay
    if state.user_profile_open {
        let pop_w = 230.0;
        let pop_x = (ui.width - pop_w - 12.0).max(10.0);
        let pop_y = BAR_H + 4.0;

        ui.container()
            .id(Id::from_u64(999999089))
            .absolute(pop_x, pop_y)
            .overlay()
            .width(pop_w)
            .bg(colors.bg_sidebar)
            .border(colors.border, 1.0)
            .radius_all(8.0)
            .padding(12.0, 12.0, 12.0, 12.0)
            .column()
            .gap(10.0)
            .show(|ui| {
                // Header with avatar and user details
                ui.container()
                    .row()
                    .valign(Align::Center)
                    .gap(10.0)
                    .show(|ui| {
                        ui.container()
                            .width(36.0).height(36.0)
                            .radius_all(18.0)
                            .bg(colors.accent.with_alpha(0.2))
                            .border(colors.accent, 1.0)
                            .halign(Align::Center)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text(zenthra::icons::NF_FA_USER)
                                    .size(16.0)
                                    .color(colors.accent)
                                    .show();
                            });

                        ui.container()
                            .column()
                            .gap(2.0)
                            .show(|ui| {
                                ui.text("Kabir")
                                    .size(13.0)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();

                                ui.text("Developer • Owner")
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            });
                    });

                // Divider
                ui.container()
                    .full_width().height(1.0)
                    .bg(colors.border)
                    .show(|_| {});

                // Workspace info
                let ws_name = if state.has_workspace {
                    state.current_dir.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("Workspace")
                } else {
                    "No Folder"
                };

                ui.container()
                    .column()
                    .gap(3.0)
                    .show(|ui| {
                        ui.text("WORKSPACE")
                            .size(9.0)
                            .weight(FontWeight::Bold)
                            .color(colors.text_muted)
                            .show();

                        ui.text(ws_name)
                            .size(11.0)
                            .weight(FontWeight::Medium)
                            .color(colors.text_primary)
                            .show();
                    });

                // Status indicators
                ui.container()
                    .row()
                    .gap(6.0)
                    .show(|ui| {
                        ui.container()
                            .padding(2.0, 6.0, 2.0, 6.0)
                            .bg(colors.bg_base)
                            .border(colors.border, 1.0)
                            .radius_all(3.0)
                            .show(|ui| {
                                ui.text(&format!("{} Tabs", state.tabs.len()))
                                    .size(9.5)
                                    .color(colors.text_muted)
                                    .show();
                            });

                        ui.container()
                            .padding(2.0, 6.0, 2.0, 6.0)
                            .bg(colors.bg_base)
                            .border(colors.border, 1.0)
                            .radius_all(3.0)
                            .show(|ui| {
                                ui.text("PTY: Active")
                                    .size(9.5)
                                    .color(Color::rgb(184.0 / 255.0, 187.0 / 255.0, 38.0 / 255.0))
                                    .show();
                            });
                    });

                // Close button
                let close_prof = ui.container()
                    .full_width()
                    .padding(5.0, 0.0, 5.0, 0.0)
                    .bg(colors.bg_base)
                    .hover_bg(colors.highlight)
                    .border(colors.border, 1.0)
                    .radius_all(4.0)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text("Close")
                            .size(10.5)
                            .color(colors.text_muted)
                            .show();
                    });

                if close_prof.clicked {
                    state.user_profile_open = false;
                    ui.needs_redraw = true;
                }
            });
    }

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

