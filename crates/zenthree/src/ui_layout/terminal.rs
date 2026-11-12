use crate::state::ZenthreeState;
use crate::theme::ThemeColors;
use zenthra::{use_context, Align, Color, CursorIcon, FontWeight, Id, Style, Ui};
use zenthra_render::RectInstance;
use zenthra_text::prelude::TextOptions;
use zenthra_widgets::ui::{DrawCommand, RectDraw, ScrollDrag, TextDraw};

fn convert_vt_color(c: vt100::Color, default_color: Color) -> Color {
    match c {
        vt100::Color::Default => default_color,
        vt100::Color::Rgb(r, g, b) => {
            Color::rgb(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
        }
        vt100::Color::Idx(idx) => match idx {
            0 => Color::rgb(0.0, 0.0, 0.0),
            1 => Color::rgb(0.85, 0.25, 0.25),
            2 => Color::rgb(0.25, 0.85, 0.25),
            3 => Color::rgb(0.85, 0.85, 0.25),
            4 => Color::rgb(0.35, 0.55, 0.95),
            5 => Color::rgb(0.85, 0.35, 0.85),
            6 => Color::rgb(0.25, 0.85, 0.85),
            7 => Color::rgb(0.85, 0.85, 0.85),
            8 => Color::rgb(0.45, 0.45, 0.45),
            9 => Color::rgb(1.0, 0.35, 0.35),
            10 => Color::rgb(0.35, 1.0, 0.35),
            11 => Color::rgb(1.0, 1.0, 0.35),
            12 => Color::rgb(0.45, 0.65, 1.0),
            13 => Color::rgb(1.0, 0.45, 1.0),
            14 => Color::rgb(0.35, 1.0, 1.0),
            15 => Color::rgb(1.0, 1.0, 1.0),
            16..=231 => {
                let i = idx - 16;
                let b = (i % 6) as f32 / 5.0;
                let g = ((i / 6) % 6) as f32 / 5.0;
                let r = (i / 36) as f32 / 5.0;
                Color::rgb(r, g, b)
            }
            232..=255 => {
                let gray = (idx - 232) as f32 / 23.0 * 0.9 + 0.05;
                Color::rgb(gray, gray, gray)
            }
        },
    }
}

fn terminal_command_color(row: &str, column: usize, default_color: Color) -> Option<Color> {
    let chars: Vec<char> = row.chars().collect();
    let prompt_end = if row.trim_start().starts_with("PS ") {
        chars.iter().position(|character| *character == '>')? + 1
    } else if chars.get(1) == Some(&':') && matches!(chars.get(2), Some('\\' | '/')) {
        chars.iter().position(|character| *character == '>')? + 1
    } else {
        ["$ ", "% ", "# ", "❯ "]
            .iter()
            .filter_map(|prompt| {
                let prompt_chars: Vec<_> = prompt.chars().collect();
                chars
                    .windows(prompt_chars.len())
                    .rposition(|window| window == prompt_chars)
                    .map(|index| index + prompt_chars.len())
            })
            .max()?
    };

    let command_start = if chars.get(prompt_end) == Some(&' ') {
        prompt_end + 1
    } else {
        prompt_end
    };
    if column < command_start {
        return None;
    }

    let delimiter = |character: char| {
        character.is_whitespace() || matches!(character, '|' | '&' | ';' | '>' | '<')
    };
    let mut token_start = command_start;
    let mut command_token = true;
    while token_start < chars.len() {
        while token_start < chars.len() && chars[token_start].is_whitespace() {
            token_start += 1;
            command_token = false;
        }
        if token_start >= chars.len() {
            return None;
        }

        let token_end = if matches!(chars[token_start], '|' | '&' | ';' | '>' | '<') {
            if column == token_start {
                return Some(Color::rgb(0.90, 0.72, 0.30));
            }
            token_start += 1;
            command_token = true;
            continue;
        } else if matches!(chars[token_start], '\'' | '"') {
            let quote = chars[token_start];
            let mut end = token_start + 1;
            while end < chars.len() && chars[end] != quote {
                end += 1;
            }
            (end + usize::from(end < chars.len())).min(chars.len())
        } else {
            let mut end = token_start + 1;
            while end < chars.len() && !delimiter(chars[end]) {
                end += 1;
            }
            end
        };

        if (token_start..token_end).contains(&column) {
            let token = &chars[token_start..token_end];
            if matches!(token.first(), Some('$' | '%')) {
                return Some(Color::rgb(0.78, 0.55, 0.92));
            }
            if matches!(token.first(), Some('\'' | '"')) {
                return Some(Color::rgb(0.86, 0.70, 0.43));
            }
            if token.first() == Some(&'-') {
                return Some(Color::rgb(0.45, 0.70, 0.95));
            }
            if command_token {
                return Some(Color::rgb(0.35, 0.82, 0.52));
            }
            if token
                .iter()
                .any(|character| matches!(character, '/' | '\\'))
                || token.first() == Some(&'.')
            {
                return Some(Color::rgb(0.35, 0.75, 0.82));
            }
            return Some(default_color);
        }

        token_start = token_end;
        command_token = false;
    }
    None
}

fn terminal_history_key_sequence(key: winit::keyboard::KeyCode) -> Option<&'static [u8]> {
    match key {
        winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[A"),
        winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[B"),
        _ => None,
    }
}

fn ordered_selection(selection: crate::state::types::TerminalTextSelection) -> ((usize, usize), (usize, usize)) {
    let anchor = (selection.anchor_row, selection.anchor_column);
    let focus = (selection.focus_row, selection.focus_column);
    if anchor <= focus {
        (anchor, focus)
    } else {
        (focus, anchor)
    }
}

fn terminal_cell_is_selected(
    selection: crate::state::types::TerminalTextSelection,
    row: usize,
    column: usize,
) -> bool {
    let (start, end) = ordered_selection(selection);
    (row, column) >= start && (row, column) <= end
}

fn terminal_selection_text(
    screen: &vt100::Screen,
    selection: crate::state::types::TerminalTextSelection,
) -> String {
    let (start, end) = ordered_selection(selection);
    let (rows, columns) = screen.size();
    let first_row = start.0.min(rows.saturating_sub(1) as usize);
    let last_row = end.0.min(rows.saturating_sub(1) as usize);
    let mut text = String::new();

    for row in first_row..=last_row {
        let first_column = if row == start.0 { start.1 } else { 0 };
        let last_column = if row == end.0 {
            end.1
        } else {
            columns.saturating_sub(1) as usize
        }
        .min(columns.saturating_sub(1) as usize);
        let mut line = String::new();
        for column in first_column.min(columns.saturating_sub(1) as usize)..=last_column {
            let contents = screen
                .cell(row as u16, column as u16)
                .map(|cell| cell.contents())
                .unwrap_or(" ");
            line.push_str(if contents.is_empty() { " " } else { contents });
        }
        text.push_str(line.trim_end());
        if row < last_row {
            text.push('\n');
        }
    }

    text
}

fn selected_terminal_text(state: &ZenthreeState, terminal_idx: usize) -> Option<String> {
    let selection = state.terminal_text_selection?;
    if selection.terminal_index != terminal_idx
        || (selection.anchor_row, selection.anchor_column)
            == (selection.focus_row, selection.focus_column)
    {
        return None;
    }

    let session = state.terminal_session_at(terminal_idx)?;
    let parser = session.parser.lock().ok()?;
    Some(terminal_selection_text(parser.screen(), selection))
}

#[cfg(test)]
mod command_highlight_tests {
    use super::{
        ordered_selection, terminal_cell_is_selected, terminal_command_color,
        terminal_history_key_sequence, terminal_selection_text,
    };
    use crate::state::types::TerminalTextSelection;
    use zenthra::Color;

    #[test]
    fn highlights_commands_after_powershell_and_cmd_prompts() {
        let powershell = "PS C:\\Users\\Prasa> python client.py";
        let python_column = powershell.find("python").unwrap();
        assert_eq!(
            terminal_command_color(powershell, python_column, Color::WHITE),
            Some(Color::rgb(0.35, 0.82, 0.52))
        );

        let command_prompt = "C:\\Users\\Prasa> python client.py";
        let python_column = command_prompt.find("python").unwrap();
        assert_eq!(
            terminal_command_color(command_prompt, python_column, Color::WHITE),
            Some(Color::rgb(0.35, 0.82, 0.52))
        );
    }

    #[test]
    fn highlights_shell_flags_and_quoted_arguments() {
        let command = "$ tool --path \"project folder\"";
        assert_eq!(
            terminal_command_color(command, command.find("--path").unwrap(), Color::WHITE),
            Some(Color::rgb(0.45, 0.70, 0.95))
        );
        assert_eq!(
            terminal_command_color(command, command.find("\"project").unwrap(), Color::WHITE),
            Some(Color::rgb(0.86, 0.70, 0.43))
        );
    }

    #[test]
    fn leaves_prompt_and_non_command_rows_unmodified() {
        assert_eq!(
            terminal_command_color("PS C:\\Users\\Prasa>", 0, Color::WHITE),
            None
        );
        assert_eq!(
            terminal_command_color("ordinary program output", 2, Color::WHITE),
            None
        );
    }

    #[test]
    fn maps_control_arrows_to_shell_history_navigation() {
        assert_eq!(
            terminal_history_key_sequence(winit::keyboard::KeyCode::ArrowUp),
            Some(&b"\x1b[A"[..])
        );
        assert_eq!(
            terminal_history_key_sequence(winit::keyboard::KeyCode::ArrowDown),
            Some(&b"\x1b[B"[..])
        );
        assert_eq!(
            terminal_history_key_sequence(winit::keyboard::KeyCode::ArrowLeft),
            None
        );
    }

    #[test]
    fn terminal_selection_is_ordered_and_includes_both_endpoints() {
        let selection = TerminalTextSelection {
            terminal_index: 0,
            anchor_row: 2,
            anchor_column: 5,
            focus_row: 1,
            focus_column: 3,
        };
        assert_eq!(ordered_selection(selection), ((1, 3), (2, 5)));
        assert!(terminal_cell_is_selected(selection, 1, 3));
        assert!(terminal_cell_is_selected(selection, 2, 5));
        assert!(!terminal_cell_is_selected(selection, 1, 2));
        assert!(!terminal_cell_is_selected(selection, 2, 6));
    }

    #[test]
    fn copies_selected_terminal_cells_with_line_breaks() {
        let mut parser = vt100::Parser::new(3, 8, 0);
        parser.process(b"alpha\r\nbeta");
        let selection = TerminalTextSelection {
            terminal_index: 0,
            anchor_row: 0,
            anchor_column: 1,
            focus_row: 1,
            focus_column: 2,
        };

        assert_eq!(terminal_selection_text(parser.screen(), selection), "lpha\nbet");
    }
}

pub fn draw_terminal_toolbar(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    let terminal_idx = state.active_terminal_idx;
    let term_id = terminal_widget_id(terminal_idx);

    ui.container()
        .height(30.0)
        .row()
        .valign(Align::Center)
        .gap(4.0)
        .show(|ui| {
            let split_btn = ui
                .container()
                .width(22.0)
                .height(22.0)
                .radius_all(3.0)
                .hover(Style::bg(colors.highlight))
                .halign(Align::Center)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("\u{eb56}")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });
            if split_btn.clicked {
                if state.terminal_session_count() < 2 {
                    state.create_terminal_session();
                }
                state.terminal_split = state.terminal_session_count() > 1;
                if state.terminal_split {
                    state.active_terminal_idx = state.terminal_session_count() - 1;
                    ui.focused_id = Some(terminal_widget_id(state.active_terminal_idx));
                    state.focused_panel = crate::state::FocusedPanel::Terminal;
                }
                ui.request_redraw();
            }

            let copy_btn = ui
                .container()
                .width(22.0)
                .height(22.0)
                .radius_all(3.0)
                .hover(Style::bg(colors.highlight))
                .halign(Align::Center)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("\u{f018f}")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });
            if copy_btn.clicked {
                let copied_text = selected_terminal_text(state, terminal_idx).or_else(|| {
                    state
                        .terminal_session_at(terminal_idx)
                        .and_then(|session| session.parser.lock().ok())
                        .map(|parser| parser.screen().contents())
                });
                if let Some(text) = copied_text {
                    state.set_clipboard(text);
                    state.set_status("Terminal text copied to clipboard");
                }
                ui.request_redraw();
            }

            let paste_btn = ui
                .container()
                .width(22.0)
                .height(22.0)
                .radius_all(3.0)
                .hover(Style::bg(colors.highlight))
                .halign(Align::Center)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("\u{f0192}")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });
            if paste_btn.clicked {
                state.terminal_text_selection = None;
                state.terminal_selection_drag = None;
                if let Some(text) = state.get_clipboard() {
                    if let Some(session) = state.terminal_session_at_mut(terminal_idx) {
                        session.write_raw(text.as_bytes());
                    }
                    state.last_cursor_activity = std::time::Instant::now();
                    ui.focused_id = Some(term_id);
                    state.focused_panel = crate::state::FocusedPanel::Terminal;
                    ui.request_redraw();
                }
            }

            let clear_btn = ui
                .container()
                .height(22.0)
                .padding(2.0, 6.0, 2.0, 6.0)
                .hover(Style::bg(colors.highlight))
                .radius_all(3.0)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("Clear")
                        .size(10.5)
                        .color(colors.text_muted)
                        .show();
                });
            if clear_btn.clicked {
                state.clear_terminal();
                state.last_cursor_activity = std::time::Instant::now();
                ui.focused_id = Some(term_id);
                state.focused_panel = crate::state::FocusedPanel::Terminal;
                ui.request_redraw();
            }

            let close_btn = ui
                .container()
                .width(22.0)
                .height(22.0)
                .radius_all(3.0)
                .hover(Style::bg(colors.highlight))
                .halign(Align::Center)
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("\u{f0156}")
                        .size(11.5)
                        .color(colors.text_muted)
                        .show();
                });
            if close_btn.clicked {
                state.close_terminal_session(terminal_idx);
                state.terminal_split = false;
                if state.terminal_visible {
                    ui.focused_id = Some(terminal_widget_id(state.active_terminal_idx));
                } else {
                    ui.focused_id = state
                        .active_tab
                        .map(|index| Id::from_u64(999996000 + index as u64));
                }
                state.save_persistent_state();
                ui.request_redraw();
            }
        });
}

pub fn draw_terminal_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>()
        .map(|c| *c)
        .unwrap_or_else(|| state.colors());
    state.ensure_terminal_session();
    let session_list_width = (width * 0.16)
        .clamp(145.0, 190.0)
        .min((width * 0.4).max(0.0));
    ui.container().width(width).height(height).row().show(|ui| {
        let terminal_area_width = (width - session_list_width).max(1.0);
        if state.terminal_split && state.terminal_session_count() > 1 {
            let pane_width = (terminal_area_width - 1.0) / 2.0;
            let right_terminal_idx = if state.active_terminal_idx == 0 {
                1
            } else {
                state.active_terminal_idx
            };
            draw_terminal_session_panel(ui, state, pane_width, height, 0);
            draw_terminal_session_panel(
                ui,
                state,
                terminal_area_width - pane_width,
                height,
                right_terminal_idx,
            );
        } else {
            draw_terminal_session_panel(
                ui,
                state,
                terminal_area_width,
                height,
                state.active_terminal_idx,
            );
        }
        draw_terminal_session_list(ui, state, session_list_width, height, &colors);
    });
    draw_terminal_context_menu(ui, state, &colors);
}

fn draw_terminal_session_list(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    width: f32,
    height: f32,
    colors: &ThemeColors,
) {
    ui.container()
        .width(width)
        .height(height)
        .bg(colors.bg_sidebar)
        .column()
        .show(|ui| {
            let session_count = state.terminal_session_count();
            for index in 0..session_count {
                let selected = state.active_terminal_idx == index;
                let row_x = ui.cursor_x;
                let row_y = ui.cursor_y;
                let row_hovered = ui.mouse_in_rect(row_x, row_y, width, 31.0);
                ui.container()
                    .width(width)
                    .height(31.0)
                    .padding(3.0, 5.0, 3.0, 7.0)
                    .row()
                    .gap(3.0)
                    .valign(Align::Center)
                    .bg(if selected {
                        colors.bg_active_tab
                    } else {
                        colors.bg_sidebar
                    })
                    .show(|ui| {
                        let terminal_entry = ui
                            .container()
                            .width((width - 54.0).max(1.0))
                            .height(25.0)
                            .padding(3.0, 5.0, 3.0, 5.0)
                            .radius_all(3.0)
                            .hover(Style::bg(colors.highlight))
                            .row()
                            .gap(7.0)
                            .valign(Align::Center)
                            .show(|ui| {
                                ui.text("\u{f489}")
                                    .size(11.0)
                                    .color(if selected {
                                        colors.accent
                                    } else {
                                        colors.text_muted
                                    })
                                    .show();
                                ui.text(&format!("Terminal {}", index + 1))
                                    .size(10.5)
                                    .color(if selected {
                                        colors.text_primary
                                    } else {
                                        colors.text_muted
                                    })
                                    .show();
                            });
                        if terminal_entry.clicked {
                            state.select_terminal_session(index);
                            state.terminal_visible = true;
                            state.bottom_panel_tab = crate::state::BottomPanelTab::Terminal;
                            state.focused_panel = crate::state::FocusedPanel::Terminal;
                            ui.focused_id = Some(terminal_widget_id(index));
                            ui.request_redraw();
                        }

                        if row_hovered {
                            let split_button = ui
                                .container()
                                .width(19.0)
                                .height(22.0)
                                .radius_all(3.0)
                                .hover(Style::bg(colors.highlight))
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{eb56}")
                                        .size(10.5)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if split_button.clicked {
                                if state.terminal_session_count() < 2 {
                                    state.create_terminal_session();
                                }
                                if state.terminal_session_count() > 1 {
                                    state.active_terminal_idx =
                                        if index == 0 { 1 } else { index };
                                    state.terminal_split = true;
                                    state.terminal_visible = true;
                                    state.bottom_panel_tab =
                                        crate::state::BottomPanelTab::Terminal;
                                    state.focused_panel =
                                        crate::state::FocusedPanel::Terminal;
                                    ui.focused_id =
                                        Some(terminal_widget_id(state.active_terminal_idx));
                                }
                                ui.request_redraw();
                            }

                            let close_button = ui
                                .container()
                                .width(19.0)
                                .height(22.0)
                                .radius_all(3.0)
                                .hover(Style::bg(colors.highlight))
                                .halign(Align::Center)
                                .valign(Align::Center)
                                .show(|ui| {
                                    ui.text("\u{f0156}")
                                        .size(10.0)
                                        .color(colors.text_muted)
                                        .show();
                                });
                            if close_button.clicked {
                                state.close_terminal_session(index);
                                state.terminal_split =
                                    state.terminal_split && state.terminal_session_count() > 1;
                                if state.terminal_visible {
                                    ui.focused_id =
                                        Some(terminal_widget_id(state.active_terminal_idx));
                                } else {
                                    ui.focused_id = state
                                        .active_tab
                                        .map(|tab| Id::from_u64(999996000 + tab as u64));
                                }
                                state.save_persistent_state();
                                ui.request_redraw();
                            }
                        }
                    });
            }
        });
}

fn terminal_widget_id(index: usize) -> Id {
    Id::from_u64(999999800 + (index as u64 * 1000))
}

fn draw_terminal_session_panel(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    width: f32,
    height: f32,
    terminal_idx: usize,
) {
    let colors = use_context::<ThemeColors>()
        .map(|c| *c)
        .unwrap_or_else(|| state.colors());
    let term_id = terminal_widget_id(terminal_idx);
    let has_other_widget_focus = ui.focused_id.is_some() && ui.focused_id != Some(term_id);
    let is_focused = state.focused_panel == crate::state::types::FocusedPanel::Terminal
        && state.active_terminal_idx == terminal_idx
        && !has_other_widget_focus;

    if is_focused && ui.focused_id.is_none() {
        ui.focused_id = Some(term_id);
    }

    // Exact screen coordinates of the entire terminal panel
    let panel_x = ui.cursor_x;
    let panel_y = ui.cursor_y;
    let panel_w = width;
    let is_blocked = ui.modal_active || state.is_modal_or_dialog_open();

    // Click anywhere in the terminal panel to focus terminal
    let mouse_in_terminal = !is_blocked
        && !state.active_resize_terminal
        && ui.mouse_in_rect(panel_x, panel_y, panel_w, height);

    if ui.right_clicked && mouse_in_terminal {
        state.active_terminal_idx = terminal_idx;
        let root = state.current_dir.clone();
        let target_path = state
            .active_tab
            .and_then(|idx| state.tabs.get(idx))
            .and_then(|tab| tab.path.clone())
            .unwrap_or_else(|| root.clone());
        state.panel_context_menu = Some(crate::state::PanelContextMenu {
            target: crate::state::PanelContextMenuTarget::Terminal,
            target_is_dir: target_path.is_dir(),
            target_path,
            terminal_index: Some(terminal_idx),
            menu_x: ui.mouse_x,
            menu_y: ui.mouse_y,
        });
        ui.consume_click();
        ui.consume_right_click();
        ui.focused_id = Some(term_id);
        state.focused_panel = crate::state::FocusedPanel::Terminal;
        state.last_cursor_activity = std::time::Instant::now();
        ui.request_redraw();
    }

    if mouse_in_terminal && (ui.clicked || (ui.mouse_down && ui.active_drag.is_none())) {
        state.active_terminal_idx = terminal_idx;
        ui.focused_id = Some(term_id);
        state.focused_panel = crate::state::types::FocusedPanel::Terminal;
        state.last_cursor_activity = std::time::Instant::now();
        ui.needs_redraw = true;
    }

    // ─── Mouse Wheel Scrolling for Terminal ───────────────────────
    let is_body_hovered = !is_blocked
        && !state.active_resize_terminal
        && ui.mouse_in_rect(panel_x, panel_y, panel_w, height);

    if is_body_hovered {
        let mut term_scroll_delta = 0.0;
        ui.input_events.retain(|event| {
            if let zenthra_platform::event::PlatformEvent::MouseWheel { delta_y, .. } = event {
                term_scroll_delta += *delta_y;
                false
            } else {
                true
            }
        });

        if term_scroll_delta != 0.0 {
            let now = std::time::Instant::now();
            let elapsed_ms = now
                .duration_since(state.terminal_last_scroll_time)
                .as_millis();
            // Reset accumulator if scrolling paused (> 100ms) or reversed direction
            if elapsed_ms > 100
                || (state.terminal_scroll_accum > 0.0 && term_scroll_delta < 0.0)
                || (state.terminal_scroll_accum < 0.0 && term_scroll_delta > 0.0)
            {
                state.terminal_scroll_accum = 0.0;
            }
            state.terminal_last_scroll_time = now;

            let is_alt_screen = state
                .terminal_session_at(terminal_idx)
                .map(|sess| {
                    sess.parser
                        .lock()
                        .map(|parser| parser.screen().alternate_screen())
                        .unwrap_or(false)
                })
                .unwrap_or(false);

            if is_alt_screen {
                if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                    // In alternate screen applications (vim, less, htop, nano):
                    // Standard terminal behavior: 3 lines per wheel notch
                    let count = if term_scroll_delta.abs() <= 1.5 {
                        3
                    } else {
                        ((term_scroll_delta.abs() / 18.0).round() as usize).clamp(1, 6)
                    };
                    let key_seq = if term_scroll_delta > 0.0 {
                        b"\x1b[A"
                    } else {
                        b"\x1b[B"
                    };
                    for _ in 0..count {
                        sess.write_raw(key_seq);
                    }
                    ui.needs_redraw = true;
                }
            } else {
                // Standard terminal scrolling (matches VS Code / GNOME Terminal / Alacritty):
                // Discrete mouse wheel notch: 3 lines per notch
                // Continuous touchpad: ~18px (1 text line height) per line
                let delta_lines = if term_scroll_delta.abs() <= 1.5 {
                    term_scroll_delta.signum() * 3.0
                } else {
                    term_scroll_delta / 18.0
                };

                state.terminal_scroll_accum += delta_lines;

                let whole_lines = (state.terminal_scroll_accum.trunc() as i32).clamp(-12, 12);
                if whole_lines != 0 {
                    state.terminal_scroll_accum -= whole_lines as f32;
                    if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                        if let Ok(mut p) = sess.parser.lock() {
                            let cur_sb = p.screen().scrollback();
                            // Wheel up (positive whole_lines) moves backward into scrollback history
                            let new_sb = if whole_lines > 0 {
                                cur_sb.saturating_add(whole_lines as usize)
                            } else {
                                cur_sb.saturating_sub((-whole_lines) as usize)
                            };
                            p.screen_mut().set_scrollback(new_sb);
                            ui.needs_redraw = true;
                        }
                    }
                }
            }
        }
    }

    // ─── PTY Output Streaming Poll ───────────────────────────────
    if let Some(sess) = state.terminal_session_at(terminal_idx) {
        if sess
            .has_new_output
            .swap(false, std::sync::atomic::Ordering::Relaxed)
        {
            ui.needs_redraw = true;
        }
        ui.request_redraw_after(std::time::Duration::from_millis(30));
    }

    // ─── Input Event Processing directly inside Terminal ─────────
    if is_focused && !is_blocked && !has_other_widget_focus {
        let mut events = std::mem::take(&mut ui.input_events);
        events.retain(|event| {
            match event {
                zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                    if !state.ctrl_pressed
                        && !state.alt_pressed
                        && *c != '\r'
                        && *c != '\n'
                        && *c != '\t'
                        && !c.is_control()
                    {
                        state.last_cursor_activity = std::time::Instant::now();
                        state.terminal_text_selection = None;
                        state.terminal_selection_drag = None;
                        let mut buf = [0u8; 4];
                        let bytes = c.encode_utf8(&mut buf).as_bytes();
                        if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                if p.screen().scrollback() > 0 {
                                    p.screen_mut().set_scrollback(0);
                                }
                            }
                            sess.write_raw(bytes);
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                }
                zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                    state.last_cursor_activity = std::time::Instant::now();

                    // ── 1. Clipboard & Terminal Control Actions ─────────────
                    // Ctrl+Shift+C copies the selection, or the visible terminal contents.
                    if state.ctrl_pressed
                        && state.shift_pressed
                        && *key == winit::keyboard::KeyCode::KeyC
                    {
                        let copied_text = selected_terminal_text(state, terminal_idx).or_else(|| {
                            state
                                .terminal_session_at(terminal_idx)
                                .and_then(|session| session.parser.lock().ok())
                                .map(|parser| parser.screen().contents())
                        });
                        if let Some(text) = copied_text {
                            state.set_clipboard(text);
                            state.set_status("Terminal text copied to clipboard");
                        }
                        ui.needs_redraw = true;
                        return false;
                    }

                    // Ctrl+C copies an active selection; without one it remains SIGINT.
                    if state.ctrl_pressed
                        && !state.shift_pressed
                        && !state.alt_pressed
                        && *key == winit::keyboard::KeyCode::KeyC
                    {
                        if let Some(text) = selected_terminal_text(state, terminal_idx) {
                            state.set_clipboard(text);
                            state.set_status("Terminal selection copied to clipboard");
                            ui.needs_redraw = true;
                            return false;
                        }
                    }

                    // Ctrl+Insert copies the current terminal selection.
                    if state.ctrl_pressed
                        && !state.shift_pressed
                        && *key == winit::keyboard::KeyCode::Insert
                    {
                        if let Some(text) = selected_terminal_text(state, terminal_idx) {
                            state.set_clipboard(text);
                            state.set_status("Terminal selection copied to clipboard");
                        }
                        ui.needs_redraw = true;
                        return false;
                    }

                    // Ctrl+Shift+V / Shift+Insert / Ctrl+V: Paste clipboard
                    if ((state.ctrl_pressed && state.shift_pressed)
                        || (state.shift_pressed && *key == winit::keyboard::KeyCode::Insert)
                        || (state.ctrl_pressed && !state.shift_pressed && !state.alt_pressed))
                        && (*key == winit::keyboard::KeyCode::KeyV
                            || *key == winit::keyboard::KeyCode::Insert)
                    {
                        state.terminal_text_selection = None;
                        state.terminal_selection_drag = None;
                        if let Some(text) = state.get_clipboard() {
                            if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                                sess.write_raw(text.as_bytes());
                            }
                        }
                        ui.needs_redraw = true;
                        return false;
                    }

                    // Ctrl+Shift+K: Clear terminal
                    if state.ctrl_pressed
                        && state.shift_pressed
                        && *key == winit::keyboard::KeyCode::KeyK
                    {
                        state.clear_terminal();
                        ui.needs_redraw = true;
                        return false;
                    }

                    // ── 2. Scrollback History Navigation ─────────────────────
                    // Shift+PageUp / Shift+PageDown (15 lines)
                    if state.shift_pressed
                        && !state.ctrl_pressed
                        && *key == winit::keyboard::KeyCode::PageUp
                    {
                        if let Some(sess) = state.terminal_session_at(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                let cur = p.screen().scrollback();
                                p.screen_mut().set_scrollback(cur.saturating_add(15));
                                ui.needs_redraw = true;
                            }
                        }
                        return false;
                    }
                    if state.shift_pressed
                        && !state.ctrl_pressed
                        && *key == winit::keyboard::KeyCode::PageDown
                    {
                        if let Some(sess) = state.terminal_session_at(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                let cur = p.screen().scrollback();
                                p.screen_mut().set_scrollback(cur.saturating_sub(15));
                                ui.needs_redraw = true;
                            }
                        }
                        return false;
                    }

                    // Shift+Home / Shift+End (scroll to top/bottom of scrollback)
                    if state.shift_pressed
                        && !state.ctrl_pressed
                        && *key == winit::keyboard::KeyCode::Home
                    {
                        if let Some(sess) = state.terminal_session_at(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                if !p.screen().alternate_screen() {
                                    p.screen_mut().set_scrollback(usize::MAX / 2);
                                    ui.needs_redraw = true;
                                    return false;
                                }
                            }
                        }
                    }
                    if state.shift_pressed
                        && !state.ctrl_pressed
                        && *key == winit::keyboard::KeyCode::End
                    {
                        if let Some(sess) = state.terminal_session_at(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                if !p.screen().alternate_screen() {
                                    p.screen_mut().set_scrollback(0);
                                    ui.needs_redraw = true;
                                    return false;
                                }
                            }
                        }
                    }

                    // Shift+Up / Shift+Down (scroll 1 line into scrollback in normal screen)
                    if state.shift_pressed
                        && !state.ctrl_pressed
                        && !state.alt_pressed
                        && *key == winit::keyboard::KeyCode::ArrowUp
                    {
                        if let Some(sess) = state.terminal_session_at(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                if !p.screen().alternate_screen() {
                                    let cur = p.screen().scrollback();
                                    p.screen_mut().set_scrollback(cur.saturating_add(1));
                                    ui.needs_redraw = true;
                                    return false;
                                }
                            }
                        }
                    }
                    if state.shift_pressed
                        && !state.ctrl_pressed
                        && !state.alt_pressed
                        && *key == winit::keyboard::KeyCode::ArrowDown
                    {
                        if let Some(sess) = state.terminal_session_at(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                if !p.screen().alternate_screen() {
                                    let cur = p.screen().scrollback();
                                    p.screen_mut().set_scrollback(cur.saturating_sub(1));
                                    ui.needs_redraw = true;
                                    return false;
                                }
                            }
                        }
                    }

                    // Ctrl+Up/Down navigate shell command history across terminal shells.
                    if state.ctrl_pressed && !state.shift_pressed && !state.alt_pressed {
                        if let Some(raw) = terminal_history_key_sequence(*key) {
                            state.terminal_text_selection = None;
                            state.terminal_selection_drag = None;
                            if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                                if let Ok(mut p) = sess.parser.lock() {
                                    if p.screen().scrollback() > 0 {
                                        p.screen_mut().set_scrollback(0);
                                    }
                                }
                                sess.write_raw(raw);
                            }
                            ui.needs_redraw = true;
                            return false;
                        }
                    }

                    // ── 3. Escape Sequence / Key Code Generation ─────────────
                    let mut handled = true;
                    let bytes: Option<&[u8]> =
                        match (state.ctrl_pressed, state.shift_pressed, state.alt_pressed) {
                            // ── Ctrl + Shift + Key ───────────────────────────────
                            (true, true, false) => match key {
                                winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[1;6A"),
                                winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[1;6B"),
                                winit::keyboard::KeyCode::ArrowRight => Some(b"\x1b[1;6C"),
                                winit::keyboard::KeyCode::ArrowLeft => Some(b"\x1b[1;6D"),
                                winit::keyboard::KeyCode::Home => Some(b"\x1b[1;6H"),
                                winit::keyboard::KeyCode::End => Some(b"\x1b[1;6F"),
                                winit::keyboard::KeyCode::PageUp => Some(b"\x1b[5;6~"),
                                winit::keyboard::KeyCode::PageDown => Some(b"\x1b[6;6~"),
                                winit::keyboard::KeyCode::Delete => Some(b"\x1b[3;6~"),
                                _ => {
                                    handled = false;
                                    None
                                }
                            },

                            // ── Ctrl + Key ───────────────────────────────────────
                            (true, false, false) => match key {
                                // Arrow navigation (Ctrl+Left / Ctrl+Right for word jump, Ctrl+Up/Down)
                                winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[1;5A"),
                                winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[1;5B"),
                                winit::keyboard::KeyCode::ArrowRight => Some(b"\x1b[1;5C"),
                                winit::keyboard::KeyCode::ArrowLeft => Some(b"\x1b[1;5D"),

                                // Navigation
                                winit::keyboard::KeyCode::Home => Some(b"\x1b[1;5H"),
                                winit::keyboard::KeyCode::End => Some(b"\x1b[1;5F"),
                                winit::keyboard::KeyCode::PageUp => Some(b"\x1b[5;5~"),
                                winit::keyboard::KeyCode::PageDown => Some(b"\x1b[6;5~"),

                                // Editing
                                winit::keyboard::KeyCode::Backspace => Some(b"\x17"), // Unix word rubout (^W)
                                winit::keyboard::KeyCode::Delete => Some(b"\x1b[3;5~"), // Word kill forward

                                // Control characters
                                winit::keyboard::KeyCode::KeyA => Some(b"\x01"), // SOH (start of line)
                                winit::keyboard::KeyCode::KeyB => Some(b"\x02"), // STX (cursor left)
                                winit::keyboard::KeyCode::KeyC => Some(b"\x03"), // ETX (SIGINT)
                                winit::keyboard::KeyCode::KeyD => Some(b"\x04"), // EOT (EOF)
                                winit::keyboard::KeyCode::KeyE => Some(b"\x05"), // ENQ (end of line)
                                winit::keyboard::KeyCode::KeyF => Some(b"\x06"), // ACK (cursor right)
                                winit::keyboard::KeyCode::KeyG => Some(b"\x07"), // BEL (abort search)
                                winit::keyboard::KeyCode::KeyH => Some(b"\x08"), // BS (backspace)
                                winit::keyboard::KeyCode::KeyJ => Some(b"\x0a"), // LF (newline)
                                winit::keyboard::KeyCode::KeyK => Some(b"\x0b"), // VT (kill line forward)
                                winit::keyboard::KeyCode::KeyL => Some(b"\x0c"), // FF (clear screen)
                                winit::keyboard::KeyCode::KeyN => Some(b"\x0e"), // SO (history next)
                                winit::keyboard::KeyCode::KeyO => Some(b"\x0f"), // SI (operate-and-get-next)
                                winit::keyboard::KeyCode::KeyP => Some(b"\x10"), // DLE (history prev)
                                winit::keyboard::KeyCode::KeyQ => Some(b"\x11"), // DC1 (XON)
                                winit::keyboard::KeyCode::KeyR => Some(b"\x12"), // DC2 (reverse search)
                                winit::keyboard::KeyCode::KeyS => Some(b"\x13"), // DC3 (XOFF / forward search)
                                winit::keyboard::KeyCode::KeyU => Some(b"\x15"), // NAK (kill line backward)
                                winit::keyboard::KeyCode::KeyW => Some(b"\x17"), // ETB (rubout word)
                                winit::keyboard::KeyCode::KeyX => Some(b"\x18"), // CAN
                                winit::keyboard::KeyCode::KeyY => Some(b"\x19"), // EM (yank / paste kill ring)
                                winit::keyboard::KeyCode::KeyZ => Some(b"\x1a"), // SUB (SIGTSTP)

                                _ => {
                                    handled = false;
                                    None
                                }
                            },

                            // ── Shift + Key ──────────────────────────────────────
                            (false, true, false) => match key {
                                winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[1;2A"),
                                winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[1;2B"),
                                winit::keyboard::KeyCode::ArrowRight => Some(b"\x1b[1;2C"),
                                winit::keyboard::KeyCode::ArrowLeft => Some(b"\x1b[1;2D"),
                                winit::keyboard::KeyCode::Home => Some(b"\x1b[1;2H"),
                                winit::keyboard::KeyCode::End => Some(b"\x1b[1;2F"),
                                winit::keyboard::KeyCode::Tab => Some(b"\x1b[Z"), // Backtab
                                winit::keyboard::KeyCode::Delete => Some(b"\x1b[3;2~"),
                                winit::keyboard::KeyCode::Enter => Some(b"\r"),
                                winit::keyboard::KeyCode::Backspace => Some(b"\x7f"),
                                _ => {
                                    handled = false;
                                    None
                                }
                            },

                            // ── Alt + Key ────────────────────────────────────────
                            (false, false, true) => match key {
                                winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[1;3A"),
                                winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[1;3B"),
                                winit::keyboard::KeyCode::ArrowRight => Some(b"\x1b[1;3C"),
                                winit::keyboard::KeyCode::ArrowLeft => Some(b"\x1b[1;3D"),
                                winit::keyboard::KeyCode::Home => Some(b"\x1b[1;3H"),
                                winit::keyboard::KeyCode::End => Some(b"\x1b[1;3F"),
                                winit::keyboard::KeyCode::Backspace => Some(b"\x1b\x7f"), // backward-kill-word
                                winit::keyboard::KeyCode::Delete => Some(b"\x1b[3;3~"),
                                winit::keyboard::KeyCode::KeyB => Some(b"\x1bb"), // backward word
                                winit::keyboard::KeyCode::KeyF => Some(b"\x1bf"), // forward word
                                winit::keyboard::KeyCode::KeyD => Some(b"\x1bd"), // kill word forward
                                _ => {
                                    handled = false;
                                    None
                                }
                            },

                            // ── Plain Key (No modifiers) ─────────────────────────
                            (false, false, false) => match key {
                                winit::keyboard::KeyCode::Enter => Some(b"\r"),
                                winit::keyboard::KeyCode::Backspace => Some(b"\x7f"),
                                winit::keyboard::KeyCode::Tab => Some(b"\t"),
                                winit::keyboard::KeyCode::Escape => Some(b"\x1b"),
                                winit::keyboard::KeyCode::ArrowUp => Some(b"\x1b[A"),
                                winit::keyboard::KeyCode::ArrowDown => Some(b"\x1b[B"),
                                winit::keyboard::KeyCode::ArrowRight => Some(b"\x1b[C"),
                                winit::keyboard::KeyCode::ArrowLeft => Some(b"\x1b[D"),
                                winit::keyboard::KeyCode::Home => Some(b"\x1b[H"),
                                winit::keyboard::KeyCode::End => Some(b"\x1b[F"),
                                winit::keyboard::KeyCode::Delete => Some(b"\x1b[3~"),
                                winit::keyboard::KeyCode::PageUp => Some(b"\x1b[5~"),
                                winit::keyboard::KeyCode::PageDown => Some(b"\x1b[6~"),
                                winit::keyboard::KeyCode::Insert => Some(b"\x1b[2~"),
                                _ => {
                                    handled = false;
                                    None
                                }
                            },

                            _ => {
                                handled = false;
                                None
                            }
                        };

                    if let Some(raw) = bytes {
                        state.terminal_text_selection = None;
                        state.terminal_selection_drag = None;
                        if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                            if let Ok(mut p) = sess.parser.lock() {
                                if p.screen().scrollback() > 0 {
                                    p.screen_mut().set_scrollback(0);
                                }
                            }
                            sess.write_raw(raw);
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    if handled {
                        ui.needs_redraw = true;
                        return false;
                    }
                }
                _ => {}
            }
            true
        });
        ui.input_events = events;
    }

    // ─── Main Terminal Panel Box ─────────────────────────────────
    let term_panel = ui
        .container()
        .id(term_id)
        .width(width)
        .height(height)
        .bg(colors.bg_base)
        .column()
        .show(|ui| {
            let body_h = (height - 1.0).max(40.0);
            let term_body_x = ui.cursor_x;
            let term_body_y = ui.cursor_y;

            let is_term_hovered = !is_blocked && ui.mouse_in_rect(panel_x, panel_y, width, body_h);
            if is_term_hovered {
                ui.cursor_icon = CursorIcon::Text;
                if ui.clicked || (ui.mouse_down && ui.active_drag.is_none()) {
                    ui.focused_id = Some(term_id);
                    state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                    state.last_cursor_activity = std::time::Instant::now();
                    ui.needs_redraw = true;
                }
            }

            let font_sz = (state.font_size * state.zoom_level).max(8.0);
            let (cell_w, cell_h) = if let Some(fs) = ui.font_system.as_ref() {
                let mut adapter =
                    zenthra_text::prelude::CosmicFontProvider::new_with_system(fs.clone());
                let opts = TextOptions::new().size(font_sz).font_family("monospace");
                let shaped = adapter.shape("M", &opts);
                let w = shaped.glyphs().first().map(|g| g.width).unwrap_or(0.0);
                let final_w = if w > 3.0 && w < 40.0 {
                    w
                } else {
                    (font_sz * 0.6).round().max(7.0)
                };
                let final_h = (font_sz * 1.54).round().max(14.0);
                (final_w, final_h)
            } else {
                (
                    (font_sz * 0.6).round().max(7.0),
                    (font_sz * 1.54).round().max(14.0),
                )
            };
            let pad_left = 8.0;
            let pad_top = 4.0;

            let target_cols = ((width - pad_left - 8.0) / cell_w).floor().max(20.0) as u16;
            let target_rows = ((body_h - pad_top - 4.0) / cell_h).floor().max(4.0) as u16;

            let selection_left = term_body_x + pad_left;
            let selection_top = term_body_y + pad_top;
            let selection_right = selection_left + target_cols as f32 * cell_w;
            let selection_bottom = selection_top + target_rows as f32 * cell_h;
            let mouse_in_cells = ui.mouse_x >= selection_left
                && ui.mouse_x < selection_right
                && ui.mouse_y >= selection_top
                && ui.mouse_y < selection_bottom;
            if ui.clicked
                && mouse_in_cells
                && !ui.right_clicked
                && !is_blocked
            {
                let row = ((ui.mouse_y - selection_top) / cell_h).floor() as usize;
                let column = ((ui.mouse_x - selection_left) / cell_w).floor() as usize;
                state.terminal_text_selection = Some(crate::state::types::TerminalTextSelection {
                    terminal_index: terminal_idx,
                    anchor_row: row,
                    anchor_column: column,
                    focus_row: row,
                    focus_column: column,
                });
                state.terminal_selection_drag = Some(terminal_idx);
            }
            if state.terminal_selection_drag == Some(terminal_idx) {
                if ui.mouse_down || !ui.clicked {
                    let row = ((ui.mouse_y - selection_top) / cell_h)
                        .floor()
                        .clamp(0.0, target_rows.saturating_sub(1) as f32)
                        as usize;
                    let column = ((ui.mouse_x - selection_left) / cell_w)
                        .floor()
                        .clamp(0.0, target_cols.saturating_sub(1) as f32)
                        as usize;
                    if let Some(selection) = state.terminal_text_selection.as_mut() {
                        selection.focus_row = row;
                        selection.focus_column = column;
                    }
                    ui.needs_redraw = true;
                }
                if !ui.mouse_down {
                    state.terminal_selection_drag = None;
                }
            }

            if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                let (cur_rows, cur_cols) = {
                    let p = sess.parser.lock().unwrap();
                    p.screen().size()
                };
                if cur_rows != target_rows || cur_cols != target_cols {
                    sess.resize(target_rows, target_cols);
                }
            }

            let clip = [term_body_x, term_body_y, width, body_h];

            let body_resp = ui
                .container()
                .id(Id::from_u64(999999802 + (terminal_idx as u64 * 1000)))
                .width(width)
                .height(body_h)
                .bg(colors.bg_base)
                .show(|ui| {
                    if let Some(sess) = state.terminal_session_at(terminal_idx) {
                        if let Ok(mut p) = sess.parser.lock() {
                            let mut cur_sb = p.screen().scrollback();
                            p.screen_mut().set_scrollback(usize::MAX);
                            let max_sb = p.screen().scrollback();
                            p.screen_mut().set_scrollback(cur_sb);

                            let (rows, cols) = p.screen().size();
                            let total_lines = (rows as f32) + (max_sb as f32);
                            let visible_ratio = ((rows as f32) / total_lines).clamp(0.04, 1.0);
                            let thumb_h =
                                (body_h * visible_ratio).clamp(24.0, (body_h - 4.0).max(24.0));
                            let track_h = (body_h - thumb_h).max(0.0);

                            // Handle scrollbar drag / click before borrowing screen immutably
                            let terminal_sb_id =
                                Id::from_u64(999999805 + (terminal_idx as u64 * 1000));
                            let sb_w = 5.0;
                            let sb_x = term_body_x + width - sb_w - 3.0;
                            let sb_hit_x = sb_x - 4.0;
                            let sb_hit_w = sb_w + 8.0;
                            let mouse_in_sb = ui.mouse_x >= sb_hit_x
                                && ui.mouse_x <= sb_hit_x + sb_hit_w
                                && ui.mouse_y >= term_body_y
                                && ui.mouse_y <= term_body_y + body_h;

                            if max_sb > 0 && track_h > 0.0 {
                                if let Some(drag) = ui.active_drag {
                                    if drag.id == terminal_sb_id {
                                        if ui.mouse_down {
                                            let delta_y = ui.mouse_y - drag.start_mouse;
                                            let delta_sb = (delta_y / track_h) * (max_sb as f32);
                                            let new_sb = ((drag.start_scroll - delta_sb).round()
                                                as isize)
                                                .clamp(0, max_sb as isize)
                                                as usize;
                                            p.screen_mut().set_scrollback(new_sb);
                                            cur_sb = new_sb;
                                            ui.needs_redraw = true;
                                        } else {
                                            ui.active_drag = None;
                                        }
                                    }
                                }

                                if ui.clicked && mouse_in_sb && !state.active_resize_terminal {
                                    let ratio_from_top =
                                        1.0 - (cur_sb as f32 / max_sb as f32).clamp(0.0, 1.0);
                                    let thumb_y = term_body_y + ratio_from_top * track_h;
                                    let mouse_in_thumb =
                                        ui.mouse_y >= thumb_y && ui.mouse_y <= thumb_y + thumb_h;
                                    if mouse_in_thumb {
                                        ui.active_drag = Some(ScrollDrag {
                                            id: terminal_sb_id,
                                            start_mouse: ui.mouse_y,
                                            start_scroll: cur_sb as f32,
                                        });
                                    } else {
                                        let click_ratio =
                                            ((ui.mouse_y - term_body_y - thumb_h * 0.5) / track_h)
                                                .clamp(0.0, 1.0);
                                        let target_sb =
                                            ((1.0 - click_ratio) * max_sb as f32).round() as usize;
                                        p.screen_mut().set_scrollback(target_sb);
                                        cur_sb = target_sb;
                                        ui.active_drag = Some(ScrollDrag {
                                            id: terminal_sb_id,
                                            start_mouse: ui.mouse_y,
                                            start_scroll: target_sb as f32,
                                        });
                                        ui.needs_redraw = true;
                                    }
                                }
                            }

                            let screen = p.screen();
                            let (cursor_row, cursor_col) = screen.cursor_position();
                            let hide_cursor = screen.hide_cursor();
                            let command_row = if cur_sb == 0 && !screen.alternate_screen() {
                                let mut line = String::new();
                                for column in 0..cols {
                                    let contents = screen
                                        .cell(cursor_row, column)
                                        .map(|cell| cell.contents())
                                        .unwrap_or(" ");
                                    line.push_str(if contents.is_empty() { " " } else { contents });
                                }
                                Some(line)
                            } else {
                                None
                            };

                            for r in 0..rows {
                                let row_y = term_body_y + pad_top + r as f32 * cell_h;
                                if row_y + cell_h < term_body_y || row_y > term_body_y + body_h {
                                    continue;
                                }

                                // Scan row into spans of contiguous cells with identical attributes
                                let mut cur_span_text = String::new();
                                let mut cur_span_start = 0u16;
                                let mut cur_span_fg = colors.text_primary;
                                let mut cur_span_bg: Option<Color> = None;
                                let mut cur_span_bold = false;
                                let mut in_span = false;

                                for c in 0..cols {
                                    let cell = screen.cell(r, c);
                                    let contents =
                                        cell.map(|cell_ref| cell_ref.contents()).unwrap_or(" ");
                                    let mut fg = cell
                                        .map(|cell_ref| {
                                            convert_vt_color(
                                                cell_ref.fgcolor(),
                                                colors.text_primary,
                                            )
                                        })
                                        .unwrap_or(colors.text_primary);
                                    if r == cursor_row {
                                        if let Some(highlight_color) =
                                            command_row.as_deref().and_then(|line| {
                                                terminal_command_color(
                                                    line,
                                                    c as usize,
                                                    colors.text_primary,
                                                )
                                            })
                                        {
                                            fg = highlight_color;
                                        }
                                    }
                                    let bg = cell.and_then(|cell_ref| match cell_ref.bgcolor() {
                                        vt100::Color::Default => None,
                                        other => Some(convert_vt_color(other, Color::TRANSPARENT)),
                                    });
                                    let bold =
                                        cell.map(|cell_ref| cell_ref.bold()).unwrap_or(false);

                                    // Handle inverse video
                                    let (fg, bg) =
                                        if cell.map(|cell_ref| cell_ref.inverse()).unwrap_or(false)
                                        {
                                            (bg.unwrap_or(colors.bg_base), Some(fg))
                                        } else {
                                            (fg, bg)
                                        };
                                    let (fg, bg) = if state
                                        .terminal_text_selection
                                        .filter(|selection| {
                                            selection.terminal_index == terminal_idx
                                                && terminal_cell_is_selected(
                                                    *selection,
                                                    r as usize,
                                                    c as usize,
                                                )
                                        })
                                        .is_some()
                                    {
                                        (
                                            Color::WHITE,
                                            Some(Color::rgb(0.20, 0.38, 0.65)),
                                        )
                                    } else {
                                        (fg, bg)
                                    };

                                    if in_span
                                        && (fg != cur_span_fg
                                            || bg != cur_span_bg
                                            || bold != cur_span_bold)
                                    {
                                        // Flush current span
                                        let span_x =
                                            term_body_x + pad_left + cur_span_start as f32 * cell_w;
                                        let span_w = (c - cur_span_start) as f32 * cell_w;
                                        if let Some(bg_c) = cur_span_bg {
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [span_x, row_y],
                                                    size: [span_w, cell_h],
                                                    color: bg_c.to_array(),
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                },
                                            }));
                                        }
                                        if !cur_span_text.trim().is_empty() {
                                            let mut opts = TextOptions::new()
                                                .size(font_sz)
                                                .scale_factor(ui.scale_factor)
                                                .font_family("monospace")
                                                .color(cur_span_fg);
                                            if cur_span_bold {
                                                opts = opts.font_weight(FontWeight::Bold);
                                            }
                                            ui.draws.push(DrawCommand::Text(TextDraw {
                                                text: std::mem::take(&mut cur_span_text),
                                                pos: [span_x, row_y + 1.0],
                                                options: opts,
                                                clip,
                                            }));
                                        } else {
                                            cur_span_text.clear();
                                        }
                                        in_span = false;
                                    }

                                    if !in_span {
                                        cur_span_start = c;
                                        cur_span_fg = fg;
                                        cur_span_bg = bg;
                                        cur_span_bold = bold;
                                        in_span = true;
                                    }

                                    cur_span_text.push_str(if contents.is_empty() {
                                        " "
                                    } else {
                                        contents
                                    });
                                }

                                if in_span {
                                    let span_x =
                                        term_body_x + pad_left + cur_span_start as f32 * cell_w;
                                    let span_w = (cols - cur_span_start) as f32 * cell_w;
                                    if let Some(bg_c) = cur_span_bg {
                                        ui.draws.push(DrawCommand::Rect(RectDraw {
                                            instance: RectInstance {
                                                pos: [span_x, row_y],
                                                size: [span_w, cell_h],
                                                color: bg_c.to_array(),
                                                clip_rect: clip,
                                                ..Default::default()
                                            },
                                        }));
                                    }
                                    if !cur_span_text.trim().is_empty() {
                                        let mut opts = TextOptions::new()
                                            .size(font_sz)
                                            .scale_factor(ui.scale_factor)
                                            .font_family("monospace")
                                            .color(cur_span_fg);
                                        if cur_span_bold {
                                            opts = opts.font_weight(FontWeight::Bold);
                                        }
                                        ui.draws.push(DrawCommand::Text(TextDraw {
                                            text: cur_span_text,
                                            pos: [span_x, row_y + 1.0],
                                            options: opts,
                                            clip,
                                        }));
                                    }
                                }
                            }

                            let is_scrolled_back = cur_sb > 0;
                            let has_text_selection = state
                                .terminal_text_selection
                                .map(|selection| {
                                    selection.terminal_index == terminal_idx
                                        && (selection.anchor_row, selection.anchor_column)
                                            != (selection.focus_row, selection.focus_column)
                                })
                                .unwrap_or(false);

                            if is_focused
                                && !hide_cursor
                                && !is_scrolled_back
                                && !has_text_selection
                                && rows > 0
                                && cols > 0
                            {
                                let elapsed_ms =
                                    state.last_cursor_activity.elapsed().as_millis();
                                let show_cursor = if elapsed_ms < 500 {
                                    true
                                } else {
                                    ((elapsed_ms - 500) / 530) % 2 == 0
                                };
                                ui.request_redraw_after(std::time::Duration::from_millis(530));
                                if show_cursor {
                                    let safe_row = cursor_row.min(rows.saturating_sub(1));
                                    let safe_col = cursor_col.min(cols.saturating_sub(1));
                                    let cursor_color = state.get_cursor_color(colors.accent);
                                    let row_y = term_body_y + pad_top + safe_row as f32 * cell_h;
                                    let cur_x = term_body_x + pad_left + safe_col as f32 * cell_w;

                                    // Line cursor matches text glyph height cleanly down to baseline
                                    let line_cursor_h = (font_sz * 0.98).round().max(13.0);
                                    let line_cur_y = row_y + 1.5;

                                    match state.cursor_style {
                                        crate::state::types::CursorStyle::Line => {
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [cur_x, line_cur_y],
                                                    size: [2.0, line_cursor_h],
                                                    color: cursor_color.to_array(),
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                },
                                            }));
                                        }
                                        crate::state::types::CursorStyle::Block => {
                                            let block_h = (font_sz + 1.5).round().max(14.0);
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [cur_x, row_y + 1.0],
                                                    size: [cell_w.max(2.0), block_h],
                                                    color: cursor_color.to_array(),
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                },
                                            }));

                                            // Render character under block cursor in inverted high-contrast color
                                            if let Some(cell_ref) = screen.cell(safe_row, safe_col) {
                                                let contents = cell_ref.contents();
                                                if !contents.trim().is_empty() {
                                                    let inv_color = if cursor_color.r * 0.299 + cursor_color.g * 0.587 + cursor_color.b * 0.114 > 0.5 {
                                                        colors.bg_base
                                                    } else {
                                                        Color::rgb(1.0, 1.0, 1.0)
                                                    };
                                                    let opts = TextOptions::new()
                                                        .size(font_sz)
                                                        .scale_factor(ui.scale_factor)
                                                        .font_family("monospace")
                                                        .color(inv_color);
                                                    ui.draws.push(DrawCommand::Text(TextDraw {
                                                        text: contents.to_string(),
                                                        pos: [cur_x, row_y + 1.0],
                                                        options: opts,
                                                        clip,
                                                    }));
                                                }
                                            }
                                        }
                                        crate::state::types::CursorStyle::Underline => {
                                            let under_y = line_cur_y + line_cursor_h - 1.0;
                                            ui.draws.push(DrawCommand::Rect(RectDraw {
                                                instance: RectInstance {
                                                    pos: [cur_x, under_y],
                                                    size: [cell_w.max(2.0), 2.0],
                                                    color: cursor_color.to_array(),
                                                    clip_rect: clip,
                                                    ..Default::default()
                                                },
                                            }));
                                        }
                                    }
                                }
                            }

                            // Show sleek scrollbar indicator if scrollback history exists
                            if max_sb > 0 {
                                let ratio_from_top =
                                    1.0 - (cur_sb as f32 / max_sb as f32).clamp(0.0, 1.0);
                                let thumb_y = term_body_y + ratio_from_top * track_h;

                                // Subtle scrollbar track background
                                ui.draws.push(DrawCommand::Rect(RectDraw {
                                    instance: RectInstance {
                                        pos: [sb_x, term_body_y],
                                        size: [sb_w, body_h],
                                        color: [1.0, 1.0, 1.0, 0.04],
                                        radius: [2.5; 4],
                                        clip_rect: clip,
                                        ..Default::default()
                                    },
                                }));

                                // Thumb appearance: vibrant accent when scrolled back, subtle muted when hovered at prompt
                                let thumb_color = if is_scrolled_back {
                                    colors.accent.with_alpha(0.75).to_array()
                                } else if is_body_hovered {
                                    colors.text_muted.with_alpha(0.35).to_array()
                                } else {
                                    [0.0; 4]
                                };

                                if thumb_color[3] > 0.0 {
                                    ui.draws.push(DrawCommand::Rect(RectDraw {
                                        instance: RectInstance {
                                            pos: [sb_x, thumb_y],
                                            size: [sb_w, thumb_h],
                                            color: thumb_color,
                                            radius: [2.5; 4],
                                            clip_rect: clip,
                                            ..Default::default()
                                        },
                                    }));
                                }
                            }
                        }
                    } else {
                        ui.text("Initializing terminal...")
                            .size(11.0)
                            .font_family("monospace")
                            .color(colors.text_muted)
                            .show();
                    }
                });

            if body_resp.clicked {
                ui.focused_id = Some(term_id);
                state.focused_panel = crate::state::types::FocusedPanel::Terminal;
                state.last_cursor_activity = std::time::Instant::now();
                ui.needs_redraw = true;
            }
        });

    if term_panel.clicked {
        ui.focused_id = Some(term_id);
        state.focused_panel = crate::state::types::FocusedPanel::Terminal;
        state.last_cursor_activity = std::time::Instant::now();
        ui.needs_redraw = true;
    }
}

#[derive(Clone, Copy)]
enum TerminalContextAction {
    CopyAll,
    Paste,
    Clear,
    File(crate::state::FileContextAction),
}

fn draw_terminal_context_menu(ui: &mut Ui, state: &mut ZenthreeState, colors: &ThemeColors) {
    let Some(menu) = state.panel_context_menu.clone() else {
        return;
    };
    if menu.target != crate::state::PanelContextMenuTarget::Terminal {
        return;
    }

    let menu_w = 210.0;
    let menu_h = 13.0 * 26.0 + 5.0 + 17.0 + 8.0;
    let menu_x = menu.menu_x.clamp(8.0, (ui.width - menu_w - 8.0).max(8.0));
    let menu_y = menu.menu_y.clamp(8.0, (ui.height - menu_h - 8.0).max(8.0));
    let mouse_in_menu = ui.mouse_in_rect(menu_x, menu_y, menu_w, menu_h);
    let escape_pressed = ui.input_events.iter().any(|event| {
        matches!(
            event,
            zenthra_platform::event::PlatformEvent::KeyDown {
                key: winit::keyboard::KeyCode::Escape
            }
        )
    });
    if escape_pressed {
        state.panel_context_menu = None;
        ui.input_events.retain(|event| {
            !matches!(
                event,
                zenthra_platform::event::PlatformEvent::KeyDown {
                    key: winit::keyboard::KeyCode::Escape
                }
            )
        });
        ui.request_redraw();
        return;
    }
    if (ui.clicked || ui.right_clicked) && !mouse_in_menu {
        state.panel_context_menu = None;
        ui.consume_click();
        ui.consume_right_click();
        ui.request_redraw();
        return;
    }

    let target_path = menu.target_path.clone();
    let target_is_dir = menu.target_is_dir;
    let terminal_idx = menu.terminal_index.unwrap_or(state.active_terminal_idx);
    let term_id = terminal_widget_id(terminal_idx);
    let has_terminal = state.terminal_session_at(terminal_idx).is_some();
    let root = state.current_dir.clone();
    let has_file_target = target_path != root;
    let has_file_clipboard = state.tree_clipboard.is_some();
    let mut action = None;
    ui.overlay(|ui| {
        ui.container()
            .raw_id(Id::from_u64(999997300))
            .pos(menu_x, menu_y)
            .overlay()
            .width(menu_w)
            .bg(colors.bg_sidebar)
            .border((colors.border, 1.0))
            .radius_all(6.0)
            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.22), 0.0, 2.0, 8.0)
            .padding(4.0, 4.0, 4.0, 4.0)
            .column()
            .gap(1.0)
            .show(|ui| {
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997301),
                    "\u{f018f}",
                    "Copy All",
                    "",
                    has_terminal,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::CopyAll);
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997302),
                    "\u{f0192}",
                    "Paste",
                    "Ctrl+V",
                    true,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::Paste);
                }
                ui.container()
                    .fill_x()
                    .height(1.0)
                    .bg(colors.border)
                    .show(|_| {});
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997303),
                    "\u{f12d}",
                    "Clear Terminal",
                    "",
                    has_terminal,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::Clear);
                }
                ui.container()
                    .fill_x()
                    .height(1.0)
                    .bg(colors.border)
                    .show(|_| {});
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997304),
                    "\u{ea7f}",
                    "New File...",
                    "",
                    true,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::NewFile,
                    ));
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997305),
                    "\u{ea80}",
                    "New Folder...",
                    "",
                    state.has_workspace,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::NewFolder,
                    ));
                }
                ui.container()
                    .fill_x()
                    .height(1.0)
                    .bg(colors.border)
                    .show(|_| {});
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997306),
                    "\u{f044}",
                    "Rename File...",
                    "F2",
                    has_file_target,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::Rename,
                    ));
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997307),
                    "\u{f014}",
                    "Delete File",
                    "Del",
                    has_file_target,
                    true,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::Delete,
                    ));
                }
                ui.container()
                    .fill_x()
                    .height(1.0)
                    .bg(colors.border)
                    .show(|_| {});
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997308),
                    "\u{f0c4}",
                    "Cut File",
                    "",
                    has_file_target,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::Cut,
                    ));
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997309),
                    "\u{f0c5}",
                    "Copy File",
                    "",
                    has_file_target,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::Copy,
                    ));
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997310),
                    "\u{f0ea}",
                    "Paste File",
                    "",
                    has_file_clipboard,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::Paste,
                    ));
                }
                ui.container()
                    .fill_x()
                    .height(1.0)
                    .bg(colors.border)
                    .show(|_| {});
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997311),
                    "\u{f0c1}",
                    "Copy Path",
                    "",
                    true,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::CopyPath,
                    ));
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997312),
                    "⎘",
                    "Copy Relative Path",
                    "",
                    true,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::CopyRelativePath,
                    ));
                }
                if super::sidebar::draw_menu_item(
                    ui,
                    Id::from_u64(999997313),
                    "\u{f08e}",
                    "Reveal in File Manager",
                    "",
                    true,
                    false,
                    colors,
                ) {
                    action = Some(TerminalContextAction::File(
                        crate::state::FileContextAction::RevealInFileManager,
                    ));
                }
            });
    });

    if let Some(action) = action {
        state.panel_context_menu = None;
        match action {
            TerminalContextAction::File(file_action) => {
                state.run_file_context_action(file_action, &target_path, target_is_dir);
                if matches!(
                    file_action,
                    crate::state::FileContextAction::NewFile
                        | crate::state::FileContextAction::NewFolder
                ) {
                    state.focused_panel = crate::state::FocusedPanel::Explorer;
                    ui.focused_id = None;
                } else {
                    ui.focused_id = Some(term_id);
                    state.focused_panel = crate::state::FocusedPanel::Terminal;
                }
            }
            action => {
                ui.focused_id = Some(term_id);
                state.focused_panel = crate::state::FocusedPanel::Terminal;
                match action {
                    TerminalContextAction::CopyAll => {
                        let contents = state.terminal_session_at(terminal_idx).map(|sess| {
                            sess.parser
                                .lock()
                                .map(|parser| parser.screen().contents())
                                .map_err(|_| ())
                        });
                        match contents {
                            Some(Ok(text)) => {
                                state.set_clipboard(text);
                                state.set_status("Terminal output copied to clipboard");
                            }
                            Some(Err(_)) => state.set_status("Failed to copy terminal output"),
                            None => {}
                        }
                    }
                    TerminalContextAction::Paste => {
                        if let Some(text) = state.get_clipboard() {
                            if let Some(sess) = state.terminal_session_at_mut(terminal_idx) {
                                sess.write_raw(text.as_bytes());
                            }
                            state.last_cursor_activity = std::time::Instant::now();
                        }
                    }
                    TerminalContextAction::Clear => {
                        state.clear_terminal();
                        state.last_cursor_activity = std::time::Instant::now();
                    }
                    TerminalContextAction::File(_) => unreachable!(),
                }
            }
        }
        ui.request_redraw();
    }
}
