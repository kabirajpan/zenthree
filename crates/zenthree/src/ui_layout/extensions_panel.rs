use crate::state::{ZenthreeState, ExtensionFilterTab, ExtensionMainTab};
use crate::extensions::ExtensionCatalog;
use crate::lsp::LanguageServerCatalogItem;
use zenthra::{Color, Ui, Align, FontWeight, Id, use_context};
use crate::theme::ThemeColors;

pub fn draw_extensions_panel(ui: &mut Ui, state: &mut ZenthreeState, panel_w: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    // Dimensions derived directly from panel_w so everything fits strictly
    let card_w = (panel_w - 20.0).max(140.0);
    let inner_w = (card_w - 16.0).max(120.0);

    // Outer column layout
    ui.container()
        .width(panel_w)
        .fill_y()
        .clip(true)
        .column()
        .show(|ui| {
            // 1. Two Main Tabs: [  Language Packs ] vs [ 󰘧 Language Servers ] (50/50 Notes panel style)
            let is_packs = state.extension_main_tab == ExtensionMainTab::LanguagePacks;
            let is_servers = state.extension_main_tab == ExtensionMainTab::LanguageServers;

            let total_avail = (panel_w - 16.0 - 4.0 - 2.0).max(60.0);
            let tab_w = (total_avail * 0.50).floor();

            ui.container()
                .full_width()
                .padding(6.0, 8.0, 3.0, 8.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
                        .height(26.0)
                        .bg(colors.bg_base)
                        .border(colors.border.with_alpha(0.4), 1.0)
                        .radius_all(4.0)
                        .padding(2.0, 2.0, 2.0, 2.0)
                        .row()
                        .valign(Align::Center)
                        .gap(2.0)
                        .show(|ui| {
                            // Tab 1: Language Packs (50%)
                            let btn_packs = ui.button(" Language Packs")
                                .id(Id::from_u64(999998190))
                                .height(22.0)
                                .width(tab_w)
                                .padding(0.0, 4.0, 0.0, 4.0)
                                .bg(if is_packs { colors.accent.with_alpha(0.20) } else { Color::TRANSPARENT })
                                .hover_bg(if is_packs { colors.accent.with_alpha(0.28) } else { Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.07) })
                                .text_color(if is_packs { colors.accent } else { colors.text_muted })
                                .radius_all(3.0)
                                .size(10.5)
                                .show();
                            if btn_packs.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if btn_packs.clicked && !is_packs {
                                state.extension_main_tab = ExtensionMainTab::LanguagePacks;
                                ui.request_redraw();
                            }

                            // Tab 2: Language Servers (50%)
                            let btn_servers = ui.button("\u{f0627} Language Servers")
                                .id(Id::from_u64(999998191))
                                .height(22.0)
                                .width(tab_w)
                                .padding(0.0, 4.0, 0.0, 4.0)
                                .bg(if is_servers { colors.accent.with_alpha(0.20) } else { Color::TRANSPARENT })
                                .hover_bg(if is_servers { colors.accent.with_alpha(0.28) } else { Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.07) })
                                .text_color(if is_servers { colors.accent } else { colors.text_muted })
                                .radius_all(3.0)
                                .size(10.5)
                                .show();
                            if btn_servers.hovered {
                                ui.cursor_icon = zenthra::CursorIcon::Pointer;
                            }
                            if btn_servers.clicked && !is_servers {
                                state.extension_main_tab = ExtensionMainTab::LanguageServers;
                                ui.request_redraw();
                            }
                        });
                });

            // 2. Search Box
            ui.container()
                .full_width()
                .padding(2.0, 10.0, 4.0, 10.0)
                .column()
                .gap(6.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
                        .height(28.0)
                        .bg(colors.bg_base)
                        .border(colors.border, 1.0)
                        .radius_all(4.0)
                        .padding_left(8.0)
                        .padding_right(8.0)
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text(zenthra::icons::NF_FA_SEARCH)
                                .size(11.0)
                                .color(colors.text_dim)
                                .show();

                            let query_empty = state.extension_search_query.is_empty();

                            ui.input(&mut state.extension_search_query, Id::from_u64(999998200))
                                .fill_x()
                                .no_bg()
                                .no_border()
                                .color(colors.text_primary)
                                .size(11.0)
                                .show();

                            if !query_empty {
                                let clear_btn = ui.container()
                                    .padding(2.0, 4.0, 2.0, 4.0)
                                    .radius_all(3.0)
                                    .hover_bg(colors.highlight)
                                    .clickable(true)
                                    .show(|ui| {
                                        ui.text("\u{f00d}")
                                            .size(9.0)
                                            .color(colors.text_muted)
                                            .show();
                                    });
                                if clear_btn.hovered { ui.cursor_icon = zenthra::CursorIcon::Pointer; }
                                if clear_btn.clicked {
                                    state.extension_search_query.clear();
                                    ui.needs_redraw = true;
                                }
                            }
                        });
                });

            // 3. Render Selected Tab Content
            match state.extension_main_tab {
                ExtensionMainTab::LanguagePacks => {
                    draw_language_packs_tab(ui, state, colors, card_w, inner_w);
                }
                ExtensionMainTab::LanguageServers => {
                    draw_language_servers_tab(ui, state, colors, card_w, inner_w);
                }
            }
        });
}

fn draw_language_packs_tab(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: ThemeColors,
    card_w: f32,
    inner_w: f32,
) {
    let all_extensions = ExtensionCatalog::all();
    let installed_count = all_extensions.iter().filter(|e| state.is_extension_installed(e.id)).count();
    let available_count = all_extensions.len() - installed_count;

    // Subheader: Filter Tabs
    ui.container()
        .full_width()
        .padding(2.0, 10.0, 6.0, 10.0)
        .row()
        .valign(Align::Center)
        .gap(4.0)
        .show(|ui| {
            let tab_all = state.extension_tab_filter == ExtensionFilterTab::All;
            let btn_all = ui.button(&format!("All ({})", all_extensions.len()))
                .id(Id::from_u64(999998201))
                .height(22.0)
                .bg(if tab_all { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                .text_color(if tab_all { colors.accent } else { colors.text_muted })
                .radius_all(3.0)
                .padding_left(6.0)
                .padding_right(6.0)
                .size(10.0)
                .show();
            if btn_all.clicked {
                state.extension_tab_filter = ExtensionFilterTab::All;
                ui.request_redraw();
            }

            let tab_inst = state.extension_tab_filter == ExtensionFilterTab::Installed;
            let btn_inst = ui.button(&format!("Installed ({})", installed_count))
                .id(Id::from_u64(999998202))
                .height(22.0)
                .bg(if tab_inst { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                .text_color(if tab_inst { colors.accent } else { colors.text_muted })
                .radius_all(3.0)
                .padding_left(6.0)
                .padding_right(6.0)
                .size(10.0)
                .show();
            if btn_inst.clicked {
                state.extension_tab_filter = ExtensionFilterTab::Installed;
                ui.request_redraw();
            }

            let tab_avail = state.extension_tab_filter == ExtensionFilterTab::Available;
            let btn_avail = ui.button(&format!("Available ({})", available_count))
                .id(Id::from_u64(999998203))
                .height(22.0)
                .bg(if tab_avail { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                .text_color(if tab_avail { colors.accent } else { colors.text_muted })
                .radius_all(3.0)
                .padding_left(6.0)
                .padding_right(6.0)
                .size(10.0)
                .show();
            if btn_avail.clicked {
                state.extension_tab_filter = ExtensionFilterTab::Available;
                ui.request_redraw();
            }
        });

    // Separator border
    ui.container()
        .full_width()
        .height(1.0)
        .bg(colors.border)
        .show(|_| {});

    // Scrollable List of Language Packs
    ui.container()
        .full_width()
        .fill_y()
        .scroll_y(true)
        .clip(true)
        .padding(6.0, 8.0, 6.0, 8.0)
        .gap(6.0)
        .show(|ui| {
            let search_query = &state.extension_search_query;

            let mut filtered: Vec<_> = all_extensions
                .into_iter()
                .filter(|ext| {
                    let is_inst = state.is_extension_installed(ext.id);
                    match state.extension_tab_filter {
                        ExtensionFilterTab::All => true,
                        ExtensionFilterTab::Installed => is_inst,
                        ExtensionFilterTab::Available => !is_inst,
                    }
                })
                .filter_map(|ext| {
                    let score = search_match_score(
                        search_query,
                        ext.name,
                        ext.extensions,
                        ext.id,
                        ext.description,
                        ext.features,
                    )?;
                    Some((score, ext))
                })
                .collect();

            // Sort by match score descending (exact extension match or name starts-with at top)
            filtered.sort_by(|a, b| b.0.cmp(&a.0));
            let filtered: Vec<_> = filtered.into_iter().map(|(_, ext)| ext).collect();

            if filtered.is_empty() {
                ui.container()
                    .full_width()
                    .padding(24.0, 10.0, 24.0, 10.0)
                    .valign(Align::Center)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text("No language packs found")
                            .size(11.0)
                            .color(colors.text_muted)
                            .show();
                    });
                return;
            }

            let mut to_install = None;
            let mut to_uninstall = None;

            for (idx, ext) in filtered.into_iter().enumerate() {
                let is_installed = state.is_extension_installed(ext.id);
                let card_id = Id::from_u64(999998300 + idx as u64);

                ui.container()
                    .id(card_id)
                    .width(card_w)
                    .clip(true)
                    .bg(colors.bg_sidebar)
                    .hover_bg(colors.highlight)
                    .border(colors.border, 1.0)
                    .radius_all(6.0)
                    .padding(8.0, 8.0, 8.0, 8.0)
                    .column()
                    .gap(5.0)
                    .show(|ui| {
                        // Top row: Icon + Name on Left, Action Button on Right
                        ui.container()
                            .width(inner_w)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                // Left: Icon + Title
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(6.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(22.0)
                                            .height(22.0)
                                            .bg(ext.icon_color.with_alpha(0.18))
                                            .radius_all(4.0)
                                            .valign(Align::Center)
                                            .halign(Align::Center)
                                            .show(|ui| {
                                                ui.text(ext.glyph)
                                                    .size(13.0)
                                                    .color(ext.icon_color)
                                                    .show();
                                            });

                                        ui.text(ext.name)
                                            .size(12.0)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();
                                    });

                                // Right: Install / Uninstall Action button
                                if is_installed {
                                    let uninst_btn = ui.button("Installed")
                                        .id(Id::from_u64(999998400 + idx as u64))
                                        .height(20.0)
                                        .bg(Color::TRANSPARENT)
                                        .hover_bg(Color::rgba(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0, 0.15))
                                        .text_color(colors.text_muted)
                                        .radius_all(3.0)
                                        .padding_left(6.0)
                                        .padding_right(6.0)
                                        .size(10.0)
                                        .show();

                                    if uninst_btn.clicked {
                                        to_uninstall = Some(ext.id);
                                    }
                                } else {
                                    let inst_btn = ui.button("Install")
                                        .id(Id::from_u64(999998400 + idx as u64))
                                        .height(20.0)
                                        .bg(colors.accent)
                                        .hover_bg(colors.accent.with_alpha(0.85))
                                        .text_color(Color::WHITE)
                                        .radius_all(3.0)
                                        .padding_left(8.0)
                                        .padding_right(8.0)
                                        .size(10.0)
                                        .show();

                                    if inst_btn.clicked {
                                        to_install = Some(ext.id);
                                    }
                                }
                            });

                        // Row 2: Description
                        ui.text(ext.description)
                            .size(10.0)
                            .color(colors.text_muted)
                            .max_width(inner_w)
                            .wrap(true)
                            .show();

                        // Row 3: Meta
                        ui.container()
                            .width(inner_w)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container()
                                    .row()
                                    .gap(4.0)
                                    .show(|ui| {
                                        for ext_tag in ext.extensions.iter().take(3) {
                                            ui.container()
                                                .bg(colors.bg_editor)
                                                .radius_all(2.0)
                                                .padding(1.0, 4.0, 1.0, 4.0)
                                                .show(|ui| {
                                                    ui.text(&format!(".{}", ext_tag))
                                                        .size(9.0)
                                                        .color(colors.text_dim)
                                                        .show();
                                                });
                                        }
                                    });

                                 let ver_label = format!("v{}", ext.version);
                                ui.text(if is_installed { "✓ Installed" } else { &ver_label })
                                    .size(9.5)
                                    .color(if is_installed { Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0) } else { colors.text_dim })
                                    .show();
                            });
                    });
            }

            if let Some(id) = to_install {
                state.install_extension(id);
                ui.request_redraw();
            } else if let Some(id) = to_uninstall {
                state.uninstall_extension(id);
                ui.request_redraw();
            }
        });
}

fn draw_language_servers_tab(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    colors: ThemeColors,
    card_w: f32,
    inner_w: f32,
) {
    let all_servers = LanguageServerCatalogItem::all();
    let ready_count = all_servers.iter().filter(|s| crate::lsp::LspManager::is_language_installed(s.language_id)).count();
    let available_count = all_servers.len() - ready_count;

    // Subheader: Filter Tabs
    ui.container()
        .full_width()
        .padding(2.0, 10.0, 6.0, 10.0)
        .row()
        .valign(Align::Center)
        .gap(4.0)
        .show(|ui| {
            let tab_all = state.extension_tab_filter == ExtensionFilterTab::All;
            let btn_all = ui.button(&format!("All ({})", all_servers.len()))
                .id(Id::from_u64(999998210))
                .height(22.0)
                .bg(if tab_all { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                .text_color(if tab_all { colors.accent } else { colors.text_muted })
                .radius_all(3.0)
                .padding_left(6.0)
                .padding_right(6.0)
                .size(10.0)
                .show();
            if btn_all.clicked {
                state.extension_tab_filter = ExtensionFilterTab::All;
                ui.request_redraw();
            }

            let tab_inst = state.extension_tab_filter == ExtensionFilterTab::Installed;
            let btn_inst = ui.button(&format!("Ready ({})", ready_count))
                .id(Id::from_u64(999998211))
                .height(22.0)
                .bg(if tab_inst { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                .text_color(if tab_inst { colors.accent } else { colors.text_muted })
                .radius_all(3.0)
                .padding_left(6.0)
                .padding_right(6.0)
                .size(10.0)
                .show();
            if btn_inst.clicked {
                state.extension_tab_filter = ExtensionFilterTab::Installed;
                ui.request_redraw();
            }

            let tab_avail = state.extension_tab_filter == ExtensionFilterTab::Available;
            let btn_avail = ui.button(&format!("Available ({})", available_count))
                .id(Id::from_u64(999998212))
                .height(22.0)
                .bg(if tab_avail { colors.accent.with_alpha(0.2) } else { Color::TRANSPARENT })
                .text_color(if tab_avail { colors.accent } else { colors.text_muted })
                .radius_all(3.0)
                .padding_left(6.0)
                .padding_right(6.0)
                .size(10.0)
                .show();
            if btn_avail.clicked {
                state.extension_tab_filter = ExtensionFilterTab::Available;
                ui.request_redraw();
            }
        });

    // Separator border
    ui.container()
        .full_width()
        .height(1.0)
        .bg(colors.border)
        .show(|_| {});

    // Scrollable List of Language Servers
    ui.container()
        .full_width()
        .fill_y()
        .scroll_y(true)
        .clip(true)
        .padding(6.0, 8.0, 6.0, 8.0)
        .gap(6.0)
        .show(|ui| {
            let search_query = &state.extension_search_query;

            let mut filtered: Vec<_> = all_servers
                .into_iter()
                .filter_map(|srv| {
                    let is_ready = crate::lsp::LspManager::is_language_installed(srv.language_id);
                    let matches_tab = match state.extension_tab_filter {
                        ExtensionFilterTab::All => true,
                        ExtensionFilterTab::Installed => is_ready,
                        ExtensionFilterTab::Available => !is_ready,
                    };
                    if !matches_tab {
                        return None;
                    }
                    let score = search_match_score(
                        search_query,
                        srv.name,
                        srv.extensions,
                        srv.server_binary,
                        srv.description,
                        &[],
                    )?;
                    Some((score, srv, is_ready))
                })
                .collect();

            // Sort by match score descending
            filtered.sort_by(|a, b| b.0.cmp(&a.0));

            if filtered.is_empty() {
                ui.container()
                    .full_width()
                    .padding(24.0, 10.0, 24.0, 10.0)
                    .valign(Align::Center)
                    .halign(Align::Center)
                    .show(|ui| {
                        ui.text("No language servers found")
                            .size(11.0)
                            .color(colors.text_muted)
                            .show();
                    });
                return;
            }

            let mut to_install_lsp = None;

            for (idx, (_, srv, is_installed)) in filtered.into_iter().enumerate() {
                let is_installing = state.is_lsp_installing(srv.language_id);
                let card_id = Id::from_u64(999998500 + idx as u64);

                ui.container()
                    .id(card_id)
                    .width(card_w)
                    .clip(true)
                    .bg(colors.bg_sidebar)
                    .hover_bg(colors.highlight)
                    .border(colors.border, 1.0)
                    .radius_all(6.0)
                    .padding(8.0, 8.0, 8.0, 8.0)
                    .column()
                    .gap(5.0)
                    .show(|ui| {
                        // Top row: Lambda Icon + Name on Left, Action Button on Right
                        ui.container()
                            .width(inner_w)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                // Left: Lambda glyph box + Name
                                ui.container()
                                    .row()
                                    .valign(Align::Center)
                                    .gap(6.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(22.0)
                                            .height(22.0)
                                            .bg(colors.accent.with_alpha(0.18))
                                            .radius_all(4.0)
                                            .valign(Align::Center)
                                            .halign(Align::Center)
                                            .show(|ui| {
                                                ui.text("\u{f0627}")
                                                    .size(13.0)
                                                    .color(colors.accent)
                                                    .show();
                                            });

                                        ui.text(srv.name)
                                            .size(12.0)
                                            .weight(FontWeight::Bold)
                                            .color(colors.text_primary)
                                            .show();
                                    });

                                // Right: Action button
                                if is_installing {
                                    let _ = ui.button("⠋ Installing...")
                                        .id(Id::from_u64(999998600 + idx as u64))
                                        .height(20.0)
                                        .bg(colors.accent.with_alpha(0.2))
                                        .text_color(colors.accent)
                                        .radius_all(3.0)
                                        .padding_left(6.0)
                                        .padding_right(6.0)
                                        .size(10.0)
                                        .show();
                                } else if is_installed {
                                    let _ = ui.button("✓ Ready")
                                        .id(Id::from_u64(999998600 + idx as u64))
                                        .height(20.0)
                                        .bg(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0).with_alpha(0.15))
                                        .text_color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                                        .radius_all(3.0)
                                        .padding_left(6.0)
                                        .padding_right(6.0)
                                        .size(10.0)
                                        .show();
                                } else {
                                    let inst_btn = ui.button("\u{f0627} Install LSP")
                                        .id(Id::from_u64(999998600 + idx as u64))
                                        .height(20.0)
                                        .bg(colors.accent)
                                        .hover_bg(colors.accent.with_alpha(0.85))
                                        .text_color(Color::WHITE)
                                        .radius_all(3.0)
                                        .padding_left(8.0)
                                        .padding_right(8.0)
                                        .size(10.0)
                                        .show();

                                    if inst_btn.clicked {
                                        to_install_lsp = Some(srv.language_id);
                                    }
                                }
                            });

                        // Row 2: Description
                        ui.text(srv.description)
                            .size(10.0)
                            .color(colors.text_muted)
                            .max_width(inner_w)
                            .wrap(true)
                            .show();

                        // Row 3: Meta (binary + extensions)
                        ui.container()
                            .width(inner_w)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.container()
                                    .row()
                                    .gap(4.0)
                                    .show(|ui| {
                                        // Binary name badge
                                        ui.container()
                                            .bg(colors.bg_editor)
                                            .radius_all(2.0)
                                            .padding(1.0, 4.0, 1.0, 4.0)
                                            .show(|ui| {
                                                ui.text(srv.server_binary)
                                                    .size(9.0)
                                                    .color(colors.accent)
                                                    .show();
                                            });

                                        for ext_tag in srv.extensions.iter().take(2) {
                                            ui.container()
                                                .bg(colors.bg_editor)
                                                .radius_all(2.0)
                                                .padding(1.0, 4.0, 1.0, 4.0)
                                                .show(|ui| {
                                                    ui.text(&format!(".{}", ext_tag))
                                                        .size(9.0)
                                                        .color(colors.text_dim)
                                                        .show();
                                                });
                                        }
                                    });

                                if is_installing {
                                    ui.text("Installing...")
                                        .size(9.5)
                                        .color(colors.accent)
                                        .show();
                                } else if is_installed {
                                    ui.text("✓ Ready")
                                        .size(9.5)
                                        .color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                                        .show();
                                } else {
                                    ui.text("Not installed")
                                        .size(9.5)
                                        .color(colors.text_dim)
                                        .show();
                                }
                            });
                    });
            }

            if let Some(lang_id) = to_install_lsp {
                state.start_lsp_install(lang_id);
                ui.request_redraw();
            }
        });
}

/// Computes a relevance score for extension and LSP search queries.
/// Returns `None` if the item does not match the search query.
/// Prioritizes exact extension match, name prefix, and avoids false positives
/// on short queries (e.g. "ex" matching "context" or "syntax").
fn search_match_score(
    query: &str,
    name: &str,
    extensions: &[&str],
    binary_or_id: &str,
    description: &str,
    features: &[&str],
) -> Option<usize> {
    let q = query.trim().to_lowercase();
    if q.is_empty() {
        return Some(10);
    }

    // 1. Exact extension match gets highest priority (e.g. query "ex" matching ".ex")
    for ext in extensions {
        let ext_clean = ext.trim_start_matches('.').to_lowercase();
        if ext_clean == q {
            return Some(100);
        }
    }

    // 2. Extension prefix match (e.g. query "ex" matching ".exs")
    for ext in extensions {
        let ext_clean = ext.trim_start_matches('.').to_lowercase();
        if ext_clean.starts_with(&q) {
            return Some(90);
        }
    }

    let name_lower = name.to_lowercase();
    // 3. Name starts with query (e.g. "Elixir")
    if name_lower.starts_with(&q) {
        return Some(85);
    }

    let id_lower = binary_or_id.to_lowercase();
    // 4. Binary/ID starts with query (e.g. "elixir-ls")
    if id_lower.starts_with(&q) {
        return Some(80);
    }

    // 5. Name contains query
    if name_lower.contains(&q) {
        return Some(70);
    }

    // 6. Binary/ID contains query
    if id_lower.contains(&q) {
        return Some(60);
    }

    // 7. Any extension contains query
    for ext in extensions {
        let ext_clean = ext.trim_start_matches('.').to_lowercase();
        if ext_clean.contains(&q) {
            return Some(50);
        }
    }

    // 8. Description or features match:
    // For short queries (<= 3 chars), do NOT match arbitrary substrings in words
    // (e.g. "ex" shouldn't match "context", "index", "syntax", "complex").
    // Only match if a word begins with the query.
    let desc_lower = description.to_lowercase();
    if q.len() <= 3 {
        let matches_word_prefix = desc_lower
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word.starts_with(&q))
            || features.iter().any(|f| {
                f.to_lowercase()
                    .split(|c: char| !c.is_alphanumeric())
                    .any(|word| word.starts_with(&q))
            });

        if matches_word_prefix {
            return Some(30);
        }
    } else {
        if desc_lower.contains(&q) || features.iter().any(|f| f.to_lowercase().contains(&q)) {
            return Some(25);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_match_score_prioritizes_extension_and_eliminates_substring_noise() {
        // Query "ex"
        let elixir_score = search_match_score(
            "ex",
            "Elixir",
            &["ex", "exs"],
            "elixir",
            "Elixir language support with Tree-sitter syntax highlighting",
            &["Syntax Highlighting"],
        );
        assert_eq!(elixir_score, Some(100)); // Exact extension match

        // Rust has word "context" in description. "context" does not start with "ex".
        let rust_score = search_match_score(
            "ex",
            "Rust",
            &["rs"],
            "rust",
            "Full Rust support with context-aware syntax highlighting and cargo integration",
            &["Syntax Highlighting", "Type Inference"],
        );
        assert_eq!(rust_score, None); // Filtered out!

        // Python has word "context" in description.
        let python_score = search_match_score(
            "ex",
            "Python",
            &["py", "pyi"],
            "python",
            "Python language pack with context definitions and indentation",
            &["Syntax Highlighting"],
        );
        assert_eq!(python_score, None); // Filtered out!

        // Shell has word "execution" in description. "execution" starts with "ex".
        let shell_score = search_match_score(
            "ex",
            "Shell Script",
            &["sh", "bash", "zsh"],
            "bash",
            "Shell script highlighting and execution helper",
            &["Syntax Highlighting"],
        );
        assert_eq!(shell_score, Some(30)); // Word prefix match, but much lower than Elixir (100)

        // Elixir ranks significantly higher than Shell
        assert!(elixir_score.unwrap() > shell_score.unwrap());
    }

    #[test]
    fn test_search_match_score_empty_query() {
        assert_eq!(
            search_match_score("", "Rust", &["rs"], "rust", "Rust support", &[]),
            Some(10)
        );
    }
}
