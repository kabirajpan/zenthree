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
        .halign(Align::Right)
        .padding_left(12.0)
        .padding_right(8.0)
        .show(|ui| {
            // Right: cursor and buffer details & Zenthree IDE tag
            ui.container()
                .height(24.0)
                .row()
                .valign(Align::Center)
                .gap(12.0)
                .show(|ui| {
                    if let Some(active_idx) = state.active_tab {
                        if active_idx < state.tabs.len() {
                            let tab = &state.tabs[active_idx];
                            let cursor = tab.editor.cursor();

                            // Cursor position visible if window >= 400px
                            if avail_w >= 400.0 {
                                ui.text(&format!("Ln {}, Col {}", cursor.row + 1, cursor.column + 1))
                                    .size(11.0)
                                    .color(colors.text_muted)
                                    .show();
                            }

                            // Total line count visible if window >= 650px
                            if avail_w >= 650.0 {
                                let line_count = tab.editor.buffer.line_count().max(1);
                                ui.text(&format!("{} lines", line_count))
                                    .size(11.0)
                                    .color(colors.text_muted)
                                    .show();
                            }
                        }
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
