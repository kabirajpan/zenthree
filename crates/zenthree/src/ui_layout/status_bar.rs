use crate::state::ZenthreeState;
use zenthra::{Ui, Align, Color};

pub fn draw_status_bar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();
    let avail_w = ui.available_width;

    ui.container()
        .full_width()
        .height(24.0)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween) // SpaceBetween: Left status vs Right details
        .no_wrap()
        .padding_left(12.0)
        .padding_right(8.0)
        .show(|ui| {
            // Left: status message
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    if !state.status_message.is_empty() {
                        let max_chars = ((avail_w * 0.35) / 7.0).max(10.0) as usize;
                        let msg = if state.status_message.len() > max_chars {
                            format!("{}...", &state.status_message[..max_chars - 3])
                        } else {
                            state.status_message.clone()
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
                    let mut prompt_lang_path: Option<std::path::PathBuf> = None;
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

                            // LSP Server Indicator
                            if let Some(ref p) = tab_fpath {
                                if let Some(server_info) = crate::lsp::LspServerInfo::for_path(p) {
                                    let is_active = state.lsp_manager.sessions.get(server_info.language_id)
                                        .map(|s| s.status == crate::lsp::ServerStatus::Ready)
                                        .unwrap_or(false);
                                    let is_installed = crate::lsp::LspManager::is_server_installed(server_info.binary_name);

                                    let (status_text, status_color) = if is_active {
                                        (format!("⚡ {}", server_info.display_name), colors.accent)
                                    } else if is_installed {
                                        (format!("⚡ {}: ready", server_info.display_name), colors.text_muted)
                                    } else {
                                        (format!("⚡ {}: not installed", server_info.display_name), colors.text_dim)
                                    };

                                    ui.text(&status_text)
                                        .size(10.0)
                                        .color(status_color)
                                        .show();
                                }
                            }

                            if let Some((lang_name, installed, fpath)) = lang_info {
                                let label = if installed {
                                    lang_name.to_string()
                                } else {
                                    format!("{} [Install]", lang_name)
                                };
                                let lang_btn = ui.button(&label)
                                    .id(zenthra::Id::from_u64(999998150))
                                    .height(18.0)
                                    .bg(if installed { colors.accent.with_alpha(0.15) } else { colors.accent.with_alpha(0.35) })
                                    .hover_bg(colors.accent.with_alpha(0.5))
                                    .text_color(if installed { colors.accent } else { Color::WHITE })
                                    .radius_all(3.0)
                                    .padding_left(6.0)
                                    .padding_right(6.0)
                                    .size(10.5)
                                    .show();

                                if lang_btn.clicked {
                                    if !installed {
                                        prompt_lang_path = Some(fpath);
                                    } else {
                                        show_extensions_view = true;
                                    }
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

                    if let Some(p) = prompt_lang_path {
                        state.check_language_support_prompt(&p);
                        ui.request_redraw();
                    } else if show_extensions_view {
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
}
