use crate::state::ZenthreeState;
use crate::theme::{ThemeMode, ACCENT_COLOR_OPTIONS, ICON_THEME_OPTIONS};
use zenthra::{Color, Ui, FontWeight, Align, Id, use_context};
use crate::theme::ThemeColors;

pub fn draw_title_bar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    crate::theme::sync_theme_to_ui(ui, state.theme, &colors, state.glassmorphism);

    // Reset hover flag for menus in title bar since we do not use menu_bar wrapper
    let hover_flag_key = Id::from_u64(999999902);
    ui.interaction_state.insert(hover_flag_key, 0.0);

    const BAR_H: f32 = 36.0;

    let titlebar_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha((state.glass_opacity * 0.55).clamp(0.18, 0.65))
    } else {
        colors.bg_sidebar
    };

    ui.container()
        .full_width()
        .height(BAR_H)
        .bg(titlebar_bg)
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
                                state.request_open_workspace(path);
                            }
                            ui.request_redraw();
                        }
                        if state.has_workspace {
                            if ui.menu_item("Close Folder").show().clicked {
                                state.request_close_workspace();
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
                                state.set_status("Line endings set to LF");
                                ui.request_redraw();
                            }
                            if ui.menu_item("  CRLF (Windows)").show().clicked {
                                state.set_status("Line endings set to CRLF");
                                ui.request_redraw();
                            }
                        });

                        ui.spacing(4.0);

                        // Cascading Submenu: Preferences
                        ui.sub_menu("Preferences").show(|ui| {
                            if ui.menu_item("Editor Settings").shortcut("Ctrl+,").show().clicked {
                                state.editor_settings_open = true;
                                state.editor_settings_just_opened = true;
                                state.editor_settings_tab = crate::state::EditorSettingsTab::Appearance;
                                ui.request_redraw();
                            }
                            if ui.menu_item("AI & Agent Settings").show().clicked {
                                state.ai_settings.open = true;
                                state.ai_settings.just_opened = true;
                                ui.request_redraw();
                            }
                        });

                        ui.spacing(4.0);
                        if ui.menu_item("Exit").show().clicked {
                            state.cancel_active_ai();
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
                                state.set_status("Transformed to UPPERCASE");
                                ui.request_redraw();
                            }
                            if ui.menu_item("lowercase").show().clicked {
                                state.set_status("Transformed to lowercase");
                                ui.request_redraw();
                            }
                            if ui.menu_item("Title Case").show().clicked {
                                state.set_status("Transformed to Title Case");
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
                                state.set_status("Selected all");
                                ui.request_redraw();
                            }
                            if ui.menu_item("Expand Selection").show().clicked {
                                state.set_status("Expand Selection");
                                ui.request_redraw();
                            }
                            if ui.menu_item("Shrink Selection").show().clicked {
                                state.set_status("Shrink Selection");
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

                        let orbs_mark = if state.thinking_orbs_demo_open { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Thinking Orbs (9-Mode Showcase)", orbs_mark)).show().clicked {
                            state.thinking_orbs_demo_open = !state.thinking_orbs_demo_open;
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
                            let glass_mark = if state.glassmorphism { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Native Window Blur", glass_mark)).show().clicked {
                                state.glassmorphism = !state.glassmorphism;
                                ui.set_blur(state.glassmorphism);
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
                                state.zoom_in();
                                ui.request_redraw();
                            }
                            if ui.menu_item("Zoom Out").shortcut("Ctrl -").show().clicked {
                                state.zoom_out();
                                ui.request_redraw();
                            }
                            ui.spacing(4.0);
                            if ui.menu_item(&format!("{}100% (Reset)", z100)).shortcut("Ctrl 0").show().clicked {
                                state.reset_zoom();
                                ui.request_redraw();
                            }
                            if ui.menu_item(&format!("{}125%", z125)).show().clicked {
                                state.set_zoom_level(1.25);
                                ui.request_redraw();
                            }
                            if ui.menu_item(&format!("{}150%", z150)).show().clicked {
                                state.set_zoom_level(1.5);
                                ui.request_redraw();
                            }
                        });
                    });

                    ui.menu("Theme").show(|ui| {
                        // Cascading Submenu: Color Mode
                        ui.sub_menu("Color Mode").show(|ui| {
                            let dark_mark = if state.theme == ThemeMode::Dark { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Dark Theme", dark_mark)).show().clicked {
                                state.set_theme(ThemeMode::Dark);
                                state.highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::dark());
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                            let light_mark = if state.theme == ThemeMode::Light { "● " } else { "  " };
                            if ui.menu_item(&format!("{}Light Theme", light_mark)).show().clicked {
                                state.set_theme(ThemeMode::Light);
                                state.highlighter = syntax::SyntaxHighlighter::new(syntax::SyntaxTheme::light());
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                        });

                        // Native Window Blur Mode Toggle
                        let glass_mark = if state.glassmorphism { "● " } else { "  " };
                        if ui.menu_item(&format!("{}Native Window Blur", glass_mark)).show().clicked {
                            state.glassmorphism = !state.glassmorphism;
                            ui.set_blur(state.glassmorphism);
                            state.save_persistent_state();
                            ui.request_redraw();
                        }

                        ui.spacing(4.0);

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
                                    state.set_status(format!("Icon theme: {}", theme_name));
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
                            state.about_just_opened = true;
                            ui.request_redraw();
                        }
                    });
                });

            // RIGHT: window controls & panel toggles
            // Sequence: left, bottom, right, user icon, gap, minimize, floating window, cross
            ui.container()
                .height(BAR_H)
                .row()
                .valign(Align::Center)
                .padding_right(4.0)
                .gap(6.0)
                .show(|ui| {
                    // 0. Focus Mode (\u{e25d})
                    let focus_key = Id::from_u64(999999909);
                    let focus_hov = ui.interaction_state.get(&focus_key).copied().unwrap_or(0.0) > 0.5;
                    let focus_resp = ui.container()
                        .clickable(true)
                        .width(22.0).height(20.0)
                        .radius_all(3.0)
                        .bg(if focus_hov {
                            colors.highlight
                        } else if state.focus_mode {
                            colors.accent.with_alpha(0.15)
                        } else {
                            Color::TRANSPARENT
                        })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let icon_color = if state.focus_mode {
                                colors.accent
                            } else if focus_hov {
                                colors.text_primary
                            } else {
                                colors.text_muted
                            };
                            ui.text("\u{e25d}")
                                .size(13.0)
                                .color(icon_color)
                                .show();
                        });
                    let h = focus_resp.hovered;
                    let prev = ui.interaction_state.insert(focus_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if focus_resp.clicked {
                        state.focus_mode = !state.focus_mode;
                        if state.focus_mode {
                            // Entering focus mode: hide title bar + activity bar + terminal + AI
                            state.activity_bar_visible = false;
                            state.terminal_visible = false;
                            state.ai_panel_visible = false;
                        } else {
                            // Exiting focus mode: restore activity bar
                            state.activity_bar_visible = true;
                        }
                        ui.request_redraw();
                    }

                    // 1. Left Panel (File Tree) Toggle (\u{f10aa})
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
                            let icon_color = if state.sidebar_visible {
                                colors.accent
                            } else if left_hov {
                                colors.text_primary
                            } else {
                                colors.text_muted
                            };
                            ui.text("\u{f10aa}")
                                .size(13.0)
                                .color(icon_color)
                                .show();
                        });
                    let h = left_resp.hovered;
                    let prev = ui.interaction_state.insert(left_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if left_resp.clicked {
                        state.sidebar_visible = !state.sidebar_visible;
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 2. Bottom Panel (Terminal) Toggle (\u{f10a9})
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
                            let icon_color = if state.terminal_visible {
                                colors.accent
                            } else if btm_hov {
                                colors.text_primary
                            } else {
                                colors.text_muted
                            };
                            ui.text("\u{f10a9}")
                                .size(13.0)
                                .color(icon_color)
                                .show();
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

                    // 3. Right Panel (AI Agent) Toggle (\u{f10ab})
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
                            let icon_color = if state.ai_panel_visible {
                                colors.accent
                            } else if right_hov {
                                colors.text_primary
                            } else {
                                colors.text_muted
                            };
                            ui.text("\u{f10ab}")
                                .size(13.0)
                                .color(icon_color)
                                .show();
                        });
                    let h = right_resp.hovered;
                    let prev = ui.interaction_state.insert(right_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if right_resp.clicked {
                        state.ai_panel_visible = !state.ai_panel_visible;
                        state.save_persistent_state();
                        ui.request_redraw();
                    }

                    // 4. User Icon (\u{f2bd}) - Clean, unconstrained glyph without small enclosing circle/padding
                    let user_key = Id::from_u64(999999913);
                    let user_hov = ui.interaction_state.get(&user_key).copied().unwrap_or(0.0) > 0.5;
                    let user_resp = ui.container()
                        .clickable(true)
                        .width(22.0).height(22.0)
                        .radius_all(4.0)
                        .bg(if user_hov {
                            colors.highlight
                        } else if state.user_profile_open {
                            colors.accent.with_alpha(0.18)
                        } else {
                            Color::TRANSPARENT
                        })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f2bd}")
                                .size(14.0)
                                .color(if state.user_profile_open {
                                    colors.accent
                                } else if user_hov {
                                    colors.text_primary
                                } else {
                                    colors.text_muted
                                })
                                .show();
                        });
                    let h = user_resp.hovered;
                    let prev = ui.interaction_state.insert(user_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if user_resp.clicked {
                        state.user_profile_open = !state.user_profile_open;
                        ui.consume_click();
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

                    // 6. Minimize (\u{e224})
                    let min_key = Id::from_u64(999999903);
                    let min_hov = ui.interaction_state.get(&min_key).copied().unwrap_or(0.0) > 0.5;
                    let min_resp = ui.container()
                        .clickable(true)
                        .width(24.0).height(24.0)
                        .radius_all(4.0)
                        .bg(if min_hov {
                            match state.theme {
                                ThemeMode::Dark => Color::rgba(1.0, 1.0, 1.0, 0.08),
                                ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.06),
                            }
                        } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{e224}")
                                .size(13.0)
                                .color(if min_hov { colors.text_primary } else { colors.text_muted })
                                .show();
                        });
                    let h = min_resp.hovered;
                    let prev = ui.interaction_state.insert(min_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if min_resp.clicked {
                        ui.minimize();
                    }

                    // 7. Maximize / Restore (\u{f0616})
                    let max_key = Id::from_u64(999999904);
                    let max_hov = ui.interaction_state.get(&max_key).copied().unwrap_or(0.0) > 0.5;
                    let max_resp = ui.container()
                        .clickable(true)
                        .width(24.0).height(24.0)
                        .radius_all(4.0)
                        .bg(if max_hov {
                            match state.theme {
                                ThemeMode::Dark => Color::rgba(1.0, 1.0, 1.0, 0.08),
                                ThemeMode::Light => Color::rgba(0.0, 0.0, 0.0, 0.06),
                            }
                        } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("\u{f0616}")
                                .size(13.0)
                                .color(if max_hov { colors.text_primary } else { colors.text_muted })
                                .show();
                        });
                    let h = max_resp.hovered;
                    let prev = ui.interaction_state.insert(max_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if max_resp.clicked {
                        ui.maximize();
                    }

                    // 8. Close (\u{f0159}) - Clean unconstrained button with larger glyph
                    let cls_key = Id::from_u64(999999905);
                    let cls_hov = ui.interaction_state.get(&cls_key).copied().unwrap_or(0.0) > 0.5;
                    let cls_resp = ui.container()
                        .clickable(true)
                        .width(24.0).height(24.0)
                        .radius_all(4.0)
                        .bg(if cls_hov { Color::rgb(0.85, 0.15, 0.15) } else { Color::TRANSPARENT })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .show(|ui| {
                            let fg = if cls_hov { Color::WHITE } else { colors.text_muted };
                            ui.text("\u{f0159}").size(14.0).color(fg).show();
                        });
                    let h = cls_resp.hovered;
                    let prev = ui.interaction_state.insert(cls_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }
                    if cls_resp.clicked {
                        state.cancel_active_ai();
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
        let profile_popup_id = Id::from_u64(999999089);

        ui.container()
            .raw_id(profile_popup_id)
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
                let close_profile_id = Id::from_u64(999999088);
                let close_prof = ui.container()
                    .raw_id(close_profile_id)
                    .full_width()
                    .clickable(true)
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

                let close_rect = ui.screen_layout_cache.get(&close_profile_id)
                    .or_else(|| ui.next_screen_layout_cache.get(&close_profile_id))
                    .copied();
                let pointer_over_close = close_rect
                    .map(|rect| ui.mouse_in_rect(rect.origin.x, rect.origin.y, rect.size.width, rect.size.height))
                    .unwrap_or(false);
                if close_prof.clicked || (ui.clicked && pointer_over_close) {
                    state.user_profile_open = false;
                    ui.consume_click();
                    ui.needs_redraw = true;
                }
            });

        let profile_rect = ui.next_screen_layout_cache.get(&profile_popup_id)
            .or_else(|| ui.screen_layout_cache.get(&profile_popup_id))
            .copied();
        if ui.clicked
            && profile_rect
                .map(|rect| !ui.mouse_in_rect(rect.origin.x, rect.origin.y, rect.size.width, rect.size.height))
                .unwrap_or(false)
        {
            state.user_profile_open = false;
            ui.request_redraw();
        }
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

    // Bottom 1px divider line separating title bar from workspace
    ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [0.0, BAR_H - 1.0],
            size: [ui.width, 1.0],
            color: colors.border.to_array(),
            radius: [0.0; 4],
            border_width: 0.0,
            border_color: [0.0; 4],
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));
}
