use zenthra::{Align, Color, FontWeight, Id, Ui};
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
    let popup_w = 270.0f32;
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

    // Determine if the attached documentation flyout should be displayed:
    // It should ONLY appear if there is real, meaningful content:
    // 1. A code snippet expansion or postfix template differing from the label
    // 2. OR authentic documentation / markdown docstring
    // 3. OR a rich function / method / class signature
    let (has_doc_flyout, code_preview, doc_text) = if let Some(item) = selected_item {
        let explicit_code = item.postfix_template.as_deref().filter(|c| !c.trim().is_empty());
        let insert_code = item.insert_text.as_deref().filter(|c| *c != item.label && !c.trim().is_empty());
        let code = explicit_code.or(insert_code);

        let doc = item.documentation.as_deref().filter(|d| {
            let t = d.trim();
            !t.is_empty()
                && t != "buffer"
                && t != "Text identifier from active buffer"
                && t != item.label
        });

        // Detail with rich signature info (e.g. `(a: i32) -> bool` or `def func()`)
        let rich_sig = item.detail.as_deref().filter(|det| {
            let t = det.trim();
            !t.is_empty()
                && t != "buffer"
                && t != "keyword"
                && (t.contains('(') || t.contains("->") || t.contains(':') || t.contains('{'))
        });

        let preview_code = code.or(rich_sig);
        let has_content = preview_code.is_some() || doc.is_some();
        (has_content, preview_code, doc)
    } else {
        (false, None, None)
    };

    let doc_w = 340.0f32;
    let show_doc_on_right = px + popup_w + doc_w + 8.0 <= ui.width;
    let doc_x = if show_doc_on_right {
        px + popup_w + 4.0
    } else {
        (px - doc_w - 4.0).max(6.0)
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
            .absolute(px, py)
            .overlay()
            .width(popup_w)
            .height(popup_h)
            .scroll_y(true)
            .bg(colors.bg_sidebar)
            .border(colors.border.with_alpha(0.85), 1.0)
            .radius_all(5.0)
            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.65), 0.0, 6.0, 18.0)
            .padding(3.0, 3.0, 3.0, 3.0)
            .column()
            .gap(1.0)
            .show(|ui| {
                for (idx, item) in completion.items.iter().enumerate() {
                    let is_selected = idx == selected_idx;
                    let item_id = Id::from_u64(0xCAD_0000_2000 + idx as u64);
                    let (kind_icon, kind_color) = item.kind.icon_and_color();

                    let mut row = ui.container()
                        .id(item_id)
                        .full_width()
                        .height(item_h)
                        .row()
                        .valign(Align::Center)
                        .halign(Align::SpaceBetween)
                        .padding_left(7.0)
                        .padding_right(7.0)
                        .radius_all(3.5);

                    if is_selected {
                        // Rich accent highlight for selected item
                        row = row.bg(Color::rgba(56.0 / 255.0, 85.0 / 255.0, 72.0 / 255.0, 0.95))
                            .border(Color::rgba(74.0 / 255.0, 222.0 / 255.0, 128.0 / 255.0, 0.40), 1.0);
                    } else {
                        row = row.bg(Color::TRANSPARENT).hover_bg(colors.highlight);
                    }

                    let resp = row.show(|ui| {
                        // Left: Icon + Label
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(7.0)
                            .show(|ui| {
                                ui.text(kind_icon)
                                    .size(10.5)
                                    .weight(FontWeight::Bold)
                                    .color(kind_color)
                                    .show();

                                ui.text(&item.label)
                                    .size(11.5)
                                    .weight(if is_selected { FontWeight::Bold } else { FontWeight::Regular })
                                    .color(if is_selected { Color::WHITE } else { colors.text_primary })
                                    .show();
                            });

                        // Right: Kind or Detail tag
                        let tag_str = item.detail.as_deref().unwrap_or_else(|| match item.kind {
                            super::CompletionKind::Keyword => "keyword",
                            super::CompletionKind::BufferWord => "buffer",
                            super::CompletionKind::PostfixSnippet => "snippet",
                            super::CompletionKind::Method | super::CompletionKind::Function => "fn",
                            super::CompletionKind::Variable => "var",
                            super::CompletionKind::Struct => "type",
                            _ => "",
                        });

                        if !tag_str.is_empty() {
                            let display_tag = if tag_str.len() > 14 {
                                format!("{}…", &tag_str[..13])
                            } else {
                                tag_str.to_string()
                            };
                            ui.text(&display_tag)
                                .size(9.5)
                                .color(if is_selected { Color::rgba(1.0, 1.0, 1.0, 0.85) } else { colors.text_dim })
                                .show();
                        }
                    });

                    if resp.clicked {
                        event = CompletionUiEvent::Commit(idx);
                    }
                }
            });

        // ─── 2. Attached Right Documentation & Expansion Flyout Card ───────
        if has_doc_flyout {
            if let Some(item) = selected_item {
                let doc_popup_id = Id::from_u64(0xCAD_0000_3001);
                let (kind_icon, kind_color) = item.kind.icon_and_color();

                ui.container()
                    .raw_id(doc_popup_id)
                    .absolute(doc_x, py)
                    .overlay()
                    .width(doc_w)
                    .max_height(280.0)
                    .scroll_y(true)
                    .bg(colors.bg_sidebar)
                    .border(colors.border.with_alpha(0.85), 1.0)
                    .radius_all(6.0)
                    .shadow(Color::rgba(0.0, 0.0, 0.0, 0.65), 0.0, 6.0, 18.0)
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
                                    .weight(FontWeight::Bold)
                                    .color(kind_color)
                                    .show();

                                ui.text(&item.label)
                                    .size(11.5)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();
                            });

                        // Top Zone: Snippet Expansion, Variable Assignment, or Function Body (styled card)
                        if let Some(code) = code_preview {
                            ui.container()
                                .full_width()
                                .bg(colors.bg_base)
                                .border(colors.border.with_alpha(0.6), 1.0)
                                .radius_all(4.0)
                                .padding(6.0, 8.0, 6.0, 8.0)
                                .column()
                                .gap(2.0)
                                .show(|ui| {
                                    for line in code.lines().take(6) {
                                        ui.text(line)
                                            .size(11.0)
                                            .weight(FontWeight::Medium)
                                            .color(Color::rgb(254.0 / 255.0, 138.0 / 255.0, 36.0 / 255.0))
                                            .show();
                                    }
                                });

                            if doc_text.is_some() {
                                ui.container()
                                    .full_width()
                                    .height(1.0)
                                    .bg(colors.border.with_alpha(0.40))
                                    .show(|_| {});
                            }
                        }

                        // Bottom Zone: Origin, Documentation, or Auto-Import Helper
                        if let Some(doc) = doc_text {
                            ui.container()
                                .full_width()
                                .column()
                                .gap(3.0)
                                .show(|ui| {
                                    for line in doc.lines().take(8) {
                                        let is_import_line = line.starts_with("󰏪")
                                            || line.starts_with("from ")
                                            || line.starts_with("import ")
                                            || line.starts_with("use ");
                                        ui.text(line)
                                            .size(10.0)
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
