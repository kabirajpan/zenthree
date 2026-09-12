mod state;
mod theme;
mod ui_layout;
mod assets;
mod neovim_icons;
mod mini_icons;
pub mod native_terminal;
pub mod extensions;

use state::ZenthreeState;
use theme::ThemeMode;
use zenthra::prelude::*;
use zenthra::Id;

fn main() {
    env_logger::init();

    // Initialize the modular state (loads persistent state from disk)
    let mut state = ZenthreeState::new();

    // Start with one default empty file only if no files were restored from previous session
    if state.tabs.is_empty() {
        state.new_untitled_file();
    }

    let font_bytes = include_bytes!("../../../assets/fonts/SymbolsNerdFont-Regular.ttf").to_vec();

    App::new()
        .title("Zenthree Editor")
        .size(1100, 680)
        .decorations(false)
        .load_font_data(font_bytes)
        .with_ui(move |ui| {
            // Track global modifier key states from input events
            for event in &ui.input_events {
                match event {
                    zenthra_platform::event::PlatformEvent::KeyDown { key } => match key {
                        winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                            state.shift_pressed = true;
                        }
                        winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                            state.ctrl_pressed = true;
                        }
                        winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
                            state.alt_pressed = true;
                        }
                        _ => {}
                    },
                    zenthra_platform::event::PlatformEvent::KeyUp { key } => match key {
                        winit::keyboard::KeyCode::ShiftLeft | winit::keyboard::KeyCode::ShiftRight => {
                            state.shift_pressed = false;
                        }
                        winit::keyboard::KeyCode::ControlLeft | winit::keyboard::KeyCode::ControlRight => {
                            state.ctrl_pressed = false;
                        }
                        winit::keyboard::KeyCode::AltLeft | winit::keyboard::KeyCode::AltRight => {
                            state.alt_pressed = false;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            }

            // Expose the active theme flag to interaction_state so widgets can read it
            let theme_val = if state.theme == ThemeMode::Light { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999999), theme_val);

            let colors = state.colors();

            // Provide reactive ThemeColors to the UI tree via Context API
            provide_context(colors);

            // Main Background Container: vertical layout
            ui.container()
                .fill()
                .bg(colors.bg_base)
                .column()
                .show(|ui| {
                    // 1. Draw Title Bar
                    ui_layout::draw_title_bar(ui, &mut state);

                    // 2. Middle Workspace Area (Sidebar + Editor + Terminal + AI Panel)
                    let ui_height = ui.height;
                    let ui_width = ui.width;
                    let status_bar_h = if state.status_bar_visible { 24.0 } else { 0.0 };
                    let middle_h = ui_height - 36.0 - status_bar_h;

                    let activity_bar_w = ui_layout::ACTIVITY_BAR_WIDTH;
                    let actual_sb_w = if state.sidebar_visible {
                        state.sidebar_width.min(ui_width * 0.4).max(140.0)
                    } else {
                        0.0
                    };

                    let min_editor_w = 160.0;
                    let avail_for_ai = (ui_width - activity_bar_w - actual_sb_w - min_editor_w).max(160.0);
                    let actual_ai_w = if state.ai_panel_visible {
                        state.ai_panel_width.min(avail_for_ai).max(180.0)
                    } else {
                        0.0
                    };

                    if state.ai_panel_visible {
                        state.ai_panel_width = actual_ai_w;
                    }
                    if state.sidebar_visible {
                        state.sidebar_width = actual_sb_w;
                    }

                    let center_w = (ui_width - activity_bar_w - actual_sb_w - actual_ai_w).max(min_editor_w);

                    let term_h = if state.terminal_visible {
                        state.terminal_height.min(middle_h - 80.0).max(60.0)
                    } else {
                        0.0
                    };
                    let editor_h = middle_h - term_h;

                    ui.container()
                        .row()
                        .fill_x()
                        .height(middle_h)
                        .show(|ui| {
                            // Far Left: Activity Bar (Files, Extensions, Settings)
                            ui_layout::draw_activity_bar(ui, &mut state, middle_h);

                            // Left: Sidebar / File Tree or Extensions Panel
                            if state.sidebar_visible {
                                ui_layout::draw_sidebar(ui, &mut state);
                            }

                            ui.container()
                                .width(center_w)
                                .height(middle_h)
                                .column()
                                .show(|ui| {
                                    ui_layout::draw_main_buffer(ui, &mut state, center_w, editor_h);

                                    if state.terminal_visible {
                                        ui_layout::draw_terminal_panel(ui, &mut state, center_w, term_h);
                                    }
                                });

                            // Right: AI Agent Panel
                            if state.ai_panel_visible {
                                ui_layout::draw_ai_panel(ui, &mut state, actual_ai_w, middle_h);
                            }
                        });

                    // 3. Status Bar
                    if state.status_bar_visible {
                        ui_layout::draw_status_bar(ui, &mut state);
                    }

                    // 4. Modal overlays (About dialog, Language installation prompt)
                    ui_layout::draw_about_dialog(ui, &mut state);
                    ui_layout::draw_language_prompt_dialog(ui, &mut state);
                });
        })
        .run();
}
