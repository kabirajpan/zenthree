pub mod title_bar;
pub mod sidebar;
pub mod main_buffer;
pub mod status_bar;
pub mod terminal;
pub mod ai_panel;
pub mod language_dialog;
pub mod extensions_panel;

pub use title_bar::draw_title_bar;
pub use sidebar::draw_sidebar;
pub use main_buffer::draw_main_buffer;
pub use status_bar::draw_status_bar;
pub use terminal::draw_terminal_panel;
pub use ai_panel::draw_ai_panel;
pub use language_dialog::draw_language_prompt_dialog;
pub use extensions_panel::draw_extensions_panel;

use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id};

pub fn draw_about_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.about_open {
        return;
    }

    let colors = state.colors();
    let win_w = 400.0;
    let win_h = 240.0;

    // Center of viewport
    let viewport = ui.current_viewport;
    let center_x = viewport.origin.x + (viewport.size.width - win_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height - win_h) / 2.0;

    ui.container()
        .id(Id::from_u64(999999088))
        .absolute(center_x, center_y)
        .overlay()
        .width(win_w)
        .height(win_h)
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .radius_all(8.0)
        .padding(16.0, 16.0, 16.0, 16.0)
        .column()
        .halign(Align::Center)
        .valign(Align::Center)
        .gap(12.0)
        .show(|ui| {
            ui.text(zenthra::icons::NF_FA_CODE)
                .size(36.0)
                .color(colors.accent)
                .show();

            ui.text("Zenthree Editor")
                .size(18.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();

            ui.text("Version 0.1.0")
                .size(11.0)
                .color(colors.text_muted)
                .show();

            ui.text("A high-performance text and code editor built natively on the Zenthra framework.")
                .size(11.5)
                .color(colors.text_muted)
                .align(Align::Center)
                .show();

            ui.spacing(8.0);

            let close_btn = ui.container()
                .padding(6.0, 16.0, 6.0, 16.0)
                .bg(colors.accent)
                .hover_bg(colors.highlight)
                .radius_all(4.0)
                .show(|ui| {
                    ui.text("Close")
                        .size(12.0)
                        .weight(FontWeight::Bold)
                        .color(Color::WHITE)
                        .show();
                });

            if close_btn.clicked {
                state.about_open = false;
                ui.request_redraw();
            }
        });
}
