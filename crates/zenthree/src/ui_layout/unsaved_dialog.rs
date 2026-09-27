use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, use_context};
use crate::theme::ThemeColors;

pub fn draw_unsaved_changes_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let dialog = match state.unsaved_changes_dialog.as_mut() {
        Some(d) => d,
        None => return,
    };

    let viewport = ui.current_viewport;
    let win_w = 460.0f32.min(viewport.size.width - 32.0);
    let center_x = viewport.origin.x + (viewport.size.width - win_w) / 2.0;

    let is_switch = dialog.target_workspace.is_some();
    let unsaved_count = dialog.unsaved_files.len();

    let approx_h = 190.0f32 + (unsaved_count.min(4) as f32 * 20.0);
    let center_y = viewport.origin.y + (viewport.size.height - approx_h) / 2.0;

    let modal_id = Id::from_u64(999996002);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_window_id = Some(modal_id);
    let prev_win_id = ui.current_window_id;
    ui.current_window_id = Some(modal_id);
    ui.record_layout(modal_id, zenthra_core::Rect::new(center_x, center_y, win_w, approx_h));

    // Semi-transparent backdrop overlay across entire viewport to block clicks behind
    ui.container()
        .id(Id::from_u64(999996001))
        .absolute(viewport.origin.x, viewport.origin.y)
        .overlay()
        .width(viewport.size.width)
        .height(viewport.size.height)
        .bg(Color { r: 0.0, g: 0.0, b: 0.0, a: 0.60 })
        .show(|_ui| {});

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.88)
    } else {
        colors.bg_sidebar
    };

    let mut do_save = false;
    let mut do_discard = false;
    let mut do_cancel = false;

    // Check keyboard events for Escape or Enter
    for event in &ui.input_events {
        if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
            match key {
                winit::keyboard::KeyCode::Escape => {
                    do_cancel = true;
                }
                _ => {}
            }
        }
    }

    // Outside click detection: ignore click that opened the dialog
    if dialog.just_opened {
        dialog.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, win_w, approx_h);
        if ui.clicked && !mouse_in_dialog {
            do_cancel = true;
        }
    }

    let mut card = ui.container()
        .id(modal_id)
        .absolute(center_x, center_y)
        .overlay()
        .width(win_w)
        .bg(card_bg)
        .border(colors.border, 1.0)
        .radius_all(8.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    card.padding(16.0, 20.0, 16.0, 20.0)
        .column()
        .gap(10.0)
        .show(|ui| {
            // Row 1: Header (Warning Badge + Title + Close Button)
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
                            ui.container()
                                .width(24.0)
                                .height(24.0)
                                .bg(Color { r: 245.0 / 255.0, g: 158.0 / 255.0, b: 11.0 / 255.0, a: 0.20 })
                                .radius_all(4.0)
                                .valign(Align::Center)
                                .halign(Align::Center)
                                .show(|ui| {
                                    ui.text("⚠️")
                                        .size(13.0)
                                        .show();
                                });

                            ui.text("Unsaved Changes")
                                .size(13.5)
                                .weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    let close_btn = ui.button("✕")
                        .id(Id::from_u64(999996003))
                        .width(20.0)
                        .height(20.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(colors.highlight)
                        .text_color(colors.text_muted)
                        .radius_all(3.0)
                        .size(11.0)
                        .show();

                    if close_btn.clicked {
                        do_cancel = true;
                    }
                });

            // Row 2: Prompt message
            let prompt_text = if is_switch {
                if let Some(ref target) = dialog.target_workspace {
                    let folder_name = target.file_name().and_then(|n| n.to_str()).unwrap_or("the new folder");
                    format!("Do you want to save changes to the following file(s) before opening '{}'?", folder_name)
                } else {
                    "Do you want to save changes before opening the new folder?".to_string()
                }
            } else {
                "Do you want to save changes before closing the folder?".to_string()
            };

            ui.text(&prompt_text)
                .size(11.5)
                .color(colors.text_primary)
                .show();

            // Row 3: List of dirty files
            ui.container()
                .full_width()
                .bg(colors.bg_base.with_alpha(0.60))
                .border(colors.border.with_alpha(0.60), 0.5)
                .radius_all(4.0)
                .padding(8.0, 10.0, 8.0, 10.0)
                .column()
                .gap(4.0)
                .show(|ui| {
                    let max_display = 4;
                    for file in dialog.unsaved_files.iter().take(max_display) {
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(6.0)
                            .show(|ui| {
                                ui.text("●")
                                    .size(7.0)
                                    .color(Color { r: 245.0 / 255.0, g: 158.0 / 255.0, b: 11.0 / 255.0, a: 0.90 })
                                    .show();
                                ui.text(file)
                                    .size(11.0)
                                    .color(colors.text_primary)
                                    .show();
                            });
                    }
                    if unsaved_count > max_display {
                        ui.text(&format!("... and {} other file(s)", unsaved_count - max_display))
                            .size(10.5)
                            .color(colors.text_muted)
                            .show();
                    }
                });

            ui.text("Your changes will be lost if you open without saving.")
                .size(10.5)
                .color(colors.text_muted)
                .show();

            ui.spacing(4.0);

            // Row 4: Action Buttons Row: [Cancel]  [Don't Save]  [Save / Save All]
            ui.container()
                .full_width()
                .row()
                .valign(Align::Center)
                .halign(Align::Right)
                .gap(8.0)
                .show(|ui| {
                    let cancel_btn = ui.button("Cancel")
                        .id(Id::from_u64(999996004))
                        .height(26.0)
                        .bg(Color::TRANSPARENT)
                        .hover_bg(colors.highlight)
                        .border(colors.border, 1.0)
                        .text_color(colors.text_primary)
                        .radius_all(4.0)
                        .padding_left(12.0)
                        .padding_right(12.0)
                        .size(11.0)
                        .show();

                    if cancel_btn.clicked {
                        do_cancel = true;
                    }

                    let discard_btn = ui.button("Don't Save")
                        .id(Id::from_u64(999996005))
                        .height(26.0)
                        .bg(Color { r: 239.0 / 255.0, g: 68.0 / 255.0, b: 68.0 / 255.0, a: 0.15 })
                        .hover_bg(Color { r: 239.0 / 255.0, g: 68.0 / 255.0, b: 68.0 / 255.0, a: 0.28 })
                        .border(Color { r: 239.0 / 255.0, g: 68.0 / 255.0, b: 68.0 / 255.0, a: 0.40 }, 1.0)
                        .text_color(Color { r: 248.0 / 255.0, g: 113.0 / 255.0, b: 113.0 / 255.0, a: 1.0 })
                        .radius_all(4.0)
                        .padding_left(12.0)
                        .padding_right(12.0)
                        .size(11.0)
                        .show();

                    if discard_btn.clicked {
                        do_discard = true;
                    }

                    let save_label = if unsaved_count > 1 { "Save All" } else { "Save" };
                    let save_btn = ui.button(save_label)
                        .id(Id::from_u64(999996006))
                        .height(26.0)
                        .bg(colors.accent)
                        .hover_bg(colors.accent.with_alpha(0.85))
                        .text_color(Color::WHITE)
                        .radius_all(4.0)
                        .padding_left(14.0)
                        .padding_right(14.0)
                        .size(11.0)
                        .show();

                    if save_btn.clicked {
                        do_save = true;
                    }
                });
        });

    if do_save || do_discard || do_cancel {
        ui.interaction_state.remove(&modal_key);
        ui.consume_click();
    }

    if do_save {
        state.save_unsaved_and_proceed();
        ui.request_redraw();
    } else if do_discard {
        state.discard_unsaved_and_proceed();
        ui.request_redraw();
    } else if do_cancel {
        state.cancel_unsaved_dialog();
        ui.request_redraw();
    }

    ui.current_window_id = prev_win_id;
}
