mod state;
mod theme;
mod ui_layout;
mod assets;
mod neovim_icons;
mod mini_icons;
pub mod native_terminal;
pub mod extensions;
pub mod lsp;
pub mod watcher;

use state::ZenthreeState;
use zenthra::prelude::*;

fn main() {
    // Only load .env from the current working directory, do NOT search parent directories.
    let _ = dotenvy::from_path("./.env");
    env_logger::init();

    // Initialize the modular state (loads persistent state from disk)
    let mut state = ZenthreeState::new();

    // Start with one default empty file only if no files were restored from previous session
    if state.tabs.is_empty() {
        state.new_untitled_file();
    }

    let font_bytes = include_bytes!("../../../assets/fonts/SymbolsNerdFont-Regular-3.5.1.ttf").to_vec();
    let initial_blur = state.glassmorphism;

    App::new()
        .title("Zenthree Editor")
        .size(1100, 680)
        .decorations(false)
        .transparent(true)
        .blur(initial_blur)
        .load_font_data(font_bytes)
        .register_custom_shader("spherical_polyhedra", include_str!("shaders/spherical_polyhedra.wgsl"))
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
                        winit::keyboard::KeyCode::Comma if state.ctrl_pressed => {
                            state.editor_settings_open = !state.editor_settings_open;
                            state.editor_settings_just_opened = true;
                            if state.editor_settings_open {
                                state.editor_settings_tab = crate::state::EditorSettingsTab::Appearance;
                            }
                        }
                        winit::keyboard::KeyCode::F11 => {
                            state.focus_mode = !state.focus_mode;
                            if state.focus_mode {
                                // Entering focus mode: hide title bar + activity bar + terminal + AI
                                state.activity_bar_visible = false;
                                state.terminal_visible = false;
                                state.ai_panel_visible = false;
                            } else {
                                // Exiting focus mode: restore activity bar
                                state.activity_bar_visible = true;
                            }
                        }
                        // Ctrl+B → toggle sidebar (VSCode style)
                        winit::keyboard::KeyCode::KeyB if state.ctrl_pressed && !state.shift_pressed => {
                            state.sidebar_visible = !state.sidebar_visible;
                        }
                        // Ctrl+Shift+B → toggle activity bar
                        winit::keyboard::KeyCode::KeyB if state.ctrl_pressed && state.shift_pressed => {
                            state.activity_bar_visible = !state.activity_bar_visible;
                        }
                        // Ctrl+` → toggle terminal (VSCode style)
                        winit::keyboard::KeyCode::Backquote if state.ctrl_pressed => {
                            state.terminal_visible = !state.terminal_visible;
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

            // Poll background LSP messages (diagnostics, completions, server lifecycle)
            if state.lsp_manager.poll() {
                ui.request_redraw();
            }

            // Drain background AI agent updates pushed via ArcSignal.
            // No timer needed: ArcSignal::with_mut() calls trigger_redraw_hook()
            // on every push, waking the event loop immediately.
            state.drain_ai_updates();

            if !state.active_agent_terminal_tasks.is_empty() {
                ui.request_redraw();
            }

            // Poll filesystem watcher for external file additions, deletions, renames, and git status changes
            if state.poll_file_tree_changes() {
                ui.request_redraw();
            }

            // Auto-clear stale status messages after 3 seconds.
            // The _status_clear_effect wakes the event loop when the time is up.
            state.tick_status();

            let colors = state.colors();

            // Sync full ThemeColors into UI interaction_state so menus, dropdowns, and widgets match active theme
            crate::theme::sync_theme_to_ui(ui, state.theme, &colors, state.glassmorphism);

            // Provide reactive ThemeColors to the UI tree via Context API
            provide_context(colors);

            // When any modal/dialog is open, activate modal mode so all bottom/background widgets are completely shielded
            ui.modal_active = state.is_modal_or_dialog_open();

            // When resizing any panel, disable scrollbars and scroll gestures across the UI
            ui.resize_active = state.active_resize_sidebar
                || state.active_resize_ai_panel
                || state.active_resize_terminal
                || state.git_state.active_resize_graph;

            // Main Background Container: vertical layout
            ui.container()
                .fill()
                .bg(if state.glassmorphism { Color::TRANSPARENT } else { colors.bg_base })
                .column()
                .show(|ui| {
                    // 1. Draw Title Bar (hidden in focus mode)
                    if !state.focus_mode {
                        ui_layout::draw_title_bar(ui, &mut state);
                    }

                    // 2. Middle Workspace Area (Sidebar + Editor + Terminal + AI Panel)
                    let ui_height = ui.height;
                    let ui_width = ui.width;
                    let status_bar_h = if state.status_bar_visible { 24.0 } else { 0.0 };
                    let title_bar_h = if state.focus_mode { 0.0 } else { 36.0 };
                    let middle_h = ui_height - title_bar_h - status_bar_h;

                    let activity_bar_w = if state.activity_bar_visible { ui_layout::ACTIVITY_BAR_WIDTH } else { 0.0 };
                    let actual_sb_w = if state.sidebar_visible {
                        state.sidebar_width.min(ui_width * 0.4).max(140.0)
                    } else {
                        0.0
                    };

                    let min_editor_w = 160.0;
                    let avail_for_ai = (ui_width - activity_bar_w - actual_sb_w - min_editor_w).max(160.0);
                    let actual_ai_w = if state.ai_panel_visible {
                        if state.ai_panel_expanded {
                            (ui_width - activity_bar_w - actual_sb_w).max(200.0)
                        } else {
                            state.ai_panel_width.min(avail_for_ai).max(180.0)
                        }
                    } else {
                        0.0
                    };

                    if state.ai_panel_visible && !state.ai_panel_expanded {
                        state.ai_panel_width = actual_ai_w;
                    }
                    if state.sidebar_visible {
                        state.sidebar_width = actual_sb_w;
                    }

                    let center_w = (ui_width - activity_bar_w - actual_sb_w - if state.ai_panel_visible && !state.ai_panel_expanded { actual_ai_w } else { 0.0 }).max(min_editor_w);

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
                            if state.activity_bar_visible {
                                ui_layout::draw_activity_bar(ui, &mut state, middle_h);
                            }

                            // Left: Sidebar / File Tree or Extensions Panel
                            if state.sidebar_visible {
                                ui_layout::draw_sidebar(ui, &mut state);
                            }

                            if state.ai_panel_visible && state.ai_panel_expanded {
                                // When expanded, the AI Agent Panel sits in the buffer area
                                let expanded_w = (ui_width - activity_bar_w - actual_sb_w).max(200.0);
                                ui_layout::draw_ai_panel(ui, &mut state, expanded_w, middle_h);
                            } else {
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

                                // Right: AI Agent Panel (docked mode)
                                if state.ai_panel_visible {
                                    ui_layout::draw_ai_panel(ui, &mut state, actual_ai_w, middle_h);
                                }
                            }
                        });

                    // 3. Status Bar
                    if state.status_bar_visible {
                        ui_layout::draw_status_bar(ui, &mut state);
                    }

                    // 4. Modal overlays (dialogs — called inside the column so they
                    // inherit the correct viewport/clip context)
                    ui_layout::draw_about_dialog(ui, &mut state);
                    ui_layout::draw_language_prompt_dialog(ui, &mut state);
                    ui_layout::draw_ai_settings_dialog(ui, &mut state);
                    ui_layout::draw_editor_settings_dialog(ui, &mut state);
                    ui_layout::draw_unsaved_changes_dialog(ui, &mut state);
                    ui_layout::draw_thinking_orbs_demo_window(ui, &mut state);
                    // Floating popovers rendered LAST so they appear above all dialog content
                    ui_layout::draw_antigravity_settings_model_popover(ui, &mut state);
                });
        })
        .run();
}
