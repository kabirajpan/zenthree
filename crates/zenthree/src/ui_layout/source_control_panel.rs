use crate::state::{ZenthreeState, GitFileStatusCode};
use zenthra::{Color, Ui, Align, FontWeight, Id, CursorIcon};
use std::path::PathBuf;

#[inline]
fn color_modified() -> Color { Color::from_hex(0xe5a030) }
#[inline]
fn color_added() -> Color { Color::from_hex(0x4eba6f) }
#[inline]
fn color_deleted() -> Color { Color::from_hex(0xe65050) }
#[inline]
fn color_renamed() -> Color { Color::from_hex(0x46bedc) }
#[inline]
fn color_untracked() -> Color { Color::from_hex(0x4eba6f) }
#[inline]
fn color_conflicted() -> Color { Color::from_hex(0xeb4646) }
#[inline]
fn color_error() -> Color { Color::from_hex(0xdc3c3c) }

#[inline]
fn get_graph_color(lane: usize) -> Color {
    const GRAPH_COLORS: [u32; 6] = [
        0x3b82f6, // 0: Vibrant Blue (#3b82f6)
        0x22c55e, // 1: Emerald Green (#22c55e)
        0xf59e0b, // 2: Amber / Orange (#f59e0b)
        0xa855f7, // 3: Purple / Violet (#a855f7)
        0x06b6d4, // 4: Cyan (#06b6d4)
        0xf43f5e, // 5: Rose / Pink (#f43f5e)
    ];
    Color::from_hex(GRAPH_COLORS[lane % GRAPH_COLORS.len()])
}

pub fn draw_source_control_panel(ui: &mut Ui, state: &mut ZenthreeState, panel_w: f32) {
    let colors = zenthra::use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    ui.container()
        .width(panel_w)
        .fill_y()
        .clip(true)
        .column()
        .show(|ui| {
            // Header: "SOURCE CONTROL" + branch pill + actions
            ui.container()
                .full_width()
                .padding(8.0, 10.0, 6.0, 10.0)
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .show(|ui| {
                    ui.container()
                        .row()
                        .valign(Align::Center)
                        .gap(6.0)
                        .show(|ui| {
                            ui.text("SOURCE CONTROL")
                                .size(11.0)
                                .weight(FontWeight::Bold)
                                .color(colors.text_muted)
                                .show();

                            if state.git_state.is_repo && !state.git_state.branch.is_empty() {
                                // Branch pill badge
                                ui.container()
                                    .height(18.0)
                                    .radius_all(3.0)
                                    .bg(colors.accent.with_alpha(0.14))
                                    .padding(0.0, 5.0, 0.0, 5.0)
                                    .row()
                                    .valign(Align::Center)
                                    .gap(3.5)
                                    .show(|ui| {
                                        // Dev git branch icon (\u{e725})
                                        ui.text("\u{e725}")
                                            .size(10.0)
                                            .color(colors.accent)
                                            .show();

                                        let branch_name = if state.git_state.branch.len() > 14 {
                                            format!("{}...", &state.git_state.branch[..12])
                                        } else {
                                            state.git_state.branch.clone()
                                        };

                                        ui.text(&branch_name)
                                            .size(9.5)
                                            .weight(FontWeight::Bold)
                                            .color(colors.accent)
                                            .show();
                                    });
                            }
                        });

                    if state.git_state.is_repo {
                        ui.container()
                            .row()
                            .valign(Align::Center)
                            .gap(4.0)
                            .show(|ui| {
                                // Refresh button (\u{f01e})
                                let ref_btn = ui.button("\u{f01e}")
                                    .id(Id::from_u64(999998310))
                                    .height(20.0)
                                    .bg(Color::TRANSPARENT)
                                    .text_color(colors.text_muted)
                                    .size(11.0)
                                    .radius_all(3.0)
                                    .padding_left(4.0)
                                    .padding_right(4.0)
                                    .show();
                                if ref_btn.clicked {
                                    state.refresh_git_status();
                                    ui.request_redraw();
                                }

                                // Stage all button (\u{f067})
                                if !state.git_state.unstaged.is_empty() {
                                    let stage_all_btn = ui.button("\u{f067}")
                                        .id(Id::from_u64(999998311))
                                        .height(20.0)
                                        .bg(Color::TRANSPARENT)
                                        .text_color(colors.text_muted)
                                        .size(10.5)
                                        .radius_all(3.0)
                                        .padding_left(4.0)
                                        .padding_right(4.0)
                                        .show();
                                    if stage_all_btn.clicked {
                                        state.git_stage_all();
                                        ui.request_redraw();
                                    }
                                }
                            });
                    }
                });

            // 1. Not a workspace
            if !state.has_workspace {
                draw_empty_no_workspace(ui, state);
                return;
            }

            // 2. Workspace exists but not a git repo
            if !state.git_state.is_repo {
                draw_empty_not_a_repo(ui, state);
                return;
            }

            // 3. Git repository view
            // Commit message input area
            ui.container()
                .full_width()
                .padding(4.0, 10.0, 8.0, 10.0)
                .column()
                .gap(6.0)
                .show(|ui| {
                    // Commit text area with native placeholder
                    let text_area_w = (panel_w - 20.0).max(100.0);
                    let commit_input_id = Id::from_u64(999998300);
                    let is_input_focused = ui.focused_id == Some(commit_input_id);

                    let _commit_input = ui.text_area(&mut state.git_state.commit_message, commit_input_id)
                        .width(text_area_w)
                        .height(34.0)
                        .max_height(80.0)
                        .size(11.0)
                        .line_height(1.25)
                        .color(colors.text_primary)
                        .bg(colors.bg_editor)
                        .border(colors.border, 1.0)
                        .focus_border(colors.accent, 1.0)
                        .radius_all(4.0)
                        .padding(4.0, 8.0, 4.0, 8.0)
                        .placeholder("Message (Ctrl+Enter to commit)")
                        .placeholder_color(colors.text_dim)
                        .wrap(zenthra_text::prelude::TextWrap::Word)
                        .submit_on_enter(false)
                        .show();

                    // Commit button
                    let has_changes = !state.git_state.staged.is_empty() || !state.git_state.unstaged.is_empty();
                    let has_msg = !state.git_state.commit_message.trim().is_empty();
                    let can_commit = has_changes && has_msg;

                    let mut do_commit = false;
                    if is_input_focused && state.ctrl_pressed {
                        for event in &ui.input_events {
                            if let zenthra_platform::event::PlatformEvent::KeyDown { key } = event {
                                if matches!(key, winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter) {
                                    do_commit = true;
                                    break;
                                }
                            }
                        }
                    }

                    let commit_btn = ui.container()
                        .clickable(can_commit)
                        .full_width()
                        .height(26.0)
                        .radius_all(4.0)
                        .bg(if can_commit { colors.accent } else { colors.highlight })
                        .halign(Align::Center)
                        .valign(Align::Center)
                        .row()
                        .gap(5.0)
                        .show(|ui| {
                            ui.text("\u{f00c}") // check icon
                                .size(11.0)
                                .color(if can_commit { Color::WHITE } else { colors.text_dim })
                                .show();

                            ui.text("Commit")
                                .size(11.0)
                                .weight(FontWeight::Medium)
                                .color(if can_commit { Color::WHITE } else { colors.text_dim })
                                .show();
                        });

                    if (commit_btn.clicked || do_commit) && can_commit {
                        let _ = state.git_commit();
                        ui.request_redraw();
                    }

                    // Status / error message banner if commit failed
                    if let Some(ref err) = state.git_state.status_error {
                        ui.container()
                            .full_width()
                            .radius_all(4.0)
                            .bg(color_error().with_alpha(0.12))
                            .border(color_error().with_alpha(0.3), 1.0)
                            .padding(4.0, 6.0, 4.0, 6.0)
                            .show(|ui| {
                                ui.text(err)
                                    .size(10.0)
                                    .color(color_error())
                                    .show();
                            });
                    }
                });

            let mut file_to_open: Option<PathBuf> = None;
            let mut file_to_stage: Option<PathBuf> = None;
            let mut file_to_unstage: Option<PathBuf> = None;
            let mut file_to_discard: Option<(PathBuf, bool)> = None;

            let graph_expanded = state.git_state.graph_expanded;
            let total_graph_h = if graph_expanded { 24.0 + 4.0 + state.git_state.graph_height } else { 24.0 };
            let avail_h = (ui.max_y - ui.cursor_y).max(100.0);
            let changes_h = (avail_h - total_graph_h).max(60.0);

            // Scrollable section for changes (staged + unstaged)
            ui.container()
                .full_width()
                .height(changes_h)
                .scroll_y(true)
                .clip(true)
                .column()
                .show(|ui| {
                    let staged_len = state.git_state.staged.len();
                    let unstaged_len = state.git_state.unstaged.len();

                    let icon_theme = crate::theme::IconTheme::new(&state.icon_theme);

                    // 1. Staged Changes Section
                    if staged_len > 0 {
                        let staged_expanded = state.git_state.staged_expanded;
                        let mut toggle_staged = false;
                        let mut do_unstage_all = false;

                        let staged_hdr_key = Id::from_u64(999998311);
                        let staged_hdr_hov = ui.interaction_state.get(&staged_hdr_key).copied().unwrap_or(0.0) > 0.5;

                        // Accordion header: sleek dark bar matching sidebar, highlights on hover
                        ui.container()
                            .full_width()
                            .height(24.0)
                            .padding_left(8.0)
                            .padding_right(8.0)
                            .bg(if staged_hdr_hov { colors.highlight.with_alpha(0.6) } else { Color::TRANSPARENT })
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .no_wrap()
                            .show(|ui| {
                                let hdr_btn = ui.container()
                                    .raw_id(staged_hdr_key)
                                    .clickable(true)
                                    .row()
                                    .valign(Align::Center)
                                    .no_wrap()
                                    .gap(5.0)
                                    .show(|ui| {
                                        ui.text(if staged_expanded { "\u{f078}" } else { "\u{f054}" })
                                            .size(9.0)
                                            .color(if staged_hdr_hov { colors.text_primary } else { colors.text_muted })
                                            .show();

                                        ui.text("STAGED CHANGES")
                                            .size(10.5)
                                            .weight(FontWeight::Bold)
                                            .color(if staged_hdr_hov { colors.text_primary } else { colors.text_muted })
                                            .show();

                                        // Badge count
                                        ui.container()
                                            .height(15.0)
                                            .padding(0.0, 5.0, 0.0, 5.0)
                                            .radius_all(7.5)
                                            .bg(colors.accent.with_alpha(0.18))
                                            .halign(Align::Center)
                                            .valign(Align::Center)
                                            .show(|ui| {
                                                ui.text(&format!("{}", staged_len))
                                                    .size(9.0)
                                                    .weight(FontWeight::Bold)
                                                    .color(colors.accent)
                                                    .show();
                                            });
                                    });

                                let h = hdr_btn.hovered;
                                let prev = ui.interaction_state.insert(staged_hdr_key, if h { 1.0 } else { 0.0 });
                                if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                                if hdr_btn.clicked {
                                    toggle_staged = true;
                                }

                                // Unstage All button (-)
                                let unstage_all_btn = ui.button("\u{f068}")
                                    .id(Id::from_u64(999998312))
                                    .height(18.0)
                                    .bg(Color::TRANSPARENT)
                                    .text_color(if staged_hdr_hov { colors.text_primary } else { colors.text_muted })
                                    .size(9.5)
                                    .radius_all(3.0)
                                    .padding_left(4.0)
                                    .padding_right(4.0)
                                    .show();
                                if unstage_all_btn.clicked {
                                    do_unstage_all = true;
                                }
                            });

                        if toggle_staged {
                            state.git_state.staged_expanded = !state.git_state.staged_expanded;
                            ui.request_redraw();
                        }

                        if do_unstage_all {
                            state.git_unstage_all();
                            ui.request_redraw();
                        }

                        if staged_expanded {
                            for (idx, item) in state.git_state.staged.iter().enumerate() {
                                let item_key = Id::from_u64(999998400 + idx as u64);
                                let item_hov = ui.interaction_state.get(&item_key).copied().unwrap_or(0.0) > 0.5;

                                let icon = icon_theme.get_icon_for_file(&item.filename, Some(&item.path));
                                let (badge_char, badge_color) = match item.status {
                                    GitFileStatusCode::Modified => ("M", color_modified()),
                                    GitFileStatusCode::Added => ("A", color_added()),
                                    GitFileStatusCode::Deleted => ("D", color_deleted()),
                                    GitFileStatusCode::Renamed => ("R", color_renamed()),
                                    GitFileStatusCode::Untracked => ("U", color_untracked()),
                                    GitFileStatusCode::Conflicted => ("C", color_conflicted()),
                                };

                                let right_reserved = if item_hov { 58.0 } else { 24.0 };
                                let left_max_w = (panel_w - right_reserved - 20.0).max(25.0);

                                let row = ui.container()
                                    .clickable(true)
                                    .full_width()
                                    .height(22.0)
                                    .padding_left(12.0)
                                    .padding_right(8.0)
                                    .bg(if item_hov { colors.highlight } else { Color::TRANSPARENT })
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .no_wrap()
                                    .show(|ui| {
                                        // Left: icon + filename + dynamic dir path (only truncates when shrinking close)
                                        ui.container()
                                            .max_width(left_max_w)
                                            .clip(true)
                                            .row()
                                            .valign(Align::Center)
                                            .no_wrap()
                                            .gap(5.0)
                                            .show(|ui| {
                                                icon.show(ui, 13.0);

                                                ui.text(&item.filename)
                                                    .size(11.5)
                                                    .color(colors.text_primary)
                                                    .show();

                                                if !item.dir_path.is_empty() {
                                                    let filename_w = item.filename.len() as f32 * 6.5 + 23.0;
                                                    let avail_dir_w = left_max_w - filename_w;
                                                    if avail_dir_w > 20.0 {
                                                        let max_chars = (avail_dir_w / 5.2) as usize;
                                                        let display_dir = if item.dir_path.len() <= max_chars {
                                                            item.dir_path.clone()
                                                        } else if max_chars >= 5 {
                                                            format!("{}…", &item.dir_path[..max_chars.saturating_sub(1)])
                                                        } else {
                                                            String::new()
                                                        };

                                                        if !display_dir.is_empty() {
                                                            ui.text(&display_dir)
                                                                .size(9.5)
                                                                .color(colors.text_dim)
                                                                .show();
                                                        }
                                                    }
                                                }
                                            });

                                        // Right: status badge or unstage button on hover (always pinned on right)
                                        ui.container()
                                            .row()
                                            .valign(Align::Center)
                                            .no_wrap()
                                            .gap(4.0)
                                            .show(|ui| {
                                                if item_hov {
                                                    let unstage_btn = ui.button("\u{f068}") // minus icon
                                                        .id(Id::from_u64(999998500 + idx as u64))
                                                        .height(18.0)
                                                        .bg(Color::TRANSPARENT)
                                                        .text_color(colors.text_muted)
                                                        .size(9.5)
                                                        .radius_all(3.0)
                                                        .padding_left(3.0)
                                                        .padding_right(3.0)
                                                        .show();
                                                    if unstage_btn.clicked {
                                                        file_to_unstage = Some(item.path.clone());
                                                    }
                                                }

                                                ui.text(badge_char)
                                                    .size(10.5)
                                                    .weight(FontWeight::Bold)
                                                    .color(badge_color)
                                                    .show();
                                            });
                                    });

                                let h = row.hovered;
                                let prev = ui.interaction_state.insert(item_key, if h { 1.0 } else { 0.0 });
                                if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                                if row.clicked {
                                    file_to_open = Some(item.path.clone());
                                }
                            }
                        }
                    }

                    // 2. Changes (Unstaged) Section
                    if unstaged_len > 0 {
                        let changes_expanded = state.git_state.changes_expanded;
                        let mut toggle_changes = false;
                        let mut do_stage_all = false;

                        let changes_hdr_key = Id::from_u64(999998310);
                        let changes_hdr_hov = ui.interaction_state.get(&changes_hdr_key).copied().unwrap_or(0.0) > 0.5;

                        // Accordion header: sleek dark bar matching sidebar, highlights on hover
                        ui.container()
                            .full_width()
                            .height(24.0)
                            .padding_left(8.0)
                            .padding_right(8.0)
                            .bg(if changes_hdr_hov { colors.highlight.with_alpha(0.6) } else { Color::TRANSPARENT })
                            .row()
                            .valign(Align::Center)
                            .halign(Align::SpaceBetween)
                            .no_wrap()
                            .show(|ui| {
                                let hdr_btn = ui.container()
                                    .raw_id(changes_hdr_key)
                                    .clickable(true)
                                    .row()
                                    .valign(Align::Center)
                                    .no_wrap()
                                    .gap(5.0)
                                    .show(|ui| {
                                        ui.text(if changes_expanded { "\u{f078}" } else { "\u{f054}" })
                                            .size(9.0)
                                            .color(if changes_hdr_hov { colors.text_primary } else { colors.text_muted })
                                            .show();

                                        ui.text("CHANGES")
                                             .size(10.5)
                                             .weight(FontWeight::Bold)
                                             .color(if changes_hdr_hov { colors.text_primary } else { colors.text_muted })
                                             .show();

                                        // Badge count
                                        ui.container()
                                             .height(15.0)
                                             .padding(0.0, 5.0, 0.0, 5.0)
                                             .radius_all(7.5)
                                             .bg(colors.accent.with_alpha(0.18))
                                             .halign(Align::Center)
                                             .valign(Align::Center)
                                             .show(|ui| {
                                                 ui.text(&format!("{}", unstaged_len))
                                                     .size(9.0)
                                                     .weight(FontWeight::Bold)
                                                     .color(colors.accent)
                                                     .show();
                                             });
                                    });

                                let h = hdr_btn.hovered;
                                let prev = ui.interaction_state.insert(changes_hdr_key, if h { 1.0 } else { 0.0 });
                                if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                                if hdr_btn.clicked {
                                    toggle_changes = true;
                                }

                                // Stage All button (+)
                                let stage_all_btn = ui.button("\u{f067}")
                                    .id(Id::from_u64(999998313))
                                    .height(18.0)
                                    .bg(Color::TRANSPARENT)
                                    .text_color(if changes_hdr_hov { colors.text_primary } else { colors.text_muted })
                                    .size(9.5)
                                    .radius_all(3.0)
                                    .padding_left(4.0)
                                    .padding_right(4.0)
                                    .show();
                                if stage_all_btn.clicked {
                                    do_stage_all = true;
                                }
                            });

                        if toggle_changes {
                            state.git_state.changes_expanded = !state.git_state.changes_expanded;
                            ui.request_redraw();
                        }

                        if do_stage_all {
                            state.git_stage_all();
                            ui.request_redraw();
                        }

                        if changes_expanded {
                            for (idx, item) in state.git_state.unstaged.iter().enumerate() {
                                let item_key = Id::from_u64(999998600 + idx as u64);
                                let item_hov = ui.interaction_state.get(&item_key).copied().unwrap_or(0.0) > 0.5;

                                let icon = icon_theme.get_icon_for_file(&item.filename, Some(&item.path));
                                let (badge_char, badge_color) = match item.status {
                                    GitFileStatusCode::Modified => ("M", color_modified()),
                                    GitFileStatusCode::Added => ("A", color_added()),
                                    GitFileStatusCode::Deleted => ("D", color_deleted()),
                                    GitFileStatusCode::Renamed => ("R", color_renamed()),
                                    GitFileStatusCode::Untracked => ("U", color_untracked()),
                                    GitFileStatusCode::Conflicted => ("C", color_conflicted()),
                                };

                                let is_untracked = item.status == GitFileStatusCode::Untracked;

                                let right_reserved = if item_hov { 60.0 } else { 24.0 };
                                let left_max_w = (panel_w - right_reserved - 20.0).max(25.0);

                                let row = ui.container()
                                    .clickable(true)
                                    .full_width()
                                    .height(22.0)
                                    .padding_left(12.0)
                                    .padding_right(8.0)
                                    .bg(if item_hov { colors.highlight } else { Color::TRANSPARENT })
                                    .row()
                                    .valign(Align::Center)
                                    .halign(Align::SpaceBetween)
                                    .no_wrap()
                                    .show(|ui| {
                                        // Left: icon + filename + dynamic dir path (only truncates when shrinking close)
                                        ui.container()
                                            .max_width(left_max_w)
                                            .clip(true)
                                            .row()
                                            .valign(Align::Center)
                                            .no_wrap()
                                            .gap(5.0)
                                            .show(|ui| {
                                                icon.show(ui, 13.0);

                                                ui.text(&item.filename)
                                                    .size(11.5)
                                                    .color(colors.text_primary)
                                                    .show();

                                                if !item.dir_path.is_empty() {
                                                    let filename_w = item.filename.len() as f32 * 6.5 + 23.0;
                                                    let avail_dir_w = left_max_w - filename_w;
                                                    if avail_dir_w > 20.0 {
                                                        let max_chars = (avail_dir_w / 5.2) as usize;
                                                        let display_dir = if item.dir_path.len() <= max_chars {
                                                            item.dir_path.clone()
                                                        } else if max_chars >= 5 {
                                                            format!("{}…", &item.dir_path[..max_chars.saturating_sub(1)])
                                                        } else {
                                                             String::new()
                                                        };

                                                        if !display_dir.is_empty() {
                                                            ui.text(&display_dir)
                                                                .size(9.5)
                                                                .color(colors.text_dim)
                                                                .show();
                                                        }
                                                    }
                                                }
                                            });

                                        // Right: status badge or hover action buttons (Discard + Stage)
                                        ui.container()
                                            .row()
                                            .valign(Align::Center)
                                            .no_wrap()
                                            .gap(4.0)
                                            .show(|ui| {
                                                if item_hov {
                                                    // Discard button (\u{f0e2} undo icon)
                                                    let discard_btn = ui.button("\u{f0e2}")
                                                        .id(Id::from_u64(999998700 + idx as u64))
                                                        .height(18.0)
                                                        .bg(Color::TRANSPARENT)
                                                        .text_color(colors.text_muted)
                                                        .size(9.0)
                                                        .radius_all(3.0)
                                                        .padding_left(3.0)
                                                        .padding_right(3.0)
                                                        .show();
                                                    if discard_btn.clicked {
                                                        file_to_discard = Some((item.path.clone(), is_untracked));
                                                    }

                                                    // Stage button (\u{f067} plus icon)
                                                    let stage_btn = ui.button("\u{f067}")
                                                        .id(Id::from_u64(999998800 + idx as u64))
                                                        .height(18.0)
                                                        .bg(Color::TRANSPARENT)
                                                        .text_color(colors.text_muted)
                                                        .size(9.5)
                                                        .radius_all(3.0)
                                                        .padding_left(3.0)
                                                        .padding_right(3.0)
                                                        .show();
                                                    if stage_btn.clicked {
                                                        file_to_stage = Some(item.path.clone());
                                                    }
                                                }

                                                ui.text(badge_char)
                                                    .size(10.5)
                                                    .weight(FontWeight::Bold)
                                                    .color(badge_color)
                                                    .show();
                                            });
                                    });

                                let h = row.hovered;
                                let prev = ui.interaction_state.insert(item_key, if h { 1.0 } else { 0.0 });
                                if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                                if row.clicked {
                                    file_to_open = Some(item.path.clone());
                                }
                            }
                        }
                    }

                    // Clean working tree banner
                    if staged_len == 0 && unstaged_len == 0 {
                        ui.container()
                            .full_width()
                            .padding(16.0, 16.0, 16.0, 16.0)
                            .column()
                            .halign(Align::Center)
                            .gap(5.0)
                            .show(|ui| {
                                ui.text("\u{f00c}") // check
                                    .size(20.0)
                                    .color(colors.accent.with_alpha(0.7))
                                    .show();

                                ui.text("Working Tree Clean")
                                    .size(11.5)
                                    .weight(FontWeight::Bold)
                                    .color(colors.text_primary)
                                    .show();

                                ui.text("No changes detected in repository")
                                    .size(10.0)
                                    .color(colors.text_dim)
                                    .show();
                            });
                    }
                });

            // 2. GRAPH Panel (at bottom, with size increase holder like left panel / terminal)
            if graph_expanded {
                let splitter_h = 6.0;
                let splitter_x = ui.cursor_x;
                let splitter_y = ui.cursor_y;
                let splitter_w = panel_w;
                let is_splitter_hovered = ui.mouse_in_rect(
                    splitter_x,
                    splitter_y - splitter_h / 2.0,
                    splitter_w,
                    splitter_h,
                );

                if is_splitter_hovered || state.git_state.active_resize_graph {
                    ui.cursor_icon = CursorIcon::RowResize;
                }

                if is_splitter_hovered && ui.mouse_down && ui.active_drag.is_none() {
                    state.git_state.active_resize_graph = true;
                }

                if state.git_state.active_resize_graph {
                    if !ui.mouse_down {
                        state.git_state.active_resize_graph = false;
                        state.save_persistent_state();
                    } else {
                        let status_bar_h = if state.status_bar_visible { 24.0 } else { 0.0 };
                        let bottom_y = ui.height - status_bar_h;
                        let target_h = bottom_y - ui.mouse_y - 24.0;
                        state.git_state.graph_height = target_h.clamp(80.0, (ui.height * 0.7).max(120.0));
                        ui.request_redraw();
                    }
                }

                let edge_color = if state.git_state.active_resize_graph {
                    colors.accent
                } else if is_splitter_hovered {
                    colors.accent.with_alpha(0.8)
                } else {
                    colors.border
                };

                let edge_thickness = if state.git_state.active_resize_graph || is_splitter_hovered { 2.0 } else { 1.0 };
                let edge_draw_y = splitter_y - edge_thickness / 2.0;

                ui.draws.push(zenthra_widgets::ui::DrawCommand::Rect(zenthra_widgets::ui::RectDraw {
                    instance: zenthra_render::RectInstance {
                        pos: [splitter_x, edge_draw_y],
                        size: [splitter_w, edge_thickness],
                        color: edge_color.to_array(),
                        radius: [0.0; 4],
                        border_width: 0.0,
                        border_color: [0.0; 4],
                        shadow_color: [0.0; 4],
                        shadow_offset: [0.0, 0.0],
                        shadow_blur: 0.0,
                        clip_rect: [-100000.0, -100000.0, 2000000.0, 2000000.0],
                        grayscale: 0.0,
                        brightness: 1.0,
                        opacity: 1.0,
                        border_alignment: 0.0,
                    }
                }));
            }

            let mut toggle_graph = false;
            let repo_display_name = state.current_dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "repo".to_string());

            let graph_hdr_key = Id::from_u64(999998398);
            let graph_hdr_hov = ui.interaction_state.get(&graph_hdr_key).copied().unwrap_or(0.0) > 0.5;

            // Accordion header: "v Graph   repo" (sleek dark bar, highlights on hover)
            ui.container()
                .full_width()
                .height(24.0)
                .padding_left(8.0)
                .padding_right(8.0)
                .bg(if graph_hdr_hov { colors.highlight.with_alpha(0.6) } else { Color::TRANSPARENT })
                .row()
                .valign(Align::Center)
                .halign(Align::SpaceBetween)
                .no_wrap()
                .show(|ui| {
                    let hdr_btn = ui.container()
                        .raw_id(graph_hdr_key)
                        .clickable(true)
                        .row()
                        .valign(Align::Center)
                        .no_wrap()
                        .gap(6.0)
                        .show(|ui| {
                            ui.text(if graph_expanded { "\u{f078}" } else { "\u{f054}" })
                                .size(9.0)
                                .color(if graph_hdr_hov { colors.text_primary } else { colors.text_muted })
                                .show();

                            ui.text("Graph")
                                .size(11.0)
                                .weight(FontWeight::Bold)
                                .color(if graph_hdr_hov { colors.text_primary } else { colors.text_muted })
                                .show();

                            // Folder / repo badge
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .gap(3.0)
                                .show(|ui| {
                                    ui.text(zenthra::icons::NF_FA_FOLDER)
                                        .size(10.0)
                                        .color(if graph_hdr_hov { colors.text_muted } else { colors.text_dim })
                                        .show();

                                    ui.text(&repo_display_name)
                                        .size(10.0)
                                        .color(if graph_hdr_hov { colors.text_muted } else { colors.text_dim })
                                        .show();
                                });
                        });

                    let h = hdr_btn.hovered;
                    let prev = ui.interaction_state.insert(graph_hdr_key, if h { 1.0 } else { 0.0 });
                    if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                    if hdr_btn.clicked {
                        toggle_graph = true;
                    }

                    // Refresh button on graph header
                    let ref_graph_btn = ui.button("\u{f01e}")
                        .id(Id::from_u64(999998399))
                        .height(18.0)
                        .bg(Color::TRANSPARENT)
                        .text_color(if graph_hdr_hov { colors.text_primary } else { colors.text_muted })
                        .size(9.5)
                        .radius_all(3.0)
                        .padding_left(3.0)
                        .padding_right(3.0)
                        .show();
                    if ref_graph_btn.clicked {
                        state.refresh_git_status();
                        ui.request_redraw();
                    }
                });

            if toggle_graph {
                state.git_state.graph_expanded = !state.git_state.graph_expanded;
                ui.request_redraw();
            }

            if graph_expanded {
                ui.container()
                    .full_width()
                    .height(state.git_state.graph_height)
                    .scroll_y(true)
                    .clip(true)
                    .column()
                    .show(|ui| {
                        let history_len = state.git_state.commit_history.len();
                        let mut clicked_commit_info: Option<(String, String, String, String)> = None;
                        for (idx, commit) in state.git_state.commit_history.iter().enumerate() {
                            let commit_key = Id::from_u64(999999200 + idx as u64);
                            let commit_hov = ui.interaction_state.get(&commit_key).copied().unwrap_or(0.0) > 0.5;

                            let lane = commit.lane;
                            let lane_color = get_graph_color(lane);
                            let is_head = commit.is_head;
                            let is_first = idx == 0;
                            let is_last = idx == history_len.saturating_sub(1);

                            let commit_row = ui.container()
                                .clickable(true)
                                .full_width()
                                .height(22.0)
                                .padding_left(6.0)
                                .padding_right(6.0)
                                .bg(if commit_hov { colors.highlight } else { Color::TRANSPARENT })
                                .row()
                                .valign(Align::Center)
                                .halign(Align::SpaceBetween)
                                .no_wrap()
                                .show(|ui| {
                                    // Left: Connected Graph Tracks + Node + Hash + Commit Message
                                    let content_max_w = (panel_w - 20.0).max(40.0);
                                    ui.container()
                                        .max_width(content_max_w)
                                        .clip(true)
                                        .row()
                                        .valign(Align::Center)
                                        .no_wrap()
                                        .gap(6.0)
                                        .show(|ui| {
                                            // Graph column with vertical connector rails
                                            let graph_col_w = (lane as f32 + 1.0) * 14.0 + 2.0;
                                            ui.container()
                                                .width(graph_col_w)
                                                .height(22.0)
                                                .row()
                                                .valign(Align::Center)
                                                .show(|ui| {
                                                    // Pass-through vertical rails for active lanes to the left
                                                    for l in 0..lane {
                                                        let col = get_graph_color(l);
                                                        ui.container()
                                                            .width(14.0)
                                                            .height(22.0)
                                                            .halign(Align::Center)
                                                            .show(|ui| {
                                                                ui.container()
                                                                    .width(1.5)
                                                                    .height(22.0)
                                                                    .bg(col.with_alpha(0.55))
                                                                    .show(|_| {});
                                                            });
                                                    }

                                                    // Commit node and connecting rails on current lane
                                                    ui.container()
                                                        .width(14.0)
                                                        .height(22.0)
                                                        .column()
                                                        .halign(Align::Center)
                                                        .valign(Align::Center)
                                                        .gap(0.0)
                                                        .show(|ui| {
                                                            // Top connecting rail
                                                            ui.container()
                                                                .width(1.5)
                                                                .height(5.0)
                                                                .bg(if is_first { Color::TRANSPARENT } else { lane_color.with_alpha(0.7) })
                                                                .show(|_| {});

                                                            // Node glyph (ring for HEAD, circle for commit)
                                                            ui.container()
                                                                .width(14.0)
                                                                .height(12.0)
                                                                .halign(Align::Center)
                                                                .valign(Align::Center)
                                                                .show(|ui| {
                                                                    ui.text(if is_head { "\u{25ce}" } else { "\u{25cf}" })
                                                                        .size(if is_head { 11.5 } else { 9.0 })
                                                                        .color(if is_head { lane_color } else { lane_color.with_alpha(0.95) })
                                                                        .show();
                                                                });

                                                            // Bottom connecting rail
                                                            ui.container()
                                                                .width(1.5)
                                                                .height(5.0)
                                                                .bg(if is_last { Color::TRANSPARENT } else { lane_color.with_alpha(0.7) })
                                                                .show(|_| {});
                                                        });
                                                });

                                            // Short commit hash in lane color
                                            ui.text(&commit.short_hash)
                                                .size(9.5)
                                                .color(lane_color)
                                                .show();

                                            // Commit message
                                            let msg_color = if is_head {
                                                colors.text_primary
                                            } else {
                                                colors.text_muted
                                            };

                                            ui.text(&commit.message)
                                                .size(11.0)
                                                .color(msg_color)
                                                .show();

                                            // Branch tag pill on HEAD or referenced commits
                                            if is_head && !state.git_state.branch.is_empty() {
                                                ui.container()
                                                    .height(15.0)
                                                    .radius_all(3.0)
                                                    .bg(lane_color.with_alpha(0.18))
                                                    .border(lane_color.with_alpha(0.4), 1.0)
                                                    .padding(0.0, 4.0, 0.0, 4.0)
                                                    .row()
                                                    .valign(Align::Center)
                                                    .gap(2.5)
                                                    .show(|ui| {
                                                        ui.text("\u{f192}") // target/disc icon
                                                            .size(8.0)
                                                            .color(lane_color)
                                                            .show();

                                                        ui.text(&state.git_state.branch)
                                                            .size(9.0)
                                                            .weight(FontWeight::Bold)
                                                            .color(lane_color)
                                                            .show();
                                                    });
                                            } else if let Some(ref_name) = commit.branch_refs.first() {
                                                let clean_ref = if ref_name.len() > 12 {
                                                    format!("{}...", &ref_name[..10])
                                                } else {
                                                    ref_name.clone()
                                                };
                                                ui.container()
                                                    .height(14.0)
                                                    .radius_all(2.5)
                                                    .bg(lane_color.with_alpha(0.15))
                                                    .padding(0.0, 3.5, 0.0, 3.5)
                                                    .show(|ui| {
                                                        ui.text(&clean_ref)
                                                            .size(8.5)
                                                            .color(lane_color)
                                                            .show();
                                                    });
                                            }
                                        });
                                });

                            let h = commit_row.hovered;
                            let prev = ui.interaction_state.insert(commit_key, if h { 1.0 } else { 0.0 });
                            if prev != Some(if h { 1.0 } else { 0.0 }) { ui.needs_redraw = true; }

                            if commit_row.clicked {
                                clicked_commit_info = Some((commit.short_hash.clone(), commit.message.clone(), commit.time_ago.clone(), commit.hash.clone()));
                            }
                        }

                        if let Some((short_h, msg, time, full_hash)) = clicked_commit_info {
                            state.status_message = format!("Commit {}: {} ({})", short_h, msg, time);
                            state.set_clipboard(full_hash);
                            ui.request_redraw();
                        }
                    });
            }

            // Handle deferred actions
            if let Some(path) = file_to_open {
                state.open_file(&path);
                ui.request_redraw();
            }
            if let Some(path) = file_to_stage {
                state.git_stage_file(&path);
                ui.request_redraw();
            }
            if let Some(path) = file_to_unstage {
                state.git_unstage_file(&path);
                ui.request_redraw();
            }
            if let Some((path, is_untracked)) = file_to_discard {
                state.git_discard_file(&path, is_untracked);
                ui.request_redraw();
            }
        });
}

fn draw_empty_no_workspace(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = zenthra::use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    ui.container()
        .full_width()
        .fill_y()
        .padding(32.0, 16.0, 16.0, 16.0)
        .column()
        .halign(Align::Center)
        .gap(10.0)
        .show(|ui| {
            ui.text(zenthra::icons::NF_FA_FOLDER)
                .size(32.0)
                .color(colors.text_dim)
                .show();

            ui.text("No Folder Open")
                .size(13.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();

            ui.text("Open a folder or repository to view source control.")
                .size(11.0)
                .color(colors.text_muted)
                .show();

            let open_btn = ui.button("Open Folder")
                .id(Id::from_u64(999998350))
                .height(28.0)
                .bg(colors.accent)
                .text_color(Color::WHITE)
                .radius_all(4.0)
                .padding_left(12.0)
                .padding_right(12.0)
                .show();

            if open_btn.clicked {
                if let Some(path) = rfd::FileDialog::new().pick_folder() {
                    state.open_workspace(path);
                    ui.request_redraw();
                }
            }
        });
}

fn draw_empty_not_a_repo(ui: &mut Ui, state: &mut ZenthreeState) {
    let colors = zenthra::use_context::<crate::theme::ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());

    ui.container()
        .full_width()
        .fill_y()
        .padding(32.0, 16.0, 16.0, 16.0)
        .column()
        .halign(Align::Center)
        .gap(10.0)
        .show(|ui| {
            // Source control icon (\u{ea68})
            ui.text("\u{ea68}")
                .size(32.0)
                .color(colors.text_dim)
                .show();

            ui.text("No Git Repository")
                .size(13.0)
                .weight(FontWeight::Bold)
                .color(colors.text_primary)
                .show();

            ui.text("The open folder is not a Git repository.")
                .size(11.0)
                .color(colors.text_muted)
                .show();

            let init_btn = ui.button("Initialize Repository")
                .id(Id::from_u64(999998351))
                .height(28.0)
                .bg(colors.accent)
                .text_color(Color::WHITE)
                .radius_all(4.0)
                .padding_left(12.0)
                .padding_right(12.0)
                .show();

            if init_btn.clicked {
                state.git_init();
                ui.request_redraw();
            }
        });
}
