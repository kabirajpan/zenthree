mod state;
mod theme;
mod ui_layout;
mod assets;
mod neovim_icons;
mod mini_icons;
pub mod native_terminal;
pub mod extensions;
pub mod lsp;
pub mod markdown;
pub mod groq;
pub mod gemini;
pub mod project_index;
pub mod agent;
pub mod session;



use state::ZenthreeState;
use theme::ThemeMode;
use zenthra::prelude::*;
use zenthra::Id;

fn main() {
    #[cfg(unix)]
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_IGN);
    }

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
            // Apply universal UI font scaling
            ui.font_scale = state.zoom_level;

            // Poll any completed background dialogs (e.g. Open Folder / Open File)
            let mut should_redraw = state.poll_pending_dialogs();

            // Track global modifier key states and global shortcuts from input events
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
                        // Universal font & UI zoom controller
                        winit::keyboard::KeyCode::Equal | winit::keyboard::KeyCode::NumpadAdd if state.ctrl_pressed => {
                            state.zoom_in();
                            should_redraw = true;
                        }
                        winit::keyboard::KeyCode::Minus | winit::keyboard::KeyCode::NumpadSubtract if state.ctrl_pressed => {
                            state.zoom_out();
                            should_redraw = true;
                        }
                        winit::keyboard::KeyCode::Digit0 | winit::keyboard::KeyCode::Numpad0 if state.ctrl_pressed => {
                            state.reset_zoom();
                            should_redraw = true;
                        }
                        // Toggle/Open Source Control (Ctrl+Shift+G)
                        winit::keyboard::KeyCode::KeyG if state.ctrl_pressed && state.shift_pressed => {
                            if state.sidebar_visible && state.sidebar_view == state::SidebarView::SourceControl {
                                state.set_sidebar_visible(false);
                            } else {
                                state.set_sidebar_view(state::SidebarView::SourceControl);
                                state.set_sidebar_visible(true);
                                state.refresh_git_status();
                            }
                            should_redraw = true;
                        }
                        // Go to Definition (F12)
                        winit::keyboard::KeyCode::F12 => {
                            state.goto_definition_at_cursor();
                            should_redraw = true;
                        }
                        // Request Hover Documentation (Ctrl+H)
                        winit::keyboard::KeyCode::KeyH if state.ctrl_pressed => {
                            state.request_hover_at_cursor();
                            should_redraw = true;
                        }
                        // Toggle In-Buffer Find (Ctrl+F)
                        winit::keyboard::KeyCode::KeyF if state.ctrl_pressed => {
                            state.toggle_find_in_buffer();
                            should_redraw = true;
                        }
                        // Find match navigation when find overlay is active
                        winit::keyboard::KeyCode::Enter if state.find_in_buffer_open => {
                            if state.shift_pressed {
                                state.prev_find_match();
                            } else {
                                state.next_find_match();
                            }
                            should_redraw = true;
                        }
                        winit::keyboard::KeyCode::Escape if state.find_in_buffer_open => {
                            state.find_in_buffer_open = false;
                            state.find_matches.clear();
                            should_redraw = true;
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
            if should_redraw {
                ui.request_redraw();
            }

            // Poll background LSP messages (diagnostics, completions, definition, hover, server lifecycle)
            if state.lsp_manager.poll() {
                if let Some(loc) = state.lsp_manager.latest_definition.take() {
                    state.jump_to_location(&loc);
                }
                if let Some(hover) = state.lsp_manager.latest_hover.take() {
                    state.active_hover_text = Some(hover.contents.to_markdown_or_plain());
                }
                ui.request_redraw();
            }

            // Poll background AI responses (Groq LLM + Connectome)
            if state.poll_ai_responses() {
                ui.request_redraw();
            } else if state.ai_pending_requests > 0 {
                ui.request_redraw_after(std::time::Duration::from_millis(33));
            }

            // Poll background native terminal PTY output
            if state.poll_terminal() {
                ui.request_redraw();
            }


            // Expose the active theme flag to interaction_state so widgets can read it
            let theme_val = if state.theme == ThemeMode::Light { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999999), theme_val);

            state.sync_reactive_domains();

            let colors = state.colors();

            // Feed colors to interaction_state so popup menus and framework widgets reflect active accent/theme
            ui.interaction_state.insert(Id::from_u64(999999980), colors.accent.r);
            ui.interaction_state.insert(Id::from_u64(999999981), colors.accent.g);
            ui.interaction_state.insert(Id::from_u64(999999982), colors.accent.b);
            ui.interaction_state.insert(Id::from_u64(999999983), colors.accent.a);

            ui.interaction_state.insert(Id::from_u64(999999970), colors.highlight.r);
            ui.interaction_state.insert(Id::from_u64(999999971), colors.highlight.g);
            ui.interaction_state.insert(Id::from_u64(999999972), colors.highlight.b);
            ui.interaction_state.insert(Id::from_u64(999999973), colors.highlight.a);

            ui.interaction_state.insert(Id::from_u64(999999960), colors.text_primary.r);
            ui.interaction_state.insert(Id::from_u64(999999961), colors.text_primary.g);
            ui.interaction_state.insert(Id::from_u64(999999962), colors.text_primary.b);
            ui.interaction_state.insert(Id::from_u64(999999963), colors.text_primary.a);

            ui.interaction_state.insert(Id::from_u64(999999940), colors.bg_sidebar.r);
            ui.interaction_state.insert(Id::from_u64(999999941), colors.bg_sidebar.g);
            ui.interaction_state.insert(Id::from_u64(999999942), colors.bg_sidebar.b);
            ui.interaction_state.insert(Id::from_u64(999999943), colors.bg_sidebar.a);

            ui.interaction_state.insert(Id::from_u64(999999930), colors.border.r);
            ui.interaction_state.insert(Id::from_u64(999999931), colors.border.g);
            ui.interaction_state.insert(Id::from_u64(999999932), colors.border.b);
            ui.interaction_state.insert(Id::from_u64(999999933), colors.border.a);

            // Provide reactive ThemeColors and Domain States to the UI tree via Context API
            provide_context(colors);
            provide_context(state.theme_state.clone());
            provide_context(state.ui_layout.clone());
            provide_context(state.workspace.clone());
            provide_context(state.agent_state.clone());

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
