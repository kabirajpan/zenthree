use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, TextWrap, TextDecoration, Style, use_context};
use crate::theme::ThemeColors;
use zenthra_widgets::ui::{DrawCommand, OverlayRectDraw};

pub fn draw_language_prompt_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if state.active_language_prompt.is_some() {
        draw_language_pack_prompt(ui, state);
    } else if state.active_lsp_prompt.is_some() {
        draw_lsp_prompt(ui, state);
    }
}

fn draw_language_pack_prompt(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let dialog = match state.active_language_prompt.as_mut() {
        Some(d) => d,
        None => return,
    };

    let viewport = ui.current_viewport;
    let card_w = 460.0f32.min(viewport.size.width - 32.0);
    let card_h = 185.0f32;
    let center_x = viewport.origin.x + (viewport.size.width - card_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height - card_h) / 2.0;
    let content_w = card_w - 36.0;

    let ext_id = dialog.extension_id.clone();
    let lang_name = dialog.language_name.clone();
    let file_name = dialog.file_name.clone();
    let mut dont_show = dialog.dont_show_again_checked;

    let mut do_install = false;
    let mut do_dismiss = false;

    // Semi-transparent backdrop overlay
    ui.draws.push(DrawCommand::OverlayRect(OverlayRectDraw {
        x: viewport.origin.x,
        y: viewport.origin.y,
        width: viewport.size.width,
        height: viewport.size.height,
        color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.55 },
        clip: [0.0, 0.0, 99999.0, 99999.0],
    }));

    // Escape key handling
    ui.input_events.retain(|event| {
        if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
            if *key == winit::keyboard::KeyCode::Escape {
                do_dismiss = true;
                return false;
            }
        }
        true
    });

    let modal_id = Id::from_u64(999998100);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    let prev_modal_active = ui.modal_active;
    let prev_modal_win_id = ui.modal_window_id;
    let prev_win_id = ui.current_window_id;
    ui.modal_active = true;
    ui.modal_window_id = Some(modal_id);
    ui.current_window_id = Some(modal_id);
    ui.record_layout(modal_id, zenthra_core::Rect::new(center_x, center_y, card_w, card_h));

    if dialog.just_opened {
        dialog.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, card_w, card_h);
        if ui.clicked && !mouse_in_dialog {
            do_dismiss = true;
            ui.consume_click();
        }
    }

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.92)
    } else {
        colors.bg_sidebar
    };

    let mut card = ui.container()
        .id(modal_id)
        .pos(center_x, center_y)
        .overlay()
        .width(card_w)
        .height(card_h)
        .bg(card_bg)
        .border((colors.accent.with_alpha(0.6), 1.5))
        .radius_all(10.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    card.padding(16.0, 18.0, 14.0, 18.0)
        .column()
        .show(|ui| {
            // Header
            ui.container()
                .fill_x()
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
                                .bg(colors.accent.with_alpha(0.2))
                                .radius_all(5.0)
                                .valign(Align::Center)
                                .halign(Align::Center)
                                .show(|ui| {
                                    ui.text("")
                                        .size(13.0)
                                        .color(colors.accent)
                                        .show();
                                });

                            ui.text("Language Pack Available")
                                .size(13.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    let close_btn = ui.button("✕")
                        .id(Id::from_u64(999998101))
                        .width(22.0)
                        .height(22.0)
                        .bg(Color::TRANSPARENT)
                        .hover(Style::bg(colors.highlight))
                        .color(colors.text_muted)
                        .radius_all(4.0)
                        .size(12.0)
                        .show();

                    if close_btn.clicked {
                        do_dismiss = true;
                    }
                });

            ui.spacing(10.0);

            // Message
            let msg = format!(
                "Detected \"{}\". Install {} Language Pack for Tree-sitter AST syntax highlighting, auto-indent, and code formatting?",
                file_name, lang_name
            );
            ui.text(&msg)
                .size(11.5)
                .color(colors.text_muted)
                .wrap(TextWrap::Word)
                .max_width(content_w)
                .decoration(TextDecoration::LineHeight(1.4))
                .show();

            ui.spacing(10.0);

            // Checkbox
            let cb_resp = ui.checkbox(&mut dont_show, &format!("Don't ask again for {}", lang_name))
                .box_size(15.0)
                .radius_all(3.0)
                .check_bg(colors.accent)
                .border((colors.border, 1.0))
                .size(11.5)
                .color(colors.text_muted)
                .show();

            if cb_resp.clicked || dont_show != dialog.dont_show_again_checked {
                dialog.dont_show_again_checked = dont_show;
                ui.request_redraw();
            }

            ui.spacing(12.0);

            // Action Buttons
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .halign(Align::Right)
                .gap(10.0)
                .show(|ui| {
                    let later_btn = ui.button("Later")
                        .id(Id::from_u64(999998103))
                        .height(28.0)
                        .bg(colors.highlight.with_alpha(0.35))
                        .hover(Style::bg(colors.highlight))
                        .border((colors.border, 1.0))
                        .color(colors.text_primary)
                        .radius_all(4.0)
                        .padding_left(16.0)
                        .padding_right(16.0)
                        .size(11.5)
                        .show();

                    if later_btn.clicked {
                        do_dismiss = true;
                    }

                    let inst_label = format!("Install {} Pack", lang_name);
                    let inst_btn = ui.button(&inst_label)
                        .id(Id::from_u64(999998104))
                        .height(28.0)
                        .bg(colors.accent)
                        .hover(Style::bg(colors.accent.with_alpha(0.85)))
                        .color(Color::WHITE)
                        .radius_all(4.0)
                        .padding_left(16.0)
                        .padding_right(16.0)
                        .size(11.5)
                        .show();

                    if inst_btn.clicked {
                        do_install = true;
                    }
                });
        });

    ui.current_window_id = prev_win_id;
    ui.modal_active = prev_modal_active;
    ui.modal_window_id = prev_modal_win_id;

    if do_install || do_dismiss {
        ui.interaction_state.remove(&modal_key);
        ui.consume_click();
    }

    if do_install {
        state.install_extension(&ext_id);
        state.dismiss_language_prompt(dont_show);
        ui.request_redraw();
    } else if do_dismiss {
        state.dismiss_language_prompt(dont_show);
        ui.request_redraw();
    }
}

fn draw_lsp_prompt(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let dialog = match state.active_lsp_prompt.as_mut() {
        Some(d) => d,
        None => return,
    };

    let viewport = ui.current_viewport;
    let card_w = 460.0f32.min(viewport.size.width - 32.0);
    let card_h = 185.0f32;
    let center_x = viewport.origin.x + (viewport.size.width - card_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height - card_h) / 2.0;
    let content_w = card_w - 36.0;

    let lang_id = dialog.language_id.clone();
    let lang_name = dialog.language_name.clone();
    let server_name = dialog.server_name.clone();
    let file_name = dialog.file_name.clone();
    let mut dont_show = dialog.dont_show_again_checked;

    let mut do_install = false;
    let mut do_dismiss = false;

    // Semi-transparent backdrop overlay
    ui.draws.push(DrawCommand::OverlayRect(OverlayRectDraw {
        x: viewport.origin.x,
        y: viewport.origin.y,
        width: viewport.size.width,
        height: viewport.size.height,
        color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.55 },
        clip: [0.0, 0.0, 99999.0, 99999.0],
    }));

    // Escape key handling
    ui.input_events.retain(|event| {
        if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
            if *key == winit::keyboard::KeyCode::Escape {
                do_dismiss = true;
                return false;
            }
        }
        true
    });

    let modal_id = Id::from_u64(999998110);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    let prev_modal_active = ui.modal_active;
    let prev_modal_win_id = ui.modal_window_id;
    let prev_win_id = ui.current_window_id;
    ui.modal_active = true;
    ui.modal_window_id = Some(modal_id);
    ui.current_window_id = Some(modal_id);
    ui.record_layout(modal_id, zenthra_core::Rect::new(center_x, center_y, card_w, card_h));

    if dialog.just_opened {
        dialog.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, card_w, card_h);
        if ui.clicked && !mouse_in_dialog {
            do_dismiss = true;
            ui.consume_click();
        }
    }

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.92)
    } else {
        colors.bg_sidebar
    };

    let mut card = ui.container()
        .id(modal_id)
        .pos(center_x, center_y)
        .overlay()
        .width(card_w)
        .height(card_h)
        .bg(card_bg)
        .border((colors.accent.with_alpha(0.6), 1.5))
        .radius_all(10.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    card.padding(16.0, 18.0, 14.0, 18.0)
        .column()
        .show(|ui| {
            // Header
            ui.container()
                .fill_x()
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
                                .bg(colors.accent.with_alpha(0.2))
                                .radius_all(5.0)
                                .valign(Align::Center)
                                .halign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f0627}")
                                        .size(13.0)
                                        .color(colors.accent)
                                        .show();
                                });

                            ui.text("Language Server (LSP) Available")
                                .size(13.0)
                                .font_weight(FontWeight::Bold)
                                .color(colors.text_primary)
                                .show();
                        });

                    let close_btn = ui.button("✕")
                        .id(Id::from_u64(999998111))
                        .width(22.0)
                        .height(22.0)
                        .bg(Color::TRANSPARENT)
                        .hover(Style::bg(colors.highlight))
                        .color(colors.text_muted)
                        .radius_all(4.0)
                        .size(12.0)
                        .show();

                    if close_btn.clicked {
                        do_dismiss = true;
                    }
                });

            ui.spacing(10.0);

            // Message
            let msg = format!(
                "Detected \"{}\". Install {} ({}) for code navigation, completions, type hover, and diagnostics?",
                file_name, server_name, lang_name
            );
            ui.text(&msg)
                .size(11.5)
                .color(colors.text_muted)
                .wrap(TextWrap::Word)
                .max_width(content_w)
                .decoration(TextDecoration::LineHeight(1.4))
                .show();

            ui.spacing(10.0);

            // Checkbox
            let cb_resp = ui.checkbox(&mut dont_show, &format!("Don't ask again for {}", lang_name))
                .box_size(15.0)
                .radius_all(3.0)
                .check_bg(colors.accent)
                .border((colors.border, 1.0))
                .size(11.5)
                .color(colors.text_muted)
                .show();

            if cb_resp.clicked || dont_show != dialog.dont_show_again_checked {
                dialog.dont_show_again_checked = dont_show;
                ui.request_redraw();
            }

            ui.spacing(12.0);

            // Action Buttons
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .halign(Align::Right)
                .gap(10.0)
                .show(|ui| {
                    let later_btn = ui.button("Later")
                        .id(Id::from_u64(999998113))
                        .height(28.0)
                        .bg(colors.highlight.with_alpha(0.35))
                        .hover(Style::bg(colors.highlight))
                        .border((colors.border, 1.0))
                        .color(colors.text_primary)
                        .radius_all(4.0)
                        .padding_left(16.0)
                        .padding_right(16.0)
                        .size(11.5)
                        .show();

                    if later_btn.clicked {
                        do_dismiss = true;
                    }

                    let inst_label = format!("\u{f0627} Install {} LSP", lang_name);
                    let inst_btn = ui.button(&inst_label)
                        .id(Id::from_u64(999998114))
                        .height(28.0)
                        .bg(colors.accent)
                        .hover(Style::bg(colors.accent.with_alpha(0.85)))
                        .color(Color::WHITE)
                        .radius_all(4.0)
                        .padding_left(16.0)
                        .padding_right(16.0)
                        .size(11.5)
                        .show();

                    if inst_btn.clicked {
                        do_install = true;
                    }
                });
        });

    ui.current_window_id = prev_win_id;
    ui.modal_active = prev_modal_active;
    ui.modal_window_id = prev_modal_win_id;

    if do_install || do_dismiss {
        ui.interaction_state.remove(&modal_key);
        ui.consume_click();
    }

    if do_install {
        state.start_lsp_install(&lang_id);
        state.dismiss_lsp_prompt(dont_show);
        ui.request_redraw();
    } else if do_dismiss {
        state.dismiss_lsp_prompt(dont_show);
        ui.request_redraw();
    }
}
