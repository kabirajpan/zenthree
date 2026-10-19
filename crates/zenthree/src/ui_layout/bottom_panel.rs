use crate::state::{BottomPanelTab, ZenthreeState};
use crate::theme::ThemeColors;
use zenthra::{use_context, Align, CursorIcon, FontWeight, Style, Ui};
use zenthra_render::RectInstance;
use zenthra_widgets::ui::{DrawCommand, RectDraw};

const TAB_BAR_HEIGHT: f32 = 30.0;
const PANEL_RESIZE_HEIGHT: f32 = 6.0;

pub fn draw_bottom_panel(ui: &mut Ui, state: &mut ZenthreeState, width: f32, height: f32) {
    let colors = use_context::<ThemeColors>()
        .map(|colors| *colors)
        .unwrap_or_else(|| state.colors());

    ui.container()
        .width(width)
        .height(height)
        .column()
        .show(|ui| {
            draw_panel_resize_handle(ui, state, width, &colors);
            draw_panel_tabs(ui, state, width, &colors);
            let content_height = (height - TAB_BAR_HEIGHT - PANEL_RESIZE_HEIGHT).max(1.0);
            match state.bottom_panel_tab {
                BottomPanelTab::Problems => {
                    draw_problems(ui, state, width, content_height, &colors)
                }
                BottomPanelTab::Output => draw_output(ui, state, width, content_height, &colors),
                BottomPanelTab::DebugConsole => {
                    draw_debug_console(ui, width, content_height, &colors)
                }
                BottomPanelTab::Terminal => {
                    crate::ui_layout::draw_terminal_panel(ui, state, width, content_height)
                }
                BottomPanelTab::Ports => draw_ports(ui, state, width, content_height, &colors),
            }
        });
}

fn draw_panel_resize_handle(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    width: f32,
    colors: &ThemeColors,
) {
    let handle_x = ui.cursor_x;
    let handle_y = ui.cursor_y;
    let is_blocked = ui.modal_active || state.is_modal_or_dialog_open();
    let is_hovered = !is_blocked
        && ui.mouse_in_rect(
            handle_x,
            handle_y - PANEL_RESIZE_HEIGHT / 2.0,
            width,
            PANEL_RESIZE_HEIGHT,
        );

    if is_blocked {
        state.active_resize_terminal = false;
    } else if is_hovered && ui.mouse_down && ui.active_drag.is_none() {
        state.active_resize_terminal = true;
    }

    if state.active_resize_terminal {
        ui.cursor_icon = CursorIcon::RowResize;
        ui.resize_active = true;
        if ui.mouse_down {
            let status_bar_height = if state.status_bar_visible { 24.0 } else { 0.0 };
            let panel_bottom = ui.height - status_bar_height;
            state.terminal_height = (panel_bottom - ui.mouse_y)
                .clamp(70.0, (ui.height * 0.7).max(100.0));
            ui.request_redraw();
        } else {
            state.active_resize_terminal = false;
            state.save_persistent_state();
        }
    } else if is_hovered {
        ui.cursor_icon = CursorIcon::RowResize;
    }

    let line_color = if state.active_resize_terminal {
        colors.accent
    } else if is_hovered {
        colors.accent.with_alpha(0.8)
    } else {
        colors.border
    };
    ui.draws.push(DrawCommand::Rect(RectDraw {
        instance: RectInstance {
            pos: [handle_x, handle_y],
            size: [width, if is_hovered || state.active_resize_terminal { 2.0 } else { 1.0 }],
            color: line_color.to_array(),
            clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
            ..Default::default()
        },
    }));

    if is_hovered || state.active_resize_terminal {
        ui.draws.push(DrawCommand::Rect(RectDraw {
            instance: RectInstance {
                pos: [handle_x + (width - 36.0) / 2.0, handle_y + 1.0],
                size: [36.0, 4.0],
                color: colors.accent.to_array(),
                radius: [2.0; 4],
                clip_rect: [-10000.0, -10000.0, 20000.0, 20000.0],
                ..Default::default()
            },
        }));
    }

    ui.container()
        .width(width)
        .height(PANEL_RESIZE_HEIGHT)
        .show(|_| {});
}

fn draw_panel_tabs(ui: &mut Ui, state: &mut ZenthreeState, width: f32, colors: &ThemeColors) {
    ui.container()
        .width(width)
        .height(TAB_BAR_HEIGHT)
        .bg(colors.bg_sidebar)
        .row()
        .valign(Align::Center)
        .halign(Align::SpaceBetween)
        .padding_left(8.0)
        .padding_right(6.0)
        .show(|ui| {
            ui.container().row().valign(Align::Center).gap(2.0).show(|ui| {
                for (tab, label) in [
                    (BottomPanelTab::Problems, "PROBLEMS"),
                    (BottomPanelTab::Output, "OUTPUT"),
                    (BottomPanelTab::DebugConsole, "DEBUG CONSOLE"),
                    (BottomPanelTab::Terminal, "TERMINAL"),
                    (BottomPanelTab::Ports, "PORTS"),
                ] {
                    let tab_label = if tab == BottomPanelTab::Problems {
                        let (errors, warnings) = workspace_diagnostic_counts(state);
                        format!("{label}  {}", errors + warnings)
                    } else {
                        label.to_string()
                    };
                    let selected = state.bottom_panel_tab == tab;
                    let response = ui
                        .container()
                        .height(26.0)
                        .padding(4.0, 8.0, 4.0, 8.0)
                        .radius_all(3.0)
                        .bg(if selected {
                            colors.bg_active_tab
                        } else {
                            colors.bg_sidebar
                        })
                        .hover(Style::bg(colors.highlight))
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(&tab_label)
                                .size(10.0)
                                .font_weight(if selected {
                                    FontWeight::Bold
                                } else {
                                    FontWeight::Regular
                                })
                                .color(if selected {
                                    colors.text_primary
                                } else {
                                    colors.text_muted
                                })
                                .show();
                        });
                    if response.clicked {
                        state.bottom_panel_tab = tab;
                        if tab == BottomPanelTab::Ports {
                            state.refresh_listening_ports();
                        }
                        if tab == BottomPanelTab::Terminal {
                            state.focused_panel = crate::state::FocusedPanel::Terminal;
                            ui.focused_id = Some(zenthra::Id::from_u64(
                                999999800 + state.active_terminal_idx as u64 * 1000,
                            ));
                        } else if state.focused_panel == crate::state::FocusedPanel::Terminal {
                            state.focused_panel = crate::state::FocusedPanel::Editor;
                            ui.focused_id = state
                                .active_tab
                                .map(|index| zenthra::Id::from_u64(999996000 + index as u64));
                        }
                        ui.request_redraw();
                    }
                }
            });

            if state.bottom_panel_tab == BottomPanelTab::Terminal {
                let session_list_width = (width * 0.16).clamp(145.0, 190.0).min(width * 0.4);
                ui.container()
                    .row()
                    .show(|ui| {
                        crate::ui_layout::terminal::draw_terminal_toolbar(ui, state, colors);
                        ui.container()
                            .width(session_list_width)
                            .height(TAB_BAR_HEIGHT)
                            .padding_left(10.0)
                            .padding_right(6.0)
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .show(|ui| {
                                ui.text("TERMINALS")
                                    .size(10.0)
                                    .font_weight(FontWeight::Bold)
                                    .color(colors.text_muted)
                                    .show();
                                let new_button = ui
                                    .container()
                                    .width(22.0)
                                    .height(22.0)
                                    .radius_all(3.0)
                                    .hover(Style::bg(colors.highlight))
                                    .halign(Align::Center)
                                    .valign(Align::Center)
                                    .show(|ui| {
                                        ui.text("+").size(16.0).color(colors.text_primary).show();
                                    });
                                if new_button.clicked {
                                    if state.create_terminal_session() {
                                        state.terminal_split = false;
                                        state.focused_panel = crate::state::FocusedPanel::Terminal;
                                        ui.focused_id = Some(zenthra::Id::from_u64(
                                            999999800 + state.active_terminal_idx as u64 * 1000,
                                        ));
                                    }
                                    ui.request_redraw();
                                }
                            });
                    });
            }
        });
}

fn workspace_diagnostic_counts(state: &ZenthreeState) -> (usize, usize) {
    state
        .lsp_manager
        .diagnostics
        .iter()
        .filter(|(path, _)| path.starts_with(&state.current_dir))
        .flat_map(|(_, diagnostics)| diagnostics)
        .fold((0, 0), |(errors, warnings), diagnostic| {
            match diagnostic.severity {
                Some(crate::lsp::protocol::DiagnosticSeverity::Error) => (errors + 1, warnings),
                Some(crate::lsp::protocol::DiagnosticSeverity::Warning) => (errors, warnings + 1),
                _ => (errors, warnings),
            }
        })
}

fn draw_problems(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    width: f32,
    height: f32,
    colors: &ThemeColors,
) {
    let mut diagnostics: Vec<_> = state
        .lsp_manager
        .diagnostics
        .iter()
        .filter(|(path, _)| path.starts_with(&state.current_dir))
        .flat_map(|(path, items)| {
            items
                .iter()
                .cloned()
                .map(|diagnostic| (path.clone(), diagnostic))
                .collect::<Vec<_>>()
        })
        .collect();
    diagnostics.sort_by(|(left_path, left), (right_path, right)| {
        left_path
            .cmp(right_path)
            .then(left.range.start.line.cmp(&right.range.start.line))
            .then(left.range.start.character.cmp(&right.range.start.character))
    });

    if diagnostics.is_empty() {
        draw_message(ui, width, height, "No problems detected", colors);
        return;
    }

    ui.container()
        .width(width)
        .height(height)
        .bg(colors.bg_base)
        .column()
        .show(|ui| {
            for (path, diagnostic) in diagnostics {
                let severity = diagnostic.severity;
                let color = match severity {
                    Some(crate::lsp::protocol::DiagnosticSeverity::Error) => {
                        zenthra::Color::rgb(0.92, 0.29, 0.29)
                    }
                    Some(crate::lsp::protocol::DiagnosticSeverity::Warning) => {
                        zenthra::Color::rgb(0.91, 0.67, 0.20)
                    }
                    _ => colors.text_muted,
                };
                let file_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or_else(|| path.to_str().unwrap_or("Unknown file"));
                let line = diagnostic.range.start.line + 1;
                let location = format!("{file_name}:{line}");
                let row = ui
                    .container()
                    .width(width)
                    .height(25.0)
                    .padding(4.0, 8.0, 4.0, 8.0)
                    .row()
                    .gap(10.0)
                    .valign(Align::Center)
                    .hover(zenthra::Style::bg(colors.highlight))
                    .show(|ui| {
                        ui.text(match severity {
                            Some(crate::lsp::protocol::DiagnosticSeverity::Error) => "●",
                            Some(crate::lsp::protocol::DiagnosticSeverity::Warning) => "▲",
                            _ => "·",
                        })
                        .size(10.0)
                        .color(color)
                        .show();
                        ui.text(&location)
                            .size(10.5)
                            .color(colors.text_muted)
                            .show();
                        ui.text(&diagnostic.message)
                            .size(11.0)
                            .color(colors.text_primary)
                            .show();
                    });
                if row.clicked {
                    state.open_file(&path);
                    if let Some(tab_index) = state.active_tab {
                        if let Some(tab) = state.tabs.get_mut(tab_index) {
                            let row_index = diagnostic.range.start.line as usize;
                            tab.editor.set_cursor(::editor::buffer::Point::new(
                                row_index,
                                diagnostic.range.start.character as usize,
                            ));
                            let line_height = (state.font_size * state.zoom_level * 1.54)
                                .round()
                                .max(14.0);
                            tab.scroll_y = row_index as f32 * line_height;
                            tab.target_scroll_y = tab.scroll_y;
                        }
                        state.focused_panel = crate::state::FocusedPanel::Editor;
                        ui.focused_id = Some(zenthra::Id::from_u64(999996000 + tab_index as u64));
                    }
                    state.set_status(format!("Problem at {}:{}", path.display(), line));
                }
            }
        });
}

fn draw_output(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    width: f32,
    height: f32,
    colors: &ThemeColors,
) {
    ui.container()
        .width(width)
        .height(height)
        .bg(colors.bg_base)
        .column()
        .show(|ui| {
            let clear = ui
                .container()
                .height(28.0)
                .padding(5.0, 10.0, 5.0, 10.0)
                .radius_all(3.0)
                .hover(zenthra::Style::bg(colors.highlight))
                .show(|ui| {
                    ui.text("Clear Output")
                        .size(10.5)
                        .color(colors.text_primary)
                        .show();
                });
            if clear.clicked {
                state.output_lines.clear();
                ui.request_redraw();
            }
            for line in state.output_lines.iter().rev().take(40).rev() {
                ui.text(line).size(11.0).color(colors.text_primary).show();
            }
        });
}

fn draw_debug_console(ui: &mut Ui, width: f32, height: f32, colors: &ThemeColors) {
    draw_message(
        ui,
        width,
        height,
        "No debug session is active. Debugger integration is not available yet.",
        colors,
    );
}

fn draw_ports(
    ui: &mut Ui,
    state: &mut ZenthreeState,
    width: f32,
    height: f32,
    colors: &ThemeColors,
) {
    ui.container()
        .width(width)
        .height(height)
        .bg(colors.bg_base)
        .column()
        .show(|ui| {
            let refresh = ui
                .container()
                .height(28.0)
                .padding(5.0, 10.0, 5.0, 10.0)
                .radius_all(3.0)
                .hover(zenthra::Style::bg(colors.highlight))
                .show(|ui| {
                    ui.text("Refresh")
                        .size(10.5)
                        .color(colors.text_primary)
                        .show();
                });
            if refresh.clicked {
                state.refresh_listening_ports();
                ui.request_redraw();
            }
            if let Some(error) = &state.ports_error {
                ui.text(error)
                    .size(11.0)
                    .color(zenthra::Color::rgb(0.92, 0.29, 0.29))
                    .show();
            } else if state.listening_ports.is_empty() {
                ui.text("No local listening ports detected")
                    .size(11.0)
                    .color(colors.text_muted)
                    .show();
            } else {
                for port in &state.listening_ports {
                    let process = port
                        .process_id
                        .map(|id| format!("PID {id}"))
                        .unwrap_or_else(|| "Process unavailable".to_string());
                    ui.container()
                        .height(24.0)
                        .padding(4.0, 10.0, 4.0, 10.0)
                        .row()
                        .gap(14.0)
                        .valign(Align::Center)
                        .show(|ui| {
                            ui.text(&format!("{}:{}", port.address, port.port))
                                .size(11.0)
                                .color(colors.text_primary)
                                .show();
                            ui.text(&process).size(10.5).color(colors.text_muted).show();
                        });
                }
            }
            ui.text("Lists local listeners; remote port forwarding is not supported.")
                .size(9.5)
                .color(colors.text_dim)
                .show();
        });
}

fn draw_message(ui: &mut Ui, width: f32, height: f32, message: &str, colors: &ThemeColors) {
    ui.container()
        .width(width)
        .height(height)
        .bg(colors.bg_base)
        .padding(12.0, 12.0, 8.0, 12.0)
        .show(|ui| {
            ui.text(message).size(11.0).color(colors.text_muted).show();
        });
}
