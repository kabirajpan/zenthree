use crate::state::ZenthreeState;
use crate::theme::ThemeColors;
use zenthra::{Align, FontWeight, Ui, use_context};

pub fn draw_notes_panel(ui: &mut Ui, state: &mut ZenthreeState, sidebar_w: f32) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    ui.container()
        .full_width()
        .height(36.0)
        .padding(8.0, 10.0, 8.0, 10.0)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .show(|ui| {
            ui.container()
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    ui.text("\u{f1782}")
                        .size(13.0)
                        .color(colors.accent)
                        .show();
                    ui.text("NOTES")
                        .size(11.0)
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                });
        });

    ui.container()
        .width(sidebar_w)
        .padding(20.0, 14.0, 20.0, 14.0)
        .column()
        .valign(Align::Center)
        .halign(Align::Center)
        .gap(8.0)
        .show(|ui| {
            ui.text("\u{f1782}")
                .size(28.0)
                .color(colors.text_muted.with_alpha(0.5))
                .show();
            ui.text("Notes & Scratchpad")
                .size(12.0)
                .weight(FontWeight::Medium)
                .color(colors.text_primary)
                .show();
            ui.text("Quick workspace notes coming soon.")
                .size(10.5)
                .color(colors.text_muted)
                .show();
        });
}
