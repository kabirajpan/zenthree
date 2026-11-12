use zenthra::{Align, Color, FontWeight, Id, Ui, Style};
use crate::theme::ThemeColors;
use super::CompletionState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionUiEvent {
    None,
    Commit(usize),
    Dismiss,
}

/// Renders the floating LazyVim-style completion popup and attached documentation flyout.
pub fn draw_completion_popup(
    ui: &mut Ui,
    completion: &CompletionState,
    colors: &ThemeColors,
    cursor_screen_pos: (f32, f32),
) -> CompletionUiEvent {
    if !completion.active || completion.items.is_empty() {
        return CompletionUiEvent::None;
    }

    let (cur_x, cur_y) = cursor_screen_pos;
    let popup_w = 310.0f32;
    let item_h = 23.0f32;
    let max_visible = 9;
    let visible_count = completion.items.len().min(max_visible);
    let popup_h = (visible_count as f32 * item_h + 8.0).clamp(32.0, 230.0);

    // Keep popup horizontally within the window boundaries
    let mut px = cur_x;
    if px + popup_w > ui.width - 12.0 {
        px = (ui.width - popup_w - 12.0).max(12.0);
    }

    // Flip above cursor if not enough room beneath it
    let mut py = cur_y + 4.0;
    if py + popup_h > ui.height - 30.0 {
        py = (cur_y - popup_h - 22.0).max(10.0);
    }

    let selected_idx = completion.selected_idx.min(completion.items.len().saturating_sub(1));
    let selected_item = completion.items.get(selected_idx);

    // Zenthree Smart Sub-Box Gate:
    // Never show redundant flyouts for simple keywords or buffer words.
    // The sub-box ONLY appears if there is genuine, valuable content:
    // 1. A code snippet expansion / template (e.g. `class~`, `main~`, `sout~`)
    // 2. OR authentic rich documentation from LSP/Javadoc/Rustdoc
    let (has_doc_flyout, code_preview, doc_text) = if let Some(item) = selected_item {
        // Plain keywords or buffer words never trigger a redundant sub-box
        let is_plain_keyword = item.kind == super::CompletionKind::Keyword;
        let is_plain_word = item.kind == super::CompletionKind::BufferWord;

        let snippet_template = item.postfix_template.as_deref().filter(|c| !c.trim().is_empty());
        let insert_code = item.insert_text.as_deref().filter(|c| {
            *c != item.label && !c.trim().is_empty() && (c.contains('\n') || c.contains('$') || c.contains('~'))
        });
        let code = snippet_template.or(insert_code);

        let doc = if is_plain_keyword || is_plain_word {
            None
        } else {
            item.documentation.as_deref().filter(|d| {
                let t = d.trim();
                !t.is_empty()
                    && t != "buffer"
                    && t != "Text identifier from active buffer"
                    && t != item.label
                    && !t.starts_with("keyword ")
                    && !t.starts_with("keyword\n")
                    && (t.contains('\n') || t.len() >= 24)
            })
        };

        let is_snippet = item.is_postfix || item.kind == super::CompletionKind::PostfixSnippet || code.is_some();
        let has_content = (is_snippet && code.is_some()) || doc.is_some();

        if has_content {
            (true, code, doc)
        } else {
            (false, None, None)
        }
    } else {
        (false, None, None)
    };

    let doc_w = 320.0f32;
    let show_doc_on_right = px + popup_w + doc_w + 10.0 <= ui.width;
    let doc_x = if show_doc_on_right {
        px + popup_w + 6.0
    } else {
        (px - doc_w - 6.0).max(6.0)
    };

    let total_box_x1 = if has_doc_flyout { px.min(doc_x) } else { px };
    let total_box_x2 = if has_doc_flyout { (px + popup_w).max(doc_x + doc_w) } else { px + popup_w };
    let is_mouse_inside = ui.mouse_x >= total_box_x1
        && ui.mouse_x <= total_box_x2
        && ui.mouse_y >= py
        && ui.mouse_y <= py + popup_h + 40.0;

    let mut event = CompletionUiEvent::None;

    // Dismiss when clicking outside
    if ui.clicked && !is_mouse_inside {
        return CompletionUiEvent::Dismiss;
    }

    let popup_id = Id::from_u64(0xCAD_0000_1001);

    ui.overlay(|ui| {
        // ─── 1. Left Menu Box (Modern Zed / LazyVim style) ────────────────
        ui.container()
            .raw_id(popup_id)
            .pos(px, py)
            .overlay()
            .width(popup_w)
            .height(popup_h)
            .scroll_y(true)
            .bg(colors.bg_sidebar)
            .border((colors.border.with_alpha(0.65), 1.0))
            .radius_all(6.0)
            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.28), 0.0, 4.0, 10.0)
            .padding(3.0, 5.0, 3.0, 3.0)
            .column()
            .gap(1.5)
            .show(|ui| {
                for (idx, item) in completion.items.iter().enumerate() {
                    let is_selected = idx == selected_idx;
                    let item_id = Id::from_u64(0xCAD_0000_2000 + idx as u64);
                    let (kind_icon, kind_color) = item.kind.icon_and_color();

                    // Detail or kind tag
                    let tag_str = item.detail.as_deref().unwrap_or_else(|| match item.kind {
                        super::CompletionKind::Keyword => "keyword",
                        super::CompletionKind::BufferWord => "buffer",
                        super::CompletionKind::PostfixSnippet => "snippet",
                        super::CompletionKind::Method | super::CompletionKind::Function => "fn",
                        super::CompletionKind::Variable => "var",
                        super::CompletionKind::Struct => "type",
                        _ => "",
                    });

                    let has_tag = !tag_str.is_empty();
                    let max_tag_len = 16;
                    let display_tag = if tag_str.chars().count() > max_tag_len {
                        let trunc: String = tag_str.chars().take(max_tag_len.saturating_sub(1)).collect();
                        format!("{}…", trunc)
                    } else {
                        tag_str.to_string()
                    };

                    // Safe label length with clean ellipsis so text never touches or clips borders
                    let max_label_len = if has_tag { 22 } else { 32 };
                    let display_label = if item.label.chars().count() > max_label_len {
                        let trunc: String = item.label.chars().take(max_label_len.saturating_sub(1)).collect();
                        format!("{}…", trunc)
                    } else {
                        item.label.clone()
                    };

                    let mut row = ui.container()
                        .id(item_id)
                        .fill_x()
                        .height(item_h)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .padding_left(8.0)
                        .padding_right(10.0)
                        .radius_all(4.0);

                    if is_selected {
                        // Zenthree native minimalist theme-reactive selection
                        row = row.bg(colors.accent.with_alpha(0.20))
                            .border((colors.accent.with_alpha(0.40), 1.0))
                            .hover(Style::bg(colors.accent.with_alpha(0.25)));
                    } else {
                        row = row.bg(Color::TRANSPARENT)
                            .hover(Style::bg(colors.accent.with_alpha(0.10)));
                    }

                    let resp = row.show(|ui| {
                        // Left: Icon + Label
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(7.0)
                            .show(|ui| {
                                ui.text(kind_icon)
                                    .size(11.0)
                                    .font_weight(FontWeight::Bold)
                                    .color(kind_color)
                                    .show();

                                ui.text(&display_label)
                                    .size(11.5)
                                    .font_weight(if is_selected { FontWeight::Bold } else { FontWeight::Regular })
                                    .color(colors.text_primary)
                                    .show();
                            });

                        // Right: Kind or Detail tag
                        if has_tag {
                            ui.text(&display_tag)
                                .size(9.5)
                                .color(if is_selected { colors.accent } else { colors.text_dim })
                                .show();
                        }
                    });

                    if resp.clicked {
                        event = CompletionUiEvent::Commit(idx);
                    }
                }
            });

        // ─── 2. Separate Floating Preview & Documentation Sub-Box (Zenthree Minimal) ───────
        if has_doc_flyout {
            if let Some(item) = selected_item {
                let doc_popup_id = Id::from_u64(0xCAD_0000_3001);
                let (kind_icon, kind_color) = item.kind.icon_and_color();

                ui.container()
                    .raw_id(doc_popup_id)
                    .pos(doc_x, py)
                    .overlay()
                    .width(doc_w)
                    .max_height(280.0)
                    .scroll_y(true)
                    .bg(colors.bg_sidebar)
                    .border((colors.border.with_alpha(0.65), 1.0))
                    .radius_all(6.0)
                    .shadow(Color::rgba(0.0, 0.0, 0.0, 0.28), 0.0, 4.0, 10.0)
                    .padding(8.0, 10.0, 8.0, 10.0)
                    .column()
                    .gap(6.0)
                    .show(|ui| {
                        // Header bar: Kind icon + item label
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text(kind_icon)
                                    .size(11.0)
                                    .font_weight(FontWeight::Bold)
                                    .color(kind_color)
                                    .show();

                                ui.text(&item.label)
                                    .size(11.5)
                                    .font_weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            });

                        // Top Zone: Snippet Expansion or Code Preview
                        if let Some(code) = code_preview {
                            ui.container()
                                .fill_x()
                                .bg(colors.bg_base)
                                .border((colors.border.with_alpha(0.40), 1.0))
                                .radius_all(4.0)
                                .padding(6.0, 8.0, 6.0, 8.0)
                                .column()
                                .gap(2.0)
                                .show(|ui| {
                                    for line in code.lines().take(7) {
                                        ui.text(line)
                                            .size(11.0)
                                            .font_weight(FontWeight::Medium)
                                            .color(Color::rgb(254.0 / 255.0, 138.0 / 255.0, 36.0 / 255.0))
                                            .show();
                                    }
                                });

                            if doc_text.is_some() {
                                ui.container()
                                    .fill_x()
                                    .height(1.0)
                                    .bg(colors.border.with_alpha(0.30))
                                    .show(|_| {});
                            }
                        }

                        // Bottom Zone: Real Documentation
                        if let Some(doc) = doc_text {
                            ui.container()
                                .fill_x()
                                .column()
                                .gap(3.0)
                                .show(|ui| {
                                    for line in doc.lines().take(8) {
                                        let is_import_line = line.starts_with("󰏪")
                                            || line.starts_with("from ")
                                            || line.starts_with("import ")
                                            || line.starts_with("use ");
                                        ui.text(line)
                                            .size(10.5)
                                            .color(if is_import_line {
                                                Color::rgb(130.0 / 255.0, 200.0 / 255.0, 255.0 / 255.0)
                                            } else {
                                                colors.text_muted
                                            })
                                            .show();
                                    }
                                });
                        }
                    });
            }
        }
    });

    event
}
