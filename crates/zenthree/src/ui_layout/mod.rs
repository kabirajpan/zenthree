pub mod title_bar;
pub mod sidebar;
pub mod main_buffer;
pub mod status_bar;
pub mod terminal;
pub mod ai_panel;
pub mod language_dialog;
pub mod extensions_panel;
pub mod activity_bar;
pub mod ai_settings_dialog;
pub mod editor_settings_dialog;
pub mod git_panel;
pub mod notes_panel;
pub mod thinking_orbs_demo_window;
pub mod markdown_preview;
pub mod unsaved_dialog;

pub use title_bar::draw_title_bar;
pub use sidebar::draw_sidebar;
pub use git_panel::draw_git_panel;
#[allow(unused_imports)]
pub use notes_panel::draw_notes_panel;
pub use main_buffer::draw_main_buffer;
pub use status_bar::draw_status_bar;
pub use terminal::draw_terminal_panel;
pub use ai_panel::draw_ai_panel;
pub use language_dialog::draw_language_prompt_dialog;
pub use extensions_panel::draw_extensions_panel;
pub use activity_bar::{draw_activity_bar, ACTIVITY_BAR_WIDTH};
pub use ai_settings_dialog::{draw_ai_settings_dialog, draw_antigravity_settings_model_popover};
pub use editor_settings_dialog::draw_editor_settings_dialog;
pub use thinking_orbs_demo_window::draw_thinking_orbs_demo_window;
pub use markdown_preview::draw_markdown_buffer_preview;
pub use unsaved_dialog::draw_unsaved_changes_dialog;

use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, use_context};
use crate::theme::ThemeColors;

pub fn draw_about_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.about_open {
        return;
    }

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let win_w = 400.0;
    let win_h = 240.0;

    // Center of viewport
    let viewport = ui.current_viewport;
    let center_x = viewport.origin.x + (viewport.size.width - win_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height - win_h) / 2.0;

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.80)
    } else {
        colors.bg_sidebar
    };

    let modal_id = Id::from_u64(999999088);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_window_id = Some(modal_id);
    let prev_win_id = ui.current_window_id;
    ui.current_window_id = Some(modal_id);
    ui.record_layout(modal_id, zenthra_core::Rect::new(center_x, center_y, win_w, win_h));

    // Outside click dismisses dialog and consumes click: ignore click that opened the dialog
    if state.about_just_opened {
        state.about_just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, win_w, win_h);
        if ui.clicked && !mouse_in_dialog {
            state.about_open = false;
            ui.interaction_state.remove(&modal_key);
            ui.consume_click();
            ui.request_redraw();
            return;
        }
    }

    let mut card = ui.container()
        .id(modal_id)
        .absolute(center_x, center_y)
        .overlay()
        .width(win_w)
        .height(win_h)
        .bg(card_bg)
        .border(colors.border, 1.0)
        .radius_all(8.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    card.padding(16.0, 16.0, 16.0, 16.0)
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
                .max_width(win_w - 32.0)
                .wrap(true)
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
                ui.interaction_state.remove(&modal_key);
                ui.consume_click();
                ui.request_redraw();
            }
        });

    ui.current_window_id = prev_win_id;
}
