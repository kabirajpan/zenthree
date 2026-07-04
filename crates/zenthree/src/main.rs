mod state;
mod theme;
mod ui_layout;
mod assets;

use state::ZenthreeState;
use theme::ThemeMode;
use zenthra::prelude::*;
use zenthra::Id;

fn main() {
    env_logger::init();

    // Initialize the modular state
    let mut state = ZenthreeState::new();

    // Start with one default empty file so the editor doesn't feel empty
    state.new_untitled_file();

    let font_bytes = include_bytes!("../../../../../assets/fonts/SymbolsNerdFont-Regular.ttf").to_vec();

    App::new()
        .title("Zenthree Editor")
        .size(1100, 680)
        .decorations(false)
        .load_font_data(font_bytes)
        .with_ui(move |ui| {
            // Expose the active theme flag to interaction_state so widgets can read it
            let theme_val = if state.theme == ThemeMode::Light { 1.0 } else { 0.0 };
            ui.interaction_state.insert(Id::from_u64(999999999), theme_val);

            let colors = state.colors();

            // Main Background Container: vertical layout
            ui.container()
                .fill()
                .bg(colors.bg_base)
                .column()
                .show(|ui| {
                    // 1. Draw Title Bar
                    ui_layout::draw_title_bar(ui, &mut state);

                    // 2. Middle Workspace Area (Sidebar + Editor)
                    // Fills the remaining space using row direction
                    let ui_height = ui.height;
                    ui.container()
                        .row()
                        .fill_x()
                        .height(ui_height - 36.0 - 24.0)
                        .show(|ui| {
                            // Sidebar
                            if state.sidebar_visible {
                                ui_layout::draw_sidebar(ui, &mut state);
                            }

                            // Main Editor Panels Buffer
                            ui_layout::draw_main_buffer(ui, &mut state);
                        });

                    // 3. Status Bar
                    ui_layout::draw_status_bar(ui, &mut state);

                    // 4. Modal overlays (About dialog)
                    ui_layout::draw_about_dialog(ui, &mut state);
                });
        })
        .run();
}
