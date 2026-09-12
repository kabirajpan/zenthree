use crate::state::{ZenthreeState, ExtensionFilterTab};
use crate::extensions::ExtensionCatalog;
use zenthra::{Color, Ui, Align, FontWeight, Id};

pub fn draw_extensions_panel(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();
    let all_extensions = ExtensionCatalog::all();

    let installed_count = all_extensions.iter().filter(|e| state.is_extension_installed(e.id)).count();
    let available_count = all_extensions.len() - installed_count;

    // Outer column layout
    ui.container()
        .full_width()
        .fill_y()
        .column()
        .show(|ui| {
            // 1. Search Box
            ui.container()
                .full_width()
                .padding(8.0, 10.0, 6.0, 10.0)
                .column()
                .gap(6.0)
                .show(|ui| {
                    ui.container()
                        .full_width()
                        .height(28.0)
                        .bg(colors.bg_editor)
                        .border(colors.border, 1.0)
                        .radius_all(4.0)
                        .padding_left(8.0)
                        .padding_right(8.0)
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text("🔍")
                                .size(11.0)
                                .show();

                            ui.input(&mut state.extension_search_query, Id::from_u64(999998200))
                                .fill_x()
                                .color(colors.text_primary)
                                .size(11.0)
                                .show();
                        });

                    // 2. Filter Tabs (All / Installed / Available)
                    ui.container()
                        .full_width()
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
                });

            // Separator border
            ui.container()
                .full_width()
                .height(1.0)
                .bg(colors.border)
                .show(|_| {});

            // 3. Extension Cards Scrollable List
            ui.container()
                .full_width()
                .fill_y()
                .scroll_y(true)
                .padding(4.0, 6.0, 4.0, 6.0)
                .gap(6.0)
                .show(|ui| {
                    let search_lower = state.extension_search_query.to_lowercase();

                    let filtered: Vec<_> = all_extensions
                        .into_iter()
                        .filter(|ext| {
                            // Filter by tab
                            let is_inst = state.is_extension_installed(ext.id);
                            match state.extension_tab_filter {
                                ExtensionFilterTab::All => true,
                                ExtensionFilterTab::Installed => is_inst,
                                ExtensionFilterTab::Available => !is_inst,
                            }
                        })
                        .filter(|ext| {
                            // Filter by search
                            if search_lower.is_empty() {
                                true
                            } else {
                                ext.name.to_lowercase().contains(&search_lower)
                                    || ext.description.to_lowercase().contains(&search_lower)
                                    || ext.extensions.iter().any(|e| e.contains(&search_lower))
                                    || ext.features.iter().any(|f| f.to_lowercase().contains(&search_lower))
                            }
                        })
                        .collect();

                    if filtered.is_empty() {
                        ui.container()
                            .full_width()
                            .padding(20.0, 10.0, 20.0, 10.0)
                            .valign(Align::Center)
                            .halign(Align::Center)
                            .show(|ui| {
                                ui.text("No language extensions found")
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
                            .full_width()
                            .bg(colors.bg_sidebar)
                            .hover_bg(colors.highlight)
                            .border(colors.border, 1.0)
                            .radius_all(6.0)
                            .padding(8.0, 8.0, 8.0, 8.0)
                            .column()
                            .gap(4.0)
                            .show(|ui| {
                                // Top row: Icon + Name + Version + Action
                                ui.container()
                                    .full_width()
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
                                                // Icon glyph box
                                                ui.container()
                                                    .width(20.0)
                                                    .height(20.0)
                                                    .bg(ext.icon_color.with_alpha(0.18))
                                                    .radius_all(3.0)
                                                    .valign(Align::Center)
                                                    .halign(Align::Center)
                                                    .show(|ui| {
                                                        ui.text(ext.glyph)
                                                            .size(11.0)
                                                            .color(ext.icon_color)
                                                            .show();
                                                    });

                                                ui.text(ext.name)
                                                    .size(11.5)
                                                    .weight(FontWeight::Bold)
                                                    .color(colors.text_primary)
                                                    .show();

                                                ui.text(&format!("v{}", ext.version))
                                                    .size(9.5)
                                                    .color(colors.text_muted.with_alpha(0.6))
                                                    .show();
                                            });

                                        // Right: Action button
                                        if is_installed {
                                            let uninst_btn = ui.button("Uninstall")
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

                                // Description
                                ui.text(ext.description)
                                    .size(10.0)
                                    .color(colors.text_muted)
                                    .show();

                                // Tags / Extensions row
                                ui.container()
                                    .full_width()
                                    .row()
                                    .valign(Align::Center)
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

                                        if is_installed {
                                            ui.container().fill_x().show(|_| {});
                                            ui.text("✓ Installed")
                                                .size(9.5)
                                                .color(Color::rgb(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0))
                                                .show();
                                        }
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
        });
}
