use crate::state::ZenthreeState;
use crate::theme::ThemeColors;
use crate::ui_layout::ai_panel::{
    draw_inline_spans, parse_markdown_blocks_impl, MarkdownBlock, QuoteKind,
};
use zenthra::{Align, Color, FontWeight, Id, Ui};

pub fn draw_markdown_buffer_preview(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    tab_idx: usize,
    width: f32,
    height: f32,
    colors: &ThemeColors,
) {
    if tab_idx >= state.tabs.len() {
        return;
    }

    let content = if !state.tabs[tab_idx].is_dirty {
        state.tabs[tab_idx]
            .path
            .as_ref()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .unwrap_or_else(|| state.tabs[tab_idx].editor.buffer.to_string())
    } else {
        state.tabs[tab_idx].editor.buffer.to_string()
    };
    let blocks = parse_markdown_blocks_impl(&content);

    let editor_x = ui.cursor_x;
    let editor_y = ui.cursor_y;
    let editor_w = width;
    let editor_h = height;

    // Handle scroll wheel
    let is_hovered = ui.mouse_in_rect(editor_x + ui.offset_x, editor_y + ui.offset_y, editor_w, editor_h);
    if is_hovered {
        let mut scroll_delta_y = 0.0;
        ui.input_events.retain(|event| {
            if let zenthra_platform::event::PlatformEvent::MouseWheel { delta_y, .. } = event {
                scroll_delta_y += *delta_y;
                false
            } else {
                true
            }
        });
        if scroll_delta_y.abs() > 0.001 {
            let tab = &mut state.tabs[tab_idx];
            tab.scroll_y = (tab.scroll_y - scroll_delta_y * 36.0).max(0.0);
            ui.needs_redraw = true;
        }
    }

    let scroll_y = state.tabs[tab_idx].scroll_y;

    // Outer container clipped to editor area
    ui.container()
        .width(editor_w)
        .height(editor_h)
        .bg(colors.bg_editor)
        .clip(true)
        .show(|ui| {
            let doc_max_w = (editor_w - 80.0).clamp(320.0, 920.0);
            let side_padding = ((editor_w - doc_max_w) / 2.0).max(28.0);

            // Apply scroll offset
            let start_y = ui.cursor_y - scroll_y;
            ui.cursor_y = start_y;

            let mut copy_code_clicked: Option<String> = None;

            ui.container()
                .padding_left(side_padding)
                .padding_right(side_padding)
                .padding_top(28.0)
                .padding_bottom(64.0)
                .max_width(editor_w)
                .column()
                .gap(12.0)
                .show(|ui| {
                    for (block_idx, block) in blocks.iter().enumerate() {
                        match block {
                            MarkdownBlock::Heading { level, spans } => {
                                let (font_size, pad_t, pad_b, is_h1_h2) = match level {
                                    1 => (20.0_f32, 16.0_f32, 6.0_f32, true),
                                    2 => (16.0, 14.0, 5.0, true),
                                    3 => (13.5, 10.0, 4.0, false),
                                    4 => (12.0, 8.0, 3.0, false),
                                    _ => (11.0, 6.0, 2.0, false),
                                };

                                ui.container()
                                    .padding_top(pad_t)
                                    .padding_bottom(pad_b)
                                    .column()
                                    .gap(4.0)
                                    .show(|ui| {
                                        draw_inline_spans(
                                            ui,
                                            spans,
                                            doc_max_w,
                                            font_size,
                                            FontWeight::Bold,
                                            if *level == 1 { colors.accent } else { colors.text_primary },
                                            colors,
                                        );
                                        if is_h1_h2 {
                                            ui.container()
                                                .full_width()
                                                .height(1.0)
                                                .bg(colors.border.with_alpha(0.35))
                                                .show(|_| {});
                                        }
                                    });
                            }
                            MarkdownBlock::Rule => {
                                ui.container()
                                    .full_width()
                                    .height(1.0)
                                    .bg(colors.border.with_alpha(0.4))
                                    .padding_top(8.0)
                                    .padding_bottom(8.0)
                                    .show(|_| {});
                            }
                            MarkdownBlock::BlockQuote { kind, spans } => {
                                let (accent_col, badge_label, badge_icon) = match kind {
                                    QuoteKind::Note => (Color::rgb(59.0 / 255.0, 130.0 / 255.0, 246.0 / 255.0), "NOTE", "\u{f05a}"),
                                    QuoteKind::Tip => (Color::rgb(34.0 / 255.0, 197.0 / 255.0, 94.0 / 255.0), "TIP", "\u{f0eb}"),
                                    QuoteKind::Important => (Color::rgb(139.0 / 255.0, 92.0 / 255.0, 246.0 / 255.0), "IMPORTANT", "\u{f06a}"),
                                    QuoteKind::Warning => (Color::rgb(245.0 / 255.0, 158.0 / 255.0, 11.0 / 255.0), "WARNING", "\u{f071}"),
                                    QuoteKind::Caution => (Color::rgb(239.0 / 255.0, 68.0 / 255.0, 68.0 / 255.0), "CAUTION", "\u{f057}"),
                                    QuoteKind::Plain => (colors.accent, "", "\u{f10d}"),
                                };
                                let is_keyed = !badge_label.is_empty();

                                ui.container()
                                    .max_width(doc_max_w)
                                    .row()
                                    .gap(0.0)
                                    .show(|ui| {
                                        ui.container()
                                            .width(3.5)
                                            .bg(accent_col)
                                            .radius(0.0, 0.0, 2.0, 2.0)
                                            .show(|_| {});

                                        ui.container()
                                            .bg(accent_col.with_alpha(0.08))
                                            .border(accent_col.with_alpha(0.2), 0.5)
                                            .radius(4.0, 0.0, 0.0, 4.0)
                                            .padding(8.0, 14.0, 8.0, 14.0)
                                            .column()
                                            .gap(5.0)
                                            .show(|ui| {
                                                if is_keyed {
                                                    ui.container()
                                                        .row()
                                                        .valign(Align::Center)
                                                        .gap(6.0)
                                                        .show(|ui| {
                                                            ui.text(badge_icon).size(10.0).color(accent_col).show();
                                                            ui.text(badge_label)
                                                                .size(10.0)
                                                                .weight(FontWeight::Bold)
                                                                .color(accent_col)
                                                                .show();
                                                        });
                                                }
                                                draw_inline_spans(
                                                    ui,
                                                    spans,
                                                    doc_max_w - 30.0,
                                                    11.5,
                                                    FontWeight::Regular,
                                                    colors.text_primary,
                                                    colors,
                                                );
                                            });
                                    });
                            }
                            MarkdownBlock::Table {
                                alignments: _,
                                headers,
                                rows,
                            } => {
                                let col_count = headers.len().max(1);
                                let col_w = ((doc_max_w - 2.0) / col_count as f32).max(60.0);

                                ui.container()
                                    .max_width(doc_max_w)
                                    .radius_all(6.0)
                                    .border(colors.border.with_alpha(0.5), 1.0)
                                    .clip(true)
                                    .column()
                                    .show(|ui| {
                                        // Header row
                                        ui.container()
                                            .full_width()
                                            .row()
                                            .gap(0.0)
                                            .bg(colors.bg_sidebar.with_alpha(0.9))
                                            .show(|ui| {
                                                for (ci, cell) in headers.iter().enumerate() {
                                                    let is_last = ci + 1 == col_count;
                                                    ui.container()
                                                        .width(col_w)
                                                        .padding(7.0, 10.0, 7.0, 10.0)
                                                        .show(|ui| {
                                                            draw_inline_spans(
                                                                ui,
                                                                cell,
                                                                col_w - 20.0,
                                                                11.0,
                                                                FontWeight::SemiBold,
                                                                colors.text_primary,
                                                                colors,
                                                            );
                                                        });
                                                    if !is_last {
                                                        ui.container()
                                                            .width(0.5)
                                                            .bg(colors.border.with_alpha(0.4))
                                                            .show(|_| {});
                                                    }
                                                }
                                            });

                                        ui.container().full_width().height(1.0).bg(colors.border.with_alpha(0.5)).show(|_| {});

                                        // Data rows
                                        for (ri, row) in rows.iter().enumerate() {
                                            let row_bg = if ri % 2 == 1 {
                                                colors.bg_sidebar.with_alpha(0.3)
                                            } else {
                                                Color::TRANSPARENT
                                            };

                                            ui.container()
                                                .full_width()
                                                .row()
                                                .gap(0.0)
                                                .bg(row_bg)
                                                .show(|ui| {
                                                    for (ci, cell) in row.iter().enumerate() {
                                                        let is_last = ci + 1 == col_count;
                                                        ui.container()
                                                            .width(col_w)
                                                            .padding(6.0, 10.0, 6.0, 10.0)
                                                            .show(|ui| {
                                                                draw_inline_spans(
                                                                    ui,
                                                                    cell,
                                                                    col_w - 20.0,
                                                                    11.0,
                                                                    FontWeight::Regular,
                                                                    colors.text_primary,
                                                                    colors,
                                                                );
                                                            });
                                                        if !is_last {
                                                            ui.container()
                                                                .width(0.5)
                                                                .bg(colors.border.with_alpha(0.3))
                                                                .show(|_| {});
                                                        }
                                                    }
                                                });

                                            if ri + 1 < rows.len() {
                                                ui.container().full_width().height(0.5).bg(colors.border.with_alpha(0.25)).show(|_| {});
                                            }
                                        }
                                    });
                            }
                            MarkdownBlock::BulletItem { depth, spans, checked } => {
                                let indent = (*depth as f32) * 16.0 + 4.0;
                                let avail_w = (doc_max_w - indent - 20.0).max(40.0);

                                ui.container()
                                    .full_width()
                                    .row()
                                    .valign(Align::Top)
                                    .gap(8.0)
                                    .padding_left(indent)
                                    .show(|ui| {
                                        match checked {
                                            Some(true) => {
                                                ui.text("\u{f14a}").size(12.0).color(colors.accent).show();
                                            }
                                            Some(false) => {
                                                ui.text("\u{f0c8}").size(12.0).color(colors.text_muted).show();
                                            }
                                            None => {
                                                let bullet = if *depth == 0 { "•" } else { "◦" };
                                                ui.text(bullet).size(10.0).color(colors.accent).show();
                                            }
                                        }
                                        draw_inline_spans(ui, spans, avail_w, 12.0, FontWeight::Regular, colors.text_primary, colors);
                                    });
                            }
                            MarkdownBlock::NumberedItem { num, depth, spans } => {
                                let indent = (*depth as f32) * 16.0 + 4.0;
                                let avail_w = (doc_max_w - indent - 24.0).max(40.0);

                                ui.container()
                                    .full_width()
                                    .row()
                                    .valign(Align::Top)
                                    .gap(8.0)
                                    .padding_left(indent)
                                    .show(|ui| {
                                        ui.text(&format!("{}.", num))
                                            .size(11.5)
                                            .weight(FontWeight::Medium)
                                            .color(colors.accent)
                                            .show();
                                        draw_inline_spans(ui, spans, avail_w, 12.0, FontWeight::Regular, colors.text_primary, colors);
                                    });
                            }
                            MarkdownBlock::CodeBlock { lang, code } => {
                                let code_id = Id::from_u64(999_800_000 + (tab_idx as u64) * 1000 + (block_idx as u64));
                                let lang_display = lang.as_deref().unwrap_or("code").to_uppercase();

                                ui.container()
                                    .max_width(doc_max_w)
                                    .radius_all(6.0)
                                    .bg(colors.bg_base.with_alpha(0.85))
                                    .border(colors.border.with_alpha(0.5), 1.0)
                                    .column()
                                    .show(|ui| {
                                        ui.container()
                                            .full_width()
                                            .height(26.0)
                                            .bg(colors.bg_sidebar.with_alpha(0.6))
                                            .padding(0.0, 10.0, 0.0, 10.0)
                                            .row()
                                            .halign(Align::SpaceBetween)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text(&lang_display)
                                                    .size(9.0)
                                                    .weight(FontWeight::Bold)
                                                    .monospace()
                                                    .color(colors.accent)
                                                    .show();

                                                let copy_resp = ui.container()
                                                    .id(code_id)
                                                    .row()
                                                    .valign(Align::Center)
                                                    .gap(4.0)
                                                    .padding(2.0, 6.0, 2.0, 6.0)
                                                    .radius_all(3.0)
                                                    .hover_bg(colors.highlight)
                                                    .show(|ui| {
                                                        ui.text("\u{f0c5}").size(9.0).color(colors.text_muted).show();
                                                        ui.text("Copy").size(9.5).color(colors.text_muted).show();
                                                    });

                                                if copy_resp.clicked {
                                                    copy_code_clicked = Some(code.clone());
                                                }
                                            });

                                        ui.container().full_width().height(0.5).bg(colors.border.with_alpha(0.3)).show(|_| {});

                                        ui.container()
                                            .padding(10.0, 12.0, 10.0, 12.0)
                                            .show(|ui| {
                                                ui.text(code)
                                                    .size(11.0)
                                                    .monospace()
                                                    .line_height(1.5)
                                                    .color(colors.text_primary)
                                                    .show();
                                            });
                                    });
                            }
                            MarkdownBlock::Paragraph(spans) => {
                                ui.container()
                                    .max_width(doc_max_w)
                                    .show(|ui| {
                                        draw_inline_spans(
                                            ui,
                                            spans,
                                            doc_max_w,
                                            12.0,
                                            FontWeight::Regular,
                                            colors.text_primary,
                                            colors,
                                        );
                                    });
                            }
                        }
                    }
                });

            if let Some(code) = copy_code_clicked {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    let _ = cb.set_text(code);
                }
                state.set_status("Code snippet copied to clipboard");
                ui.needs_redraw = true;
            }
        });
}
