use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id};

pub fn draw_language_prompt_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();
    let dialog = match state.active_language_prompt.as_mut() {
        Some(d) => d,
        None => return,
    };

    // Floating modal dimensions & position (anchored to bottom-right of window, above status bar)
    let viewport = ui.current_viewport;
    let card_w = 410.0f32.min(viewport.size.width - 24.0);
    let card_h = 162.0f32;
    let card_x = viewport.origin.x + viewport.size.width - card_w - 20.0;
    let card_y = viewport.origin.y + viewport.size.height - card_h - 32.0;

    let ext_id = dialog.extension_id.clone();
    let lang_name = dialog.language_name.clone();
    let file_name = dialog.file_name.clone();
    let dont_show = dialog.dont_show_again_checked;

    let mut do_install = false;
    let mut do_dismiss = false;
    let mut toggle_dont_show = false;

    // Card container
    ui.container()
        .id(Id::from_u64(999998100))
        .absolute(card_x, card_y)
        .overlay()
        .width(card_w)
        .height(card_h)
        .bg(colors.bg_sidebar)
        .border(colors.accent.with_alpha(0.6), 1.5)
        .radius_all(8.0)
        .padding(14.0, 16.0, 12.0, 16.0)
        .column()
        .show(|ui| {
            // Row 1: Header (Icon + Title + Close button)
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            // Icon badge
                            ui.container()
                                .width(22.0)
                                .height(22.0)
                                .bg(colors.accent.with_alpha(0.2))
                                .radius_all(4.0)
                                .valign(Align::Center)
                                .halign(Align::Center)
                                .show(|ui| {
                                    ui.text("🧩")
                                        .size(12.0)
                                        .show();
                                });

                            ui.text("Language Support Available")
                                .size(12.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    let close_btn = ui.button("✕")
                        .id(Id::from_u64(999998101))
                        .width(20.0)
                        .height(20.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.1))
                        .text_color(colors.text_muted)
                        .radius_all(3.0)
                        .size(11.0)
                        .show();

                    if close_btn.clicked {
                        do_dismiss = true;
                    }
                });

            ui.spacing(8.0);

            // Row 2: Prompt message
            let msg = format!(
                "Detected \"{}\". Install {} for full syntax highlighting, indentation, and formatting tools?",
                file_name, lang_name
            );
            ui.text(&msg)
                .size(11.0)
                .color(colors.text_muted)
                .show();

            ui.spacing(8.0);

            // Row 3: "Don't show again" Checkbox
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    let box_mark = if dont_show { "[✓]" } else { "[ ]" };
                    let cb_btn = ui.button(box_mark)
                        .id(Id::from_u64(999998102))
                        .width(24.0)
                        .height(18.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.08))
                        .text_color(if dont_show { colors.accent } else { colors.text_muted })
                        .size(10.5)
                        .padding(0.0, 0.0, 0.0, 0.0)
                        .show();

                    if cb_btn.clicked {
                        toggle_dont_show = true;
                    }

                    ui.text(&format!("Don't ask again for {}", lang_name))
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });

            ui.spacing(10.0);

            // Row 4: Action Buttons (Later & Install)
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::Right)
                .gap(8.0)
                .show(|ui| {
                    let later_btn = ui.button("Later")
                        .id(Id::from_u64(999998103))
                        .height(24.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(Color::rgba(255.0 / 255.0, 255.0 / 255.0, 255.0 / 255.0, 0.08))
                        .text_color(colors.text_muted)
                        .radius_all(4.0)
                        .padding_left(10.0)
                        .padding_right(10.0)
                        .size(11.0)
                        .show();

                    if later_btn.clicked {
                        do_dismiss = true;
                    }

                    let inst_label = format!("Install {}", lang_name);
                    let inst_btn = ui.button(&inst_label)
                        .id(Id::from_u64(999998104))
                        .height(24.0)
                        .bg(colors.accent)
                        .hover_bg(colors.accent.with_alpha(0.85))
                        .text_color(Color::WHITE)
                        .radius_all(4.0)
                        .padding_left(12.0)
                        .padding_right(12.0)
                        .size(11.0)
                        .show();

                    if inst_btn.clicked {
                        do_install = true;
                    }
                });
        });

    if toggle_dont_show {
        if let Some(ref mut d) = state.active_language_prompt {
            d.dont_show_again_checked = !d.dont_show_again_checked;
        }
        ui.request_redraw();
    }

    if do_install {
        state.install_extension(&ext_id);
        ui.request_redraw();
    } else if do_dismiss {
        state.dismiss_language_prompt(dont_show);
        ui.request_redraw();
    }
}
