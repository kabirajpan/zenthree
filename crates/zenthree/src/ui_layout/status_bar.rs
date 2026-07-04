use crate::state::ZenthreeState;
use zenthra::{Ui, Align};

pub fn draw_status_bar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();

    ui.container()
        .full_width()
        .height(24.0)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .row()
        .valign(Align::Center)
        .padding_left(12.0)
        .padding_right(12.0)
        .show(|ui| {
            // Left: status message
            ui.text(&state.status_message)
                .size(11.0)
                .color(colors.text_muted)
                .show();

            // Push right-aligned content
            let right_w = 200.0;
            let drag_w = (ui.available_width - right_w).max(0.0);
            ui.container()
                .width(drag_w)
                .show(|_| {});

            // Right: cursor and buffer details
            ui.container()
                .width(right_w)
                .row()
                .valign(Align::Center)
                .halign(Align::Right)
                .gap(16.0)
                .show(|ui| {
                    if let Some(active_idx) = state.active_tab {
                        if active_idx < state.tabs.len() {
                            let tab = &state.tabs[active_idx];
                            let cursor = tab.editor.cursor();

                            ui.text(&format!("Ln {}, Col {}", cursor.row + 1, cursor.column + 1))
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();

                            let line_count = tab.editor.buffer.line_count().max(1);
                            ui.text(&format!("{} lines", line_count))
                                .size(11.0)
                                .color(colors.text_muted)
                                .show();
                        }
                    }
                    ui.text("UTF-8")
                        .size(11.0)
                        .color(colors.text_muted)
                        .show();
                });
        });
}
