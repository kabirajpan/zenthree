use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, ObjectFit};

pub fn draw_sidebar(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = state.colors();
    let sidebar_w = state.sidebar_width;

    ui.container()
        .width(sidebar_w)
        .fill_y()
        .bg(colors.bg_sidebar)
        .border(colors.border, 1.0)
        .column()
        .show(|ui| {
            // Sidebar Title
            ui.container()
                .full_width()
                .padding(12.0, 12.0, 4.0, 12.0)
                .row()
                .valign(Align::Center)
                .show(|ui| {
                    ui.text("EXPLORER")
                        .size(10.5)
                        .weight(FontWeight::Bold)
                        .color(colors.text_muted)
                        .show();
                });

            // Root Folder Name / Header
            let root_name = state.current_dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("/")
                .to_uppercase();

            let icon_theme = crate::theme::IconTheme::new("gruvbox");
            let root_icon = icon_theme.get_icon_source("folder", "");

            ui.container()
                .full_width()
                .padding(4.0, 12.0, 8.0, 12.0)
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    ui.image(root_icon)
                        .size(16.0, 16.0)
                        .fit(ObjectFit::Contain)
                        .show();
                    ui.text(&root_name)
                        .size(11.0)
                        .weight(FontWeight::Bold)
                        .color(colors.text_primary)
                        .show();
                });

            // File Listing scroll area (Tree View)
            ui.container()
                .full_width()
                .fill_y()
                .scroll_y(true)
                .padding(2.0, 6.0, 2.0, 6.0)
                .gap(2.0)
                .show(|ui| {
                    let items = state.sidebar_items.clone();
                    for (idx, node) in items.iter().enumerate() {
                        let is_dir = node.is_dir;
                        let depth = node.depth;
                        
                        // Left padding based on depth
                        let indent = depth as f32 * 14.0;
                        
                        let item_id = Id::from_u64(999993000 + idx as u64);

                        let icon_source = if is_dir {
                            icon_theme.get_icon_source("folder", "")
                        } else {
                            let extension = node.path.extension()
                                .and_then(|ext| ext.to_str())
                                .unwrap_or("")
                                .to_lowercase();
                            icon_theme.get_icon_source("text", &extension)
                        };

                        let resp = ui.container()
                            .id(item_id)
                            .full_width()
                            .row()
                            .valign(Align::Center)
                            .padding_left(12.0 + indent)
                            .padding_top(4.0)
                            .padding_bottom(4.0)
                            .gap(6.0)
                            .bg(Color::TRANSPARENT)
                            .hover_bg(colors.highlight)
                            .radius_all(4.0)
                            .show(|ui| {
                                if is_dir {
                                    // Expand/Collapse arrow (Chevrons)
                                    let arrow = if node.is_expanded { "\u{f078}" } else { "\u{f054}" }; // Chevron down vs right
                                    ui.text(arrow)
                                        .size(8.0)
                                        .color(colors.text_muted)
                                        .show();
                                    
                                    ui.image(icon_source)
                                        .size(16.0, 16.0)
                                        .fit(ObjectFit::Contain)
                                        .show();
                                } else {
                                    // File Indent: spacing to align icon with folder icon
                                    ui.spacing(14.0);
                                    
                                    ui.image(icon_source)
                                        .size(16.0, 16.0)
                                        .fit(ObjectFit::Contain)
                                        .show();
                                }

                                ui.text(&node.name)
                                    .size(11.5)
                                    .color(colors.text_primary)
                                    .show();
                            });

                        if resp.clicked {
                            if is_dir {
                                if node.is_expanded {
                                    state.expanded_dirs.remove(&node.path);
                                } else {
                                    state.expanded_dirs.insert(node.path.clone());
                                }
                                state.rebuild_tree();
                            } else {
                                state.open_file(&node.path);
                            }
                            ui.request_redraw();
                        }
                    }
                });
        });

    // Render vertical splitter handle and capture mouse drags
    let _split_id = Id::from_u64(999999099);
    let splitter_x = ui.cursor_x + sidebar_w;
    let splitter_y = ui.cursor_y;
    let splitter_h = ui.height - 36.0 - 24.0;

    // Detect if mouse is hovering over 6-pixel splitter bounds
    let handle_width = 6.0;
    let is_hovered = ui.mouse_in_rect(splitter_x - handle_width/2.0 + ui.offset_x, splitter_y + ui.offset_y, handle_width, splitter_h);

    if is_hovered && ui.mouse_down {
        state.active_resize_sidebar = true;
    }

    if state.active_resize_sidebar {
        if !ui.mouse_down {
            state.active_resize_sidebar = false;
        } else {
            // Dragging: update sidebar width
            let target_w = ui.mouse_x - splitter_x + sidebar_w;
            state.sidebar_width = target_w.clamp(140.0, 600.0);
            ui.request_redraw();
        }
    }
}
