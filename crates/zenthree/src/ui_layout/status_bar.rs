use crate::state::ZenthreeState;
use zenthra::{Ui, Align, Color, FontWeight, Style, use_context};
use crate::theme::ThemeColors;

pub fn draw_status_bar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let avail_w = ui.available_width;
    let bar_x = ui.cursor_x;
    let bar_y = ui.cursor_y;

    ui.container()
        .fill_x()
        .height(24.0)
        .bg(colors.bg_sidebar)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween) // SpaceBetween: Left status vs Right details
        .padding_left(12.0)
        .padding_right(8.0)
        .show(|ui| {
            // Left: Mode badge (NORMAL / INSERT) & status message
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(8.0)
                .show(|ui| {
                    let (mode_name, mode_bg, mode_color) = if state.focused_panel == crate::state::types::FocusedPanel::Terminal {
                        ("TERMINAL", Color::rgb(0.95, 0.60, 0.20).with_alpha(0.20), Color::rgb(0.95, 0.65, 0.25))
                    } else if state.focused_panel == crate::state::types::FocusedPanel::Explorer {
                        ("EXPLORER", colors.accent.with_alpha(0.20), colors.accent)
                    } else if state.vim_mode && state.editor_vim_normal {
                        ("NORMAL", colors.accent.with_alpha(0.25), colors.accent)
                    } else {
                        ("INSERT", Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0).with_alpha(0.20), Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                    };

                    ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(3.0)
                        .bg(mode_bg)
                        .show(|ui| {
                            ui.text(mode_name)
                                .size(9.5)
                                .font_weight(FontWeight::Bold)
                                .color(mode_color)
                                .show();
                        });

                    let status_text = state.status_message.clone();
                    if !status_text.is_empty() {
                        let max_chars = ((avail_w * 0.35) / 7.0).max(10.0) as usize;
                        let msg = if status_text.len() > max_chars {
                            format!("{}...", &status_text[..max_chars - 3])
                        } else {
                            status_text
                        };
                        ui.text(&msg)
                            .size(11.0)
                            .color(colors.text_muted)
                            .show();
                    } else {
                        ui.text("Ready")
                            .size(11.0)
                            .color(colors.text_muted.with_alpha(0.6))
                            .show();
                    }
                });

            // Right: cursor and buffer details & Zenthree IDE tag
            ui.container()
                .height(24.0)
                .row()
                .valign(Align::Center)
                .gap(12.0)
                .show(|ui| {
                    let mut show_extensions_view = false;

                    if let Some(active_idx) = state.active_tab {
                        if active_idx < state.tabs.len() {
                            let (cursor_row, cursor_col, line_count, lang_info, tab_fpath) = {
                                let tab = &state.tabs[active_idx];
                                let cursor = tab.editor.cursor();
                                let lc = tab.editor.buffer.line_count().max(1);
                                let li = if let Some(ref path) = tab.path {
                                    crate::extensions::ExtensionCatalog::find_by_path(path)
                                        .map(|e| (e.name, state.is_extension_installed(e.id), path.clone()))
                                } else {
                                    None
                                };
                                (cursor.row + 1, cursor.column + 1, lc, li, tab.path.clone())
                            };

                            // LSP Diagnostics (Errors / Warnings)
                            let (errors, warnings) = state.lsp_manager.get_diagnostic_counts(tab_fpath.as_deref());
                            if errors > 0 || warnings > 0 {
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(4.0)
                                    .show(|ui| {
                                        if errors > 0 {
                                            ui.text(&format!("✕ {}", errors))
                                                .size(10.5)
                                                .color(Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0))
                                                .show();
                                        }
                                        if warnings > 0 {
                                            ui.text(&format!("⚠ {}", warnings))
                                                .size(10.5)
                                                .color(Color::rgb(234.0 / 255.0, 179.0 / 255.0, 8.0 / 255.0))
                                                .show();
                                        }
                                    });
                            }

                            // LSP Server Indicator & 1-Click Install Button
                            if let Some(ref p) = tab_fpath {
                                if let Some(server_info) = crate::lsp::LspServerInfo::for_path(p) {
                                    let is_active = state.lsp_manager.sessions.get(server_info.language_id)
                                        .map(|s| s.status == crate::lsp::ServerStatus::Ready)
                                        .unwrap_or(false);
                                    let is_installed = crate::lsp::LspManager::is_language_installed(server_info.language_id);
                                    let is_installing = state.is_lsp_installing(server_info.language_id);

                                    if is_installing {
                                        let lsp_badge = ui.container()
                                            .padding(2.0, 6.0, 2.0, 6.0)
                                            .radius_all(3.0)
                                            .bg(colors.accent.with_alpha(0.18))
                                            .hover(Style::bg(colors.accent.with_alpha(0.28)))
                                            .interactive(true)
                                            .row()
                                            .valign(Align::Center)
                                            .gap(4.0)
                                            .show(|ui| {
                                                ui.text(&format!("⠋ \u{f0627} Installing {}...", server_info.display_name))
                                                    .size(10.0)
                                                    .color(colors.accent)
                                                    .show();
                                            });
                                        if lsp_badge.clicked {
                                            state.extension_main_tab = crate::state::types::ExtensionMainTab::LanguageServers;
                                            show_extensions_view = true;
                                        }
                                    } else if is_active {
                                        let lsp_btn = ui.button(&format!("\u{f0627} {}", server_info.display_name))
                                            .id(zenthra::Id::from_u64(999998141))
                                            .height(18.0)
                                            .bg(colors.accent.with_alpha(0.18))
                                            .hover(Style::bg(colors.accent.with_alpha(0.35)))
                                            .color(colors.accent)
                                            .radius_all(3.0)
                                            .padding_left(6.0)
                                            .padding_right(6.0)
                                            .size(10.0)
                                            .show();
                                        if lsp_btn.clicked {
                                            state.extension_main_tab = crate::state::types::ExtensionMainTab::LanguageServers;
                                            show_extensions_view = true;
                                        }
                                    } else if is_installed {
                                        let lsp_btn = ui.button(&format!("\u{f0627} {}: ready", server_info.display_name))
                                            .id(zenthra::Id::from_u64(999998142))
                                            .height(18.0)
                                            .bg(Color::TRANSPARENT)
                                            .hover(Style::bg(colors.highlight))
                                            .color(colors.text_muted)
                                            .radius_all(3.0)
                                            .padding_left(6.0)
                                            .padding_right(6.0)
                                            .size(10.0)
                                            .show();
                                        if lsp_btn.clicked {
                                            state.extension_main_tab = crate::state::types::ExtensionMainTab::LanguageServers;
                                            show_extensions_view = true;
                                        }
                                    } else {
                                        let install_btn = ui.button(&format!("\u{f0627} Install {} LSP", server_info.display_name))
                                            .id(zenthra::Id::from_u64(999998140))
                                            .height(18.0)
                                            .bg(colors.accent.with_alpha(0.25))
                                            .hover(Style::bg(colors.accent.with_alpha(0.50)))
                                            .color(colors.accent)
                                            .radius_all(3.0)
                                            .padding_left(6.0)
                                            .padding_right(6.0)
                                            .size(10.0)
                                            .show();

                                        if install_btn.clicked {
                                            state.start_lsp_install(server_info.language_id);
                                        }
                                    }
                                }
                            }

                            if let Some((lang_name, _, _)) = lang_info {
                                let lang_btn = ui.button(lang_name)
                                    .id(zenthra::Id::from_u64(999998150))
                                    .height(18.0)
                                    .bg(colors.accent.with_alpha(0.15))
                                    .hover(Style::bg(colors.accent.with_alpha(0.35)))
                                    .color(colors.accent)
                                    .radius_all(3.0)
                                    .padding_left(6.0)
                                    .padding_right(6.0)
                                    .size(10.5)
                                    .show();

                                if lang_btn.clicked {
                                    state.extension_main_tab = crate::state::types::ExtensionMainTab::LanguagePacks;
                                    show_extensions_view = true;
                                }
                            } else if let Some(ref p) = tab_fpath {
                                let fallback_label = p.extension()
                                    .and_then(|e| e.to_str())
                                    .map(|ext| format!(".{}", ext))
                                    .unwrap_or_else(|| "Plain Text".to_string());
                                let lang_btn = ui.button(&fallback_label)
                                    .id(zenthra::Id::from_u64(999998150))
                                    .height(18.0)
                                    .bg(colors.highlight.with_alpha(0.25))
                                    .hover(Style::bg(colors.highlight))
                                    .color(colors.text_muted)
                                    .radius_all(3.0)
                                    .padding_left(6.0)
                                    .padding_right(6.0)
                                    .size(10.5)
                                    .show();

                                if lang_btn.clicked {
                                    state.extension_main_tab = crate::state::types::ExtensionMainTab::LanguagePacks;
                                    show_extensions_view = true;
                                }
                            } else {
                                let lang_btn = ui.button("Plain Text")
                                    .id(zenthra::Id::from_u64(999998150))
                                    .height(18.0)
                                    .bg(colors.highlight.with_alpha(0.25))
                                    .hover(Style::bg(colors.highlight))
                                    .color(colors.text_muted)
                                    .radius_all(3.0)
                                    .padding_left(6.0)
                                    .padding_right(6.0)
                                    .size(10.5)
                                    .show();

                                if lang_btn.clicked {
                                    state.extension_main_tab = crate::state::types::ExtensionMainTab::LanguagePacks;
                                    show_extensions_view = true;
                                }
                            }

                            // Cursor position visible if window >= 400px
                            if avail_w >= 400.0 {
                                ui.text(&format!("Ln {}, Col {}", cursor_row, cursor_col))
                                    .size(11.0)
                                    .color(colors.text_muted)
                                    .show();
                            }

                            // Total line count visible if window >= 650px
                            if avail_w >= 650.0 {
                                ui.text(&format!("{} lines", line_count))
                                    .size(11.0)
                                    .color(colors.text_muted)
                                    .show();
                            }
                        }
                    }

                    if show_extensions_view {
                        state.sidebar_view = crate::state::SidebarView::Extensions;
                        state.sidebar_visible = true;
                        ui.request_redraw();
                    }

                    // Encoding visible if window >= 500px
                    if avail_w >= 500.0 {
                        ui.text("UTF-8")
                            .size(11.0)
                            .color(colors.text_muted)
                            .show();
                    }

                    // Clock segment: Powerline arrow (\u{e0b2}) + Clock icon (\u{f017}) + Time
                    let current_time = chrono::Local::now().format("%H:%M").to_string();
                    let clock_bg = Color::rgb(181.0 / 255.0, 232.0 / 255.0, 144.0 / 255.0);
                    let clock_fg = Color::rgb(24.0 / 255.0, 24.0 / 255.0, 32.0 / 255.0);

                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(0.0)
                        .show(|ui| {
                            // Shift triangle physically to the left
                            ui.cursor_x -= 5.0;

                            // Powerline left triangle arrow (\u{e0b2})
                            ui.text("\u{e0b2}")
                                .size(18.0)
                                .color(clock_bg)
                                .show();

                            // Seamlessly join with pill body
                            ui.cursor_x -= 1.0;

                            // Clock pill body
                            ui.container()
                                .height(18.0)
                                .bg(clock_bg)
                                .padding_left(2.0)
                                .padding_right(6.0)
                                .row()
                                .valign(Align::Center)
                                .gap(4.0)
                                .show(|ui| {
                                    ui.text("\u{f017}")
                                        .size(11.0)
                                        .color(clock_fg)
                                        .show();
                                    ui.text(&current_time)
                                        .size(11.0)
                                        .font_weight(FontWeight::Medium)
                                        .color(clock_fg)
                                        .show();
                                });
                        });

                    // Panels button (\u{ebeb})
                    let panel_btn_id = zenthra::Id::from_u64(999999125);
                    let panel_btn = ui.container()
                        .raw_id(panel_btn_id)
                        .interactive(true)
                        .height(18.0)
                        .padding_left(6.0)
                        .padding_right(6.0)
                        .bg(if state.sidebar_visible { colors.accent.with_alpha(0.20) } else { Color::TRANSPARENT })
                        .hover(Style::bg(colors.highlight))
                        .radius_all(3.0)
                        .row()
                        .valign(Align::Center)
                        .gap(4.0)
                        .show(|ui| {
                            ui.text("\u{ebeb}")
                                .size(11.0)
                                .color(if state.sidebar_visible { colors.accent } else { colors.text_muted })
                                .show();
                            ui.text("Panels")
                                .size(10.0)
                                .color(colors.text_muted)
                                .show();
                        });

                    if panel_btn.clicked {
                        state.sidebar_visible = !state.sidebar_visible;
                        ui.request_redraw();
                    }

                    // Tag: Zenthree IDE (square badge inside bottom bar with accent color at 60% opacity)
                    let mut tag_bg = colors.accent;
                    tag_bg.a = 0.6;

                    ui.container()
                        .height(18.0)
                        .bg(tag_bg)
                        .radius_all(0.0)
                        .padding_left(8.0)
                        .padding_right(8.0)
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text("Zenthree IDE")
                                .size(10.5)
                                .color(Color::WHITE)
                                .show();
                        });
                });
        });

    // Top 1px divider line separating status bar from workspace
    ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
        instance: zenthra_render::RectInstance {
            pos: [bar_x, bar_y],
            size: [avail_w, 1.0],
            color: colors.border.to_array(),
            radius: [0.0; 4],
            border_width: 0.0,
            border_color: [0.0; 4],
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));
}
