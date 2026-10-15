use crate::state::ZenthreeState;
use zenthra::{Color, Ui, Align, FontWeight, Id, Style, use_context};
use crate::theme::ThemeColors;
use zenthra_widgets::ui::{DrawCommand, OverlayRectDraw};
use std::path::PathBuf;

impl ZenthreeState {
    /// Executes a command entered in the LazyVim command line dialog.
    pub fn execute_cmdline(&mut self, raw_cmd: &str) -> Result<String, String> {
        let trimmed = raw_cmd.trim();
        let cmd = trimmed.strip_prefix(':').unwrap_or(trimmed).trim();

        if cmd.is_empty() {
            self.close_cmdline();
            return Ok(String::new());
        }

        // ── 1. Directory Change: `cd <path>` or `cd: <path>` ──────────────
        let is_cd = cmd.to_lowercase().starts_with("cd:")
            || cmd.to_lowercase().starts_with("cd ")
            || cmd.eq_ignore_ascii_case("cd");

        if is_cd {
            let target_str = if cmd.to_lowercase().starts_with("cd:") {
                cmd[3..].trim()
            } else if cmd.to_lowercase().starts_with("cd ") {
                cmd[3..].trim()
            } else {
                ""
            };

            let resolved: PathBuf = if target_str.is_empty() || target_str == "~" {
                dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))
            } else if target_str.starts_with("~/") {
                if let Some(home) = dirs::home_dir() {
                    home.join(&target_str[2..])
                } else {
                    PathBuf::from(target_str)
                }
            } else if target_str == ".." {
                self.current_dir.parent().unwrap_or(&self.current_dir).to_path_buf()
            } else if target_str == "." {
                self.current_dir.clone()
            } else {
                let p = PathBuf::from(target_str);
                if p.is_absolute() {
                    p
                } else {
                    self.current_dir.join(p)
                }
            };

            if resolved.is_dir() {
                let canon = resolved.canonicalize().unwrap_or(resolved);
                self.current_dir = canon.clone();
                self.has_workspace = true;
                self.expanded_dirs.clear();
                self.rebuild_tree();
                self.refresh_git_status();
                self.fs_watcher.watch(&self.current_dir);
                self.notes_state.scan_notes(&self.current_dir);

                // Keep terminal session working directory in sync
                if let Some(ref mut sess) = self.terminal_session {
                    let clean = crate::native_terminal::strip_unc_prefix(&canon);
                    sess.write_raw(format!("cd \"{}\"\r", clean.display()).as_bytes());
                }

                let msg = format!("Directory changed to: {}", canon.display());
                self.set_status(&msg);
                self.save_persistent_state();
                self.close_cmdline();
                return Ok(msg);
            } else {
                return Err(format!("Directory not found: {}", target_str));
            }
        }

        // ── 2. Create File: `touch <path>` or `file <path>` or `new <path>` ─
        let is_touch = cmd.starts_with("touch ")
            || cmd.starts_with("file ")
            || cmd.starts_with("new ");

        if is_touch {
            let path_part = if cmd.starts_with("touch ") {
                cmd[6..].trim()
            } else if cmd.starts_with("file ") {
                cmd[5..].trim()
            } else {
                cmd[4..].trim()
            };

            if path_part.is_empty() {
                return Err("Please specify a filename".to_string());
            }

            return self.create_file_from_cmd(path_part);
        }

        // ── 3. Create Folder: `mkdir <path>` or `folder <path>` ───────────
        let is_mkdir = cmd.starts_with("mkdir ") || cmd.starts_with("folder ");

        if is_mkdir {
            let path_part = if cmd.starts_with("mkdir ") {
                cmd[6..].trim()
            } else {
                cmd[7..].trim()
            };

            if path_part.is_empty() {
                return Err("Please specify a directory name".to_string());
            }

            return self.create_folder_from_cmd(path_part);
        }

        // ── 4. Terminal Command: `!<cmd>` or `:!<cmd>` or `term` ──────────
        if cmd.starts_with('!') {
            let shell_cmd = cmd[1..].trim();
            self.ensure_terminal_session();
            self.terminal_visible = true;
            self.focused_panel = crate::state::types::FocusedPanel::Terminal;
            if !shell_cmd.is_empty() {
                if let Some(ref mut sess) = self.terminal_session {
                    sess.write_raw(format!("{}\r", shell_cmd).as_bytes());
                }
            }
            self.set_status(format!("Running: {}", shell_cmd));
            self.close_cmdline();
            return Ok(format!("Sent to terminal: {}", shell_cmd));
        }

        if cmd.eq_ignore_ascii_case("term") || cmd.eq_ignore_ascii_case("terminal") {
            self.ensure_terminal_session();
            self.terminal_visible = true;
            self.focused_panel = crate::state::types::FocusedPanel::Terminal;
            self.close_cmdline();
            return Ok("Terminal opened".to_string());
        }

        // ── 5. Standard Vim Commands (:w, :q, :wq, :e) ────────────────────
        if cmd == "w" || cmd.eq_ignore_ascii_case("save") {
            self.save_active_tab();
            self.close_cmdline();
            return Ok("Saved active tab".to_string());
        }

        if cmd == "q" || cmd.eq_ignore_ascii_case("quit") {
            if let Some(idx) = self.active_tab {
                self.close_tab(idx);
            }
            self.close_cmdline();
            return Ok("Closed active tab".to_string());
        }

        if cmd == "wq" || cmd == "x" {
            self.save_active_tab();
            if let Some(idx) = self.active_tab {
                self.close_tab(idx);
            }
            self.close_cmdline();
            return Ok("Saved and closed tab".to_string());
        }

        if cmd.starts_with("e ") || cmd.starts_with("open ") {
            let path_part = if cmd.starts_with("e ") {
                cmd[2..].trim()
            } else {
                cmd[5..].trim()
            };

            let p = PathBuf::from(path_part);
            let full_p = if p.is_absolute() {
                p
            } else {
                self.current_dir.join(p)
            };

            if full_p.exists() {
                self.open_file(&full_p);
                self.close_cmdline();
                return Ok(format!("Opened {}", full_p.display()));
            } else {
                return self.create_file_from_cmd(path_part);
            }
        }

        // ── 6. Smart Fallback: directory / folder / file ───────────────────
        if cmd.ends_with('/') || cmd.ends_with('\\') {
            return self.create_folder_from_cmd(cmd.trim_end_matches(['/', '\\']));
        }

        // Check if user typed a directory name directly (e.g. "Desktop" or "src")
        let candidate_dir = self.current_dir.join(cmd);
        if candidate_dir.is_dir() {
            let canon = candidate_dir.canonicalize().unwrap_or(candidate_dir);
            self.current_dir = canon.clone();
            self.has_workspace = true;
            self.expanded_dirs.clear();
            self.rebuild_tree();
            self.refresh_git_status();
            self.fs_watcher.watch(&self.current_dir);

            if let Some(ref mut sess) = self.terminal_session {
                let clean = crate::native_terminal::strip_unc_prefix(&canon);
                sess.write_raw(format!("cd \"{}\"\r", clean.display()).as_bytes());
            }

            let msg = format!("Directory changed to: {}", canon.display());
            self.set_status(&msg);
            self.save_persistent_state();
            self.close_cmdline();
            return Ok(msg);
        }

        // If it's an existing file, open it
        if candidate_dir.is_file() {
            self.open_file(&candidate_dir);
            self.close_cmdline();
            return Ok(format!("Opened {}", candidate_dir.display()));
        }

        // Otherwise treat as create file
        self.create_file_from_cmd(cmd)
    }

    fn create_file_from_cmd(&mut self, path_part: &str) -> Result<String, String> {
        let base_dir = self.cmdline_dialog.target_dir.clone().unwrap_or_else(|| self.current_dir.clone());
        let p = PathBuf::from(path_part);
        let full_path = if p.is_absolute() { p } else { base_dir.join(p) };

        if let Some(parent) = full_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        if !full_path.exists() {
            if let Err(e) = std::fs::write(&full_path, "") {
                return Err(format!("Failed to create file: {}", e));
            }
        }

        self.rebuild_tree();
        self.open_file(&full_path);
        let msg = format!("Created {}", full_path.display());
        self.set_status(&msg);
        self.close_cmdline();
        Ok(msg)
    }

    fn create_folder_from_cmd(&mut self, path_part: &str) -> Result<String, String> {
        let base_dir = self.cmdline_dialog.target_dir.clone().unwrap_or_else(|| self.current_dir.clone());
        let p = PathBuf::from(path_part);
        let full_path = if p.is_absolute() { p } else { base_dir.join(p) };

        if let Err(e) = std::fs::create_dir_all(&full_path) {
            return Err(format!("Failed to create folder: {}", e));
        }

        self.expanded_dirs.insert(full_path.clone());
        self.selected_sidebar_path = Some(full_path.clone());
        self.rebuild_tree();
        let msg = format!("Created folder {}", full_path.display());
        self.set_status(&msg);
        self.close_cmdline();
        Ok(msg)
    }
    /// Dynamically scans and updates directory suggestions when typing `cd` commands.
    pub fn update_cmdline_suggestions(&mut self) {
        let trimmed_start = self.cmdline_dialog.input.trim_start();
        let cmd = trimmed_start.strip_prefix(':').unwrap_or(trimmed_start).trim_start();

        // Check if command is `cd`
        let is_cd = cmd.to_lowercase().starts_with("cd ") || cmd.to_lowercase().starts_with("cd:");
        if !is_cd {
            self.cmdline_dialog.suggestions.clear();
            self.cmdline_dialog.selected_suggestion = None;
            return;
        }

        let raw_path = if cmd.to_lowercase().starts_with("cd:") {
            cmd[3..].trim_start()
        } else {
            cmd[3..].trim_start()
        };

        // Determine base_dir and prefix filter
        let (base_dir, file_prefix) = if raw_path.is_empty() {
            (self.current_dir.clone(), String::new())
        } else if raw_path == "~" || raw_path == "~/" {
            (dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")), String::new())
        } else if raw_path.starts_with("~/") {
            let sub = &raw_path[2..];
            if let Some(last_slash) = sub.rfind('/') {
                let dir_sub = &sub[..last_slash];
                let prefix = &sub[last_slash + 1..];
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                (home.join(dir_sub), prefix.to_string())
            } else {
                let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
                (home, sub.to_string())
            }
        } else if raw_path.starts_with('/') {
            let p = PathBuf::from(raw_path);
            if raw_path.ends_with('/') {
                (p, String::new())
            } else if let Some(last_slash) = raw_path.rfind('/') {
                let dir_part = &raw_path[..last_slash];
                let prefix = &raw_path[last_slash + 1..];
                (if dir_part.is_empty() { PathBuf::from("/") } else { PathBuf::from(dir_part) }, prefix.to_string())
            } else {
                (PathBuf::from("/"), raw_path.to_string())
            }
        } else {
            // Relative path
            if raw_path.ends_with('/') {
                (self.current_dir.join(raw_path), String::new())
            } else if let Some(last_slash) = raw_path.rfind('/') {
                let dir_part = &raw_path[..last_slash];
                let prefix = &raw_path[last_slash + 1..];
                (self.current_dir.join(dir_part), prefix.to_string())
            } else {
                (self.current_dir.clone(), raw_path.to_string())
            }
        };

        let mut entries = Vec::new();
        if let Ok(read_dir) = std::fs::read_dir(&base_dir) {
            for entry_res in read_dir {
                if let Ok(entry) = entry_res {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            // Don't show hidden unless prefix explicitly starts with '.'
                            if !file_prefix.starts_with('.') && name.starts_with('.') {
                                continue;
                            }
                            if file_prefix.is_empty()
                                || name.to_lowercase().starts_with(&file_prefix.to_lowercase())
                            {
                                entries.push(format!("{}/", name));
                            }
                        }
                    }
                }
            }
        }

        entries.sort_by(|a, b| a.to_lowercase().cmp(&b.to_lowercase()));
        self.cmdline_dialog.suggestions = entries;
        if let Some(idx) = self.cmdline_dialog.selected_suggestion {
            if idx >= self.cmdline_dialog.suggestions.len() {
                self.cmdline_dialog.selected_suggestion = None;
            }
        }
    }

    /// Autocompletes a selected folder suggestion into the cmdline input.
    pub fn complete_cmdline_suggestion(&mut self, chosen_folder: &str) {
        let trimmed_start = self.cmdline_dialog.input.trim_start();
        let has_colon = trimmed_start.starts_with(':');
        let cmd = trimmed_start.strip_prefix(':').unwrap_or(trimmed_start).trim_start();

        let (prefix_cmd, raw_path) = if cmd.to_lowercase().starts_with("cd:") {
            ("cd: ", cmd[3..].trim_start())
        } else if cmd.to_lowercase().starts_with("cd ") {
            ("cd ", cmd[3..].trim_start())
        } else {
            return;
        };

        let new_path = if raw_path.is_empty() {
            chosen_folder.to_string()
        } else if raw_path == "~" || raw_path == "~/" {
            format!("~/{}", chosen_folder)
        } else if raw_path.starts_with("~/") {
            let sub = &raw_path[2..];
            if let Some(last_slash) = sub.rfind('/') {
                format!("~/{}{}", &sub[..=last_slash], chosen_folder)
            } else {
                format!("~/{}", chosen_folder)
            }
        } else if raw_path.starts_with('/') {
            if let Some(last_slash) = raw_path.rfind('/') {
                format!("{}{}", &raw_path[..=last_slash], chosen_folder)
            } else {
                format!("/{}", chosen_folder)
            }
        } else {
            if let Some(last_slash) = raw_path.rfind('/') {
                format!("{}{}", &raw_path[..=last_slash], chosen_folder)
            } else {
                chosen_folder.to_string()
            }
        };

        let colon_prefix = if has_colon { ":" } else { "" };
        self.cmdline_dialog.input = format!("{}{}{}", colon_prefix, prefix_cmd, new_path);
        self.cmdline_dialog.cursor_idx = self.cmdline_dialog.input.len();
        self.cmdline_dialog.selected_suggestion = None;
        self.update_cmdline_suggestions();
    }
}

pub fn draw_cmdline_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.cmdline_dialog.open {
        return;
    }

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let viewport = ui.current_viewport;

    // Floating LazyVim-style Cmdline bar
    let card_w = 480.0f32.min(viewport.size.width - 32.0);
    let center_x = viewport.origin.x + (viewport.size.width - card_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height * 0.18).max(40.0);

    let modal_id = Id::from_u64(999998200);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_active = true;
    ui.modal_window_id = Some(modal_id);
    ui.current_window_id = Some(modal_id);

    // 1. Semi-transparent backdrop overlay
    ui.draws.push(DrawCommand::OverlayRect(OverlayRectDraw {
        x: viewport.origin.x,
        y: viewport.origin.y,
        width: viewport.size.width,
        height: viewport.size.height,
        color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 },
        clip: [0.0, 0.0, 99999.0, 99999.0],
    }));

    let mut do_execute = false;
    let mut do_dismiss = false;

    // 2. Event processing for text input, shortcuts, and suggestion navigation
    let mut events = std::mem::take(&mut ui.input_events);
    events.retain(|event| {
        match event {
            zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                match key {
                    winit::keyboard::KeyCode::Escape => {
                        do_dismiss = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyC if state.ctrl_pressed => {
                        do_dismiss = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowDown | winit::keyboard::KeyCode::KeyJ if state.ctrl_pressed || *key == winit::keyboard::KeyCode::ArrowDown => {
                        if !state.cmdline_dialog.suggestions.is_empty() {
                            let n = state.cmdline_dialog.suggestions.len();
                            state.cmdline_dialog.selected_suggestion = Some(match state.cmdline_dialog.selected_suggestion {
                                Some(idx) => (idx + 1) % n,
                                None => 0,
                            });
                            ui.needs_redraw = true;
                            return false;
                        }
                    }
                    winit::keyboard::KeyCode::ArrowUp | winit::keyboard::KeyCode::KeyK if state.ctrl_pressed || *key == winit::keyboard::KeyCode::ArrowUp => {
                        if !state.cmdline_dialog.suggestions.is_empty() {
                            let n = state.cmdline_dialog.suggestions.len();
                            state.cmdline_dialog.selected_suggestion = Some(match state.cmdline_dialog.selected_suggestion {
                                Some(idx) => (idx + n - 1) % n,
                                None => n - 1,
                            });
                            ui.needs_redraw = true;
                            return false;
                        }
                    }
                    winit::keyboard::KeyCode::Tab => {
                        if !state.cmdline_dialog.suggestions.is_empty() {
                            if let Some(idx) = state.cmdline_dialog.selected_suggestion {
                                if let Some(item) = state.cmdline_dialog.suggestions.get(idx).cloned() {
                                    state.complete_cmdline_suggestion(&item);
                                }
                            } else if state.cmdline_dialog.suggestions.len() == 1 {
                                let item = state.cmdline_dialog.suggestions[0].clone();
                                state.complete_cmdline_suggestion(&item);
                            } else {
                                state.cmdline_dialog.selected_suggestion = Some(0);
                            }
                            ui.needs_redraw = true;
                            return false;
                        }
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if !state.cmdline_dialog.suggestions.is_empty() {
                            if let Some(idx) = state.cmdline_dialog.selected_suggestion {
                                if let Some(item) = state.cmdline_dialog.suggestions.get(idx).cloned() {
                                    state.complete_cmdline_suggestion(&item);
                                }
                            }
                        }
                        do_execute = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        if state.cmdline_dialog.cursor_idx > 0 && !state.cmdline_dialog.input.is_empty() {
                            state.cmdline_dialog.input.remove(state.cmdline_dialog.cursor_idx - 1);
                            state.cmdline_dialog.cursor_idx -= 1;
                            state.cmdline_dialog.error_msg = None;
                            state.update_cmdline_suggestions();
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Delete => {
                        if state.cmdline_dialog.cursor_idx < state.cmdline_dialog.input.len() {
                            state.cmdline_dialog.input.remove(state.cmdline_dialog.cursor_idx);
                            state.cmdline_dialog.error_msg = None;
                            state.update_cmdline_suggestions();
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowLeft => {
                        if state.cmdline_dialog.cursor_idx > 0 {
                            state.cmdline_dialog.cursor_idx -= 1;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowRight => {
                        if state.cmdline_dialog.cursor_idx < state.cmdline_dialog.input.len() {
                            state.cmdline_dialog.cursor_idx += 1;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Home => {
                        state.cmdline_dialog.cursor_idx = 0;
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::End => {
                        state.cmdline_dialog.cursor_idx = state.cmdline_dialog.input.len();
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyV if state.ctrl_pressed => {
                        if let Some(text) = state.get_clipboard() {
                            let clean_text = text.replace('\n', " ").replace('\r', "");
                            state.cmdline_dialog.input.insert_str(state.cmdline_dialog.cursor_idx, &clean_text);
                            state.cmdline_dialog.cursor_idx += clean_text.len();
                            state.cmdline_dialog.error_msg = None;
                            state.update_cmdline_suggestions();
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyU if state.ctrl_pressed => {
                        state.cmdline_dialog.input.clear();
                        state.cmdline_dialog.cursor_idx = 0;
                        state.cmdline_dialog.error_msg = None;
                        state.update_cmdline_suggestions();
                        ui.needs_redraw = true;
                        return false;
                    }
                    _ => {}
                }
            }
            zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                    state.cmdline_dialog.input.insert(state.cmdline_dialog.cursor_idx, *c);
                    state.cmdline_dialog.cursor_idx += c.len_utf8();
                    state.cmdline_dialog.error_msg = None;
                    state.update_cmdline_suggestions();
                    ui.needs_redraw = true;
                    return false;
                }
            }
            _ => {}
        }
        true
    });
    ui.input_events = events;

    // Dropdown geometry for click hit-testing
    let dropdown_x = center_x + 28.0;
    let dropdown_y = center_y + 58.0;
    let dropdown_w = 230.0f32;
    let item_h = 24.0f32;
    let max_visible = 10;
    let count = state.cmdline_dialog.suggestions.len();
    let dropdown_h = (count as f32 * item_h + 8.0).min(max_visible as f32 * item_h + 8.0);

    // 3. Outside click dismiss (dialog card or dropdown)
    if state.cmdline_dialog.just_opened {
        state.cmdline_dialog.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, card_w, 62.0)
            || (!state.cmdline_dialog.suggestions.is_empty()
                && ui.mouse_in_rect(dropdown_x, dropdown_y, dropdown_w, dropdown_h));
        if ui.clicked && !mouse_in_dialog {
            do_dismiss = true;
            ui.consume_click();
        }
    }

    if do_dismiss {
        state.close_cmdline();
        ui.interaction_state.remove(&modal_key);
        ui.modal_active = false;
        ui.modal_window_id = None;
        ui.needs_redraw = true;
        return;
    }

    if do_execute {
        let cmd = state.cmdline_dialog.input.clone();
        match state.execute_cmdline(&cmd) {
            Ok(_) => {
                ui.interaction_state.remove(&modal_key);
                ui.modal_active = false;
                ui.modal_window_id = None;
                ui.needs_redraw = true;
                return;
            }
            Err(e) => {
                state.cmdline_dialog.error_msg = Some(e);
                ui.needs_redraw = true;
            }
        }
    }

    // 4. Floating Dialog Card — LazyVim Aesthetic with Cyan/Mint Border
    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.95)
    } else {
        colors.bg_sidebar
    };

    let border_col = Color::rgb(115.0 / 255.0, 218.0 / 255.0, 202.0 / 255.0);
    let current_input = state.cmdline_dialog.input.clone();

    let mut card = ui.container()
        .id(modal_id)
        .pos(center_x, center_y)
        .overlay()
        .width(card_w)
        .bg(card_bg)
        .border((border_col, 1.5))
        .radius_all(4.0)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.50), 0.0, 8.0, 24.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    card.padding(4.0, 10.0, 8.0, 10.0)
        .column()
        .gap(3.0)
        .show(|ui| {
            // Header title embedded in top border layout: ─── Cmdline ───
            ui.container()
                .fill_x()
                .height(20.0)
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    ui.container().width(24.0).height(1.5).bg(border_col).show(|_| {});
                    ui.text("Cmdline")
                        .size(12.0)
                        .font_weight(FontWeight::Bold)
                        .color(border_col)
                        .show();
                    ui.container().fill_x().height(1.5).bg(border_col).show(|_| {});
                });

            // Prompt row: red/pink chevron right '\u{f054}' (file tree arrow) | input text with blinking cursor | 'Esc' badge
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .gap(8.0)
                .padding_left(4.0)
                .show(|ui| {
                    ui.text("\u{f120}")
                        .size(11.0)
                        .font_weight(FontWeight::Bold)
                        .color(Color::rgb(247.0 / 255.0, 118.0 / 255.0, 142.0 / 255.0))
                        .show();

                    let text_w = card_w - 75.0;
                    let input_box = ui.container()
                        .width(text_w)
                        .height(26.0)
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            let (before, after) = current_input.split_at(state.cmdline_dialog.cursor_idx.min(current_input.len()));
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .show(|ui| {
                                    if !before.is_empty() {
                                        ui.text(before)
                                            .size(13.0)
                                            .color(colors.text_primary)
                                            .show();
                                    }

                                    let blink = (std::time::Instant::now().elapsed().as_millis() / 500) % 2 == 0;
                                    if blink {
                                        ui.container()
                                            .width(2.0)
                                            .height(15.0)
                                            .bg(colors.text_primary)
                                            .radius_all(1.0)
                                            .show(|_| {});
                                    }

                                    if !after.is_empty() {
                                        ui.text(after)
                                            .size(13.0)
                                            .color(colors.text_primary)
                                            .show();
                                    }
                                });
                        });

                    if input_box.clicked {
                        state.cmdline_dialog.cursor_idx = state.cmdline_dialog.input.len();
                        ui.needs_redraw = true;
                    }

                    // Minimal Esc badge
                    let esc_btn = ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(2.0)
                        .bg(colors.bg_base.with_alpha(0.40))
                        .hover(Style::bg(colors.highlight))
                        .show(|ui| {
                            ui.text("Esc")
                                .size(9.5)
                                .color(colors.text_muted)
                                .show();
                        });

                    if esc_btn.clicked {
                        state.close_cmdline();
                        ui.interaction_state.remove(&modal_key);
                        ui.modal_active = false;
                        ui.needs_redraw = true;
                    }
                });

            // Inline subtle error line if command errored
            if let Some(ref err) = state.cmdline_dialog.error_msg {
                ui.container()
                    .padding_left(20.0)
                    .show(|ui| {
                        ui.text(err)
                            .size(10.5)
                            .color(Color::rgb(1.0, 0.45, 0.45))
                            .show();
                    });
            }
        });

    // 5. Floating Autocomplete Suggestions Dropdown Menu
    if !state.cmdline_dialog.suggestions.is_empty() {
        let dropdown_id = Id::from_u64(999998205);
        let mut dropdown_container = ui.container()
            .id(dropdown_id)
            .pos(dropdown_x, dropdown_y)
            .overlay()
            .width(dropdown_w)
            .height(dropdown_h)
            .bg(card_bg)
            .border((Color::rgba(1.0, 1.0, 1.0, 0.16), 1.0))
            .radius_all(3.0)
            .shadow(Color::rgba(0.0, 0.0, 0.0, 0.50), 0.0, 8.0, 20.0);

        if state.glassmorphism {
            dropdown_container = dropdown_container.backdrop_blur(24.0);
        }

        let mut clicked_folder: Option<String> = None;
        let cur_sel = state.cmdline_dialog.selected_suggestion;

        dropdown_container
            .padding(4.0, 4.0, 4.0, 4.0)
            .scroll_y(true)
            .column()
            .gap(1.0)
            .show(|ui| {
                for (idx, item) in state.cmdline_dialog.suggestions.iter().enumerate() {
                    let is_sel = cur_sel == Some(idx);
                    let row_bg = if is_sel {
                        colors.highlight
                    } else {
                        Color::TRANSPARENT
                    };

                    let row = ui.container()
                        .fill_x()
                        .height(item_h)
                        .bg(row_bg)
                        .hover(Style::bg(colors.highlight))
                        .radius_all(2.0)
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .row()
                        .valign(Align::Center)
                        .gap(8.0)
                        .show(|ui| {
                            // Tag icon \u{f02b} in vibrant teal/cyan matching screenshot
                            ui.text("\u{f02b}")
                                .size(11.5)
                                .color(Color::rgb(45.0 / 255.0, 212.0 / 255.0, 191.0 / 255.0))
                                .show();

                            ui.text(item)
                                .size(12.5)
                                .color(colors.text_primary)
                                .show();
                        });

                    if row.clicked {
                        clicked_folder = Some(item.clone());
                    }
                }
            });

        if let Some(folder) = clicked_folder {
            state.complete_cmdline_suggestion(&folder);
            ui.needs_redraw = true;
        }
    }
}

/// Floating LazyVim-style confirmation dialog for deleting files/folders (dressing.nvim / vim.ui.select style)
pub fn draw_delete_confirm_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.delete_confirm_dialog.open {
        return;
    }

    let target_path = match state.delete_confirm_dialog.target_path.clone() {
        Some(p) => p,
        None => return,
    };

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let viewport = ui.current_viewport;

    let card_w = 440.0f32.min(viewport.size.width - 32.0);
    let center_x = viewport.origin.x + (viewport.size.width - card_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height * 0.18).max(40.0);

    let modal_id = Id::from_u64(999998201);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_active = true;
    ui.modal_window_id = Some(modal_id);
    ui.current_window_id = Some(modal_id);

    // 1. Semi-transparent backdrop overlay
    ui.draws.push(DrawCommand::OverlayRect(OverlayRectDraw {
        x: viewport.origin.x,
        y: viewport.origin.y,
        width: viewport.size.width,
        height: viewport.size.height,
        color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 },
        clip: [0.0, 0.0, 99999.0, 99999.0],
    }));

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum DeleteChoice {
        No,
        Yes,
    }

    struct DeleteOptionItem {
        choice: DeleteChoice,
        num_str: &'static str,
        label: &'static str,
    }

    let all_options = [
        DeleteOptionItem { choice: DeleteChoice::No, num_str: "1. ", label: "No" },
        DeleteOptionItem { choice: DeleteChoice::Yes, num_str: "2. ", label: "Yes" },
    ];

    let query = state.delete_confirm_dialog.filter_input.trim().to_lowercase();
    let filtered_options: Vec<&DeleteOptionItem> = all_options
        .iter()
        .filter(|opt| {
            if query.is_empty() {
                true
            } else if query == "1" {
                opt.choice == DeleteChoice::No
            } else if query == "2" {
                opt.choice == DeleteChoice::Yes
            } else if "no".starts_with(&query) || "1. no".starts_with(&query) {
                opt.choice == DeleteChoice::No
            } else if "yes".starts_with(&query) || "2. yes".starts_with(&query) {
                opt.choice == DeleteChoice::Yes
            } else {
                opt.label.to_lowercase().contains(&query)
            }
        })
        .collect();

    // Ensure selected option stays within valid range of filtered list
    if state.delete_confirm_dialog.selected_option >= filtered_options.len() {
        state.delete_confirm_dialog.selected_option = 0;
    }

    let mut do_delete = false;
    let mut do_dismiss = false;

    // 2. Process keyboard input events (typing filters options; Enter confirms; Esc cancels)
    let mut events = std::mem::take(&mut ui.input_events);
    events.retain(|event| {
        match event {
            zenthra_platform::event::PlatformEvent::KeyDown { key } => {
                match key {
                    winit::keyboard::KeyCode::Escape => {
                        do_dismiss = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyC if state.ctrl_pressed => {
                        do_dismiss = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowDown | winit::keyboard::KeyCode::KeyJ if state.ctrl_pressed || *key == winit::keyboard::KeyCode::ArrowDown => {
                        if !filtered_options.is_empty() {
                            state.delete_confirm_dialog.selected_option =
                                (state.delete_confirm_dialog.selected_option + 1) % filtered_options.len();
                            ui.needs_redraw = true;
                            return false;
                        }
                    }
                    winit::keyboard::KeyCode::ArrowUp | winit::keyboard::KeyCode::KeyK if state.ctrl_pressed || *key == winit::keyboard::KeyCode::ArrowUp => {
                        if !filtered_options.is_empty() {
                            state.delete_confirm_dialog.selected_option =
                                (state.delete_confirm_dialog.selected_option + filtered_options.len() - 1) % filtered_options.len();
                            ui.needs_redraw = true;
                            return false;
                        }
                    }
                    winit::keyboard::KeyCode::Tab => {
                        if !filtered_options.is_empty() {
                            state.delete_confirm_dialog.selected_option =
                                (state.delete_confirm_dialog.selected_option + 1) % filtered_options.len();
                            ui.needs_redraw = true;
                            return false;
                        }
                    }
                    winit::keyboard::KeyCode::Enter | winit::keyboard::KeyCode::NumpadEnter => {
                        if let Some(opt) = filtered_options.get(state.delete_confirm_dialog.selected_option) {
                            match opt.choice {
                                DeleteChoice::Yes => do_delete = true,
                                DeleteChoice::No => do_dismiss = true,
                            }
                        }
                        return false;
                    }
                    winit::keyboard::KeyCode::Backspace => {
                        if state.delete_confirm_dialog.cursor_idx > 0 && !state.delete_confirm_dialog.filter_input.is_empty() {
                            state.delete_confirm_dialog.filter_input.remove(state.delete_confirm_dialog.cursor_idx - 1);
                            state.delete_confirm_dialog.cursor_idx -= 1;
                            state.delete_confirm_dialog.selected_option = 0;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::Delete => {
                        if state.delete_confirm_dialog.cursor_idx < state.delete_confirm_dialog.filter_input.len() {
                            state.delete_confirm_dialog.filter_input.remove(state.delete_confirm_dialog.cursor_idx);
                            state.delete_confirm_dialog.selected_option = 0;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowLeft => {
                        if state.delete_confirm_dialog.cursor_idx > 0 {
                            state.delete_confirm_dialog.cursor_idx -= 1;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::ArrowRight => {
                        if state.delete_confirm_dialog.cursor_idx < state.delete_confirm_dialog.filter_input.len() {
                            state.delete_confirm_dialog.cursor_idx += 1;
                        }
                        ui.needs_redraw = true;
                        return false;
                    }
                    winit::keyboard::KeyCode::KeyU if state.ctrl_pressed => {
                        state.delete_confirm_dialog.filter_input.clear();
                        state.delete_confirm_dialog.cursor_idx = 0;
                        state.delete_confirm_dialog.selected_option = 0;
                        ui.needs_redraw = true;
                        return false;
                    }
                    _ => {}
                }
            }
            zenthra_platform::event::PlatformEvent::CharTyped(c) => {
                if !state.ctrl_pressed && !state.alt_pressed && *c != '\r' && *c != '\n' && *c != '\t' && !c.is_control() {
                    state.delete_confirm_dialog.filter_input.insert(state.delete_confirm_dialog.cursor_idx, *c);
                    state.delete_confirm_dialog.cursor_idx += c.len_utf8();
                    state.delete_confirm_dialog.selected_option = 0;
                    ui.needs_redraw = true;
                    return false;
                }
            }
            _ => {}
        }
        true
    });
    ui.input_events = events;

    // 3. Outside click dismiss (approx card height ~110px)
    if state.delete_confirm_dialog.just_opened {
        state.delete_confirm_dialog.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, card_w, 110.0);
        if ui.clicked && !mouse_in_dialog {
            do_dismiss = true;
            ui.consume_click();
        }
    }

    if do_delete {
        state.delete_tree_path(&target_path);
        state.close_delete_confirm();
        ui.interaction_state.remove(&modal_key);
        ui.modal_active = false;
        ui.modal_window_id = None;
        ui.needs_redraw = true;
        return;
    }

    if do_dismiss {
        state.close_delete_confirm();
        ui.interaction_state.remove(&modal_key);
        ui.modal_active = false;
        ui.modal_window_id = None;
        ui.needs_redraw = true;
        return;
    }

    let rel_path = target_path.strip_prefix(&state.current_dir).unwrap_or(&target_path);
    let rel_display = rel_path.display().to_string();
    let title_text = format!("Delete {}?", rel_display);

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.95)
    } else {
        colors.bg_sidebar
    };

    // Warm yellow/accent border matching LazyVim dressing.nvim
    let border_col = Color::rgb(224.0 / 255.0, 175.0 / 255.0, 104.0 / 255.0);

    let mut card = ui.container()
        .id(modal_id)
        .pos(center_x, center_y)
        .overlay()
        .width(card_w)
        .bg(card_bg)
        .border((border_col, 1.5))
        .radius_all(4.0)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.50), 0.0, 8.0, 24.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    let cur_input = state.delete_confirm_dialog.filter_input.clone();
    let sel_idx = state.delete_confirm_dialog.selected_option;

    card.padding(4.0, 10.0, 8.0, 10.0)
        .column()
        .gap(3.0)
        .show(|ui| {
            // Header title embedded in top border layout: ─── Delete <path>? ───
            ui.container()
                .fill_x()
                .height(20.0)
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    ui.container().width(24.0).height(1.5).bg(border_col).show(|_| {});
                    ui.text(&title_text)
                        .size(12.0)
                        .font_weight(FontWeight::Bold)
                        .color(border_col)
                        .show();
                    ui.container().fill_x().height(1.5).bg(border_col).show(|_| {});
                });

            // Prompt row: red/pink chevron right '\u{f054}' | typed filter with blinking cursor | '1/2' or '2/2' counter
            ui.container()
                .fill_x()
                .height(24.0)
                .row()
                .valign(Align::Center)
                .padding_left(4.0)
                .padding_right(4.0)
                .show(|ui| {
                    ui.text("\u{f054}")
                        .size(10.5)
                        .font_weight(FontWeight::Bold)
                        .color(Color::rgb(247.0 / 255.0, 118.0 / 255.0, 142.0 / 255.0))
                        .show();

                    let (before, after) = cur_input.split_at(state.delete_confirm_dialog.cursor_idx.min(cur_input.len()));
                    ui.container()
                        .padding_left(6.0)
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            if !before.is_empty() {
                                ui.text(before)
                                    .size(13.0)
                                    .color(colors.text_primary)
                                    .show();
                            }

                            let blink = (std::time::Instant::now().elapsed().as_millis() / 500) % 2 == 0;
                            if blink {
                                ui.container()
                                    .width(2.0)
                                    .height(14.0)
                                    .bg(colors.text_primary)
                                    .show(|_| {});
                            }

                            if !after.is_empty() {
                                ui.text(after)
                                    .size(13.0)
                                    .color(colors.text_primary)
                                    .show();
                            }
                        });

                    ui.container().fill_x().show(|_| {});

                    let count_str = format!("{}/2", filtered_options.len());
                    ui.text(&count_str)
                        .size(11.5)
                        .color(colors.text_muted.with_alpha(0.60))
                        .show();
                });

            // Divider line
            ui.container()
                .fill_x()
                .height(1.0)
                .bg(colors.border.with_alpha(0.60))
                .show(|_| {});

            // Dynamically shortened options list based on typing
            for (idx, opt) in filtered_options.iter().enumerate() {
                let is_sel = sel_idx == idx;
                let row = ui.container()
                    .fill_x()
                    .height(22.0)
                    .row()
                    .valign(Align::Center)
                    .padding_left(6.0)
                    .bg(if is_sel { Color::rgba(1.0, 1.0, 1.0, 0.08) } else { Color::TRANSPARENT })
                    .hover(Style::bg(colors.highlight))
                    .radius_all(2.0)
                    .show(|ui| {
                        ui.text(opt.num_str)
                            .size(12.0)
                            .font_weight(FontWeight::Bold)
                            .color(Color::rgb(247.0 / 255.0, 118.0 / 255.0, 142.0 / 255.0))
                            .show();
                        ui.text(opt.label)
                            .size(12.0)
                            .font_weight(if is_sel { FontWeight::Bold } else { FontWeight::Regular })
                            .color(colors.text_primary)
                            .show();
                    });

                if row.clicked {
                    if is_sel {
                        match opt.choice {
                            DeleteChoice::Yes => do_delete = true,
                            DeleteChoice::No => do_dismiss = true,
                        }
                    } else {
                        state.delete_confirm_dialog.selected_option = idx;
                    }
                    ui.needs_redraw = true;
                }
            }
        });

    if do_delete {
        state.delete_tree_path(&target_path);
        state.close_delete_confirm();
        ui.interaction_state.remove(&modal_key);
        ui.modal_active = false;
        ui.modal_window_id = None;
        ui.needs_redraw = true;
    } else if do_dismiss {
        state.close_delete_confirm();
        ui.interaction_state.remove(&modal_key);
        ui.modal_active = false;
        ui.modal_window_id = None;
        ui.needs_redraw = true;
    }
}

/// Floating LazyVim-style modal for renaming files/folders (pressing 'r')
pub fn draw_rename_dialog(ui: &mut Ui, state: &mut ZenthreeState) {
    if !state.rename_modal.open {
        return;
    }

    let target_path = match state.rename_modal.target_path.clone() {
        Some(p) => p,
        None => return,
    };

    let colors = use_context::<ThemeColors>().map(|c| *c).unwrap_or_else(|| state.colors());
    let viewport = ui.current_viewport;

    let card_w = 440.0f32.min(viewport.size.width - 32.0);
    let center_x = viewport.origin.x + (viewport.size.width - card_w) / 2.0;
    let center_y = viewport.origin.y + (viewport.size.height * 0.18).max(40.0);

    let modal_id = Id::from_u64(999998202);
    let modal_key = Id::from_u64((modal_id.raw() << 8) | 5);
    ui.interaction_state.insert(modal_key, 1.0);
    ui.modal_active = true;
    ui.modal_window_id = Some(modal_id);
    ui.current_window_id = Some(modal_id);

    // 1. Semi-transparent backdrop overlay
    ui.draws.push(DrawCommand::OverlayRect(OverlayRectDraw {
        x: viewport.origin.x,
        y: viewport.origin.y,
        width: viewport.size.width,
        height: viewport.size.height,
        color: Color { r: 0.0, g: 0.0, b: 0.0, a: 0.35 },
        clip: [0.0, 0.0, 99999.0, 99999.0],
    }));

    let mut do_dismiss = false;

    // 2. Outside click dismiss (approx height ~75px)
    if state.rename_modal.just_opened {
        state.rename_modal.just_opened = false;
    } else {
        let mouse_in_dialog = ui.mouse_in_rect(center_x, center_y, card_w, 75.0);
        if ui.clicked && !mouse_in_dialog {
            do_dismiss = true;
            ui.consume_click();
        }
    }

    if do_dismiss {
        state.close_rename_modal();
        ui.interaction_state.remove(&modal_key);
        ui.modal_active = false;
        ui.modal_window_id = None;
        ui.needs_redraw = true;
        return;
    }

    let orig_name = target_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
    let title_text = format!("Rename {}", orig_name);

    let card_bg = if state.glassmorphism {
        colors.bg_sidebar.with_alpha(0.95)
    } else {
        colors.bg_sidebar
    };

    let border_col = Color::rgb(224.0 / 255.0, 175.0 / 255.0, 104.0 / 255.0);

    let mut card = ui.container()
        .id(modal_id)
        .pos(center_x, center_y)
        .overlay()
        .width(card_w)
        .bg(card_bg)
        .border((border_col, 1.5))
        .radius_all(4.0)
        .shadow(Color::rgba(0.0, 0.0, 0.0, 0.50), 0.0, 8.0, 24.0);

    if state.glassmorphism {
        card = card.backdrop_blur(24.0);
    }

    let current_input = state.rename_modal.input.clone();

    card.padding(4.0, 10.0, 8.0, 10.0)
        .column()
        .gap(4.0)
        .show(|ui| {
            // Header title embedded in top border layout: ─── Rename <name> ───
            ui.container()
                .fill_x()
                .height(20.0)
                .row()
                .valign(Align::Center)
                .gap(6.0)
                .show(|ui| {
                    ui.container().width(24.0).height(1.5).bg(border_col).show(|_| {});
                    ui.text(&title_text)
                        .size(12.0)
                        .font_weight(FontWeight::Bold)
                        .color(border_col)
                        .show();
                    ui.container().fill_x().height(1.5).bg(border_col).show(|_| {});
                });

            // Input row: red chevron '\u{f054}' | input field | 'Esc' badge
            ui.container()
                .fill_x()
                .row()
                .valign(Align::Center)
                .gap(8.0)
                .padding_left(4.0)
                .show(|ui| {
                    ui.text("\u{f054}")
                        .size(10.5)
                        .font_weight(FontWeight::Bold)
                        .color(Color::rgb(247.0 / 255.0, 118.0 / 255.0, 142.0 / 255.0))
                        .show();

                    let text_w = card_w - 85.0;
                    ui.container()
                        .width(text_w)
                        .height(26.0)
                        .row()
                        .valign(Align::Center)
                        .show(|ui| {
                            let (before, after) = current_input.split_at(state.rename_modal.cursor_idx.min(current_input.len()));
                            ui.container()
                                .row()
                                .valign(Align::Center)
                                .show(|ui| {
                                    if !before.is_empty() {
                                        ui.text(before)
                                            .size(13.0)
                                            .color(colors.text_primary)
                                            .show();
                                    }

                                    let blink = (std::time::Instant::now().elapsed().as_millis() / 500) % 2 == 0;
                                    if blink {
                                        ui.container()
                                            .width(2.0)
                                            .height(15.0)
                                            .bg(colors.accent)
                                            .radius_all(1.0)
                                            .show(|_| {});
                                    }

                                    if !after.is_empty() {
                                        ui.text(after)
                                            .size(13.0)
                                            .color(colors.text_primary)
                                            .show();
                                    }
                                });
                        });

                    let esc_btn = ui.container()
                        .padding(2.0, 6.0, 2.0, 6.0)
                        .radius_all(2.0)
                        .bg(colors.bg_base.with_alpha(0.40))
                        .hover(Style::bg(colors.highlight))
                        .show(|ui| {
                            ui.text("Esc")
                                .size(9.5)
                                .color(colors.text_muted)
                                .show();
                        });

                    if esc_btn.clicked {
                        state.close_rename_modal();
                        ui.interaction_state.remove(&modal_key);
                        ui.modal_active = false;
                        ui.needs_redraw = true;
                    }
                });

            // Inline error if rename failed
            if let Some(ref err) = state.rename_modal.error_msg {
                ui.container()
                    .padding_left(20.0)
                    .show(|ui| {
                        ui.text(err)
                            .size(10.5)
                            .color(Color::rgb(1.0, 0.45, 0.45))
                            .show();
                    });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmdline_cd_execution() {
        let mut state = ZenthreeState::new();
        let temp_dir = std::env::temp_dir();
        let target_str = format!("cd {}", temp_dir.display());

        let res = state.execute_cmdline(&target_str);
        assert!(res.is_ok());
        assert_eq!(state.current_dir.canonicalize().unwrap(), temp_dir.canonicalize().unwrap());
        assert!(!state.cmdline_dialog.open);
    }

    #[test]
    fn test_cmdline_create_file_and_folder() {
        let mut state = ZenthreeState::new();
        let temp_dir = std::env::temp_dir().join("zenthree_test_cmdline");
        let _ = std::fs::create_dir_all(&temp_dir);
        state.current_dir = temp_dir.clone();

        // 1. Create file with `touch`
        let res_file = state.execute_cmdline("touch test_sample.rs");
        assert!(res_file.is_ok());
        assert!(temp_dir.join("test_sample.rs").exists());

        // 2. Create folder with `mkdir`
        let res_folder = state.execute_cmdline("mkdir sub_folder");
        assert!(res_folder.is_ok());
        assert!(temp_dir.join("sub_folder").is_dir());

        // 3. Smart create with trailing slash
        let res_smart_dir = state.execute_cmdline("another_dir/");
        assert!(res_smart_dir.is_ok());
        assert!(temp_dir.join("another_dir").is_dir());

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_cmdline_directory_suggestions_and_completion() {
        let mut state = ZenthreeState::new();
        let temp_dir = std::env::temp_dir().join("zenthree_test_suggestions");
        let _ = std::fs::create_dir_all(temp_dir.join("basics"));
        let _ = std::fs::create_dir_all(temp_dir.join("dsa"));
        let _ = std::fs::create_dir_all(temp_dir.join("dsa").join("array"));
        state.current_dir = temp_dir.clone();

        // 1. "cd " lists subdirectories
        state.cmdline_dialog.input = "cd ".to_string();
        state.update_cmdline_suggestions();
        assert_eq!(state.cmdline_dialog.suggestions, vec!["basics/".to_string(), "dsa/".to_string()]);

        // 2. "cd b" filters to "basics/"
        state.cmdline_dialog.input = "cd b".to_string();
        state.update_cmdline_suggestions();
        assert_eq!(state.cmdline_dialog.suggestions, vec!["basics/".to_string()]);

        // 3. Complete suggestion
        state.complete_cmdline_suggestion("basics/");
        assert_eq!(state.cmdline_dialog.input, "cd basics/");

        // 4. Nested drill-down
        state.cmdline_dialog.input = "cd dsa/".to_string();
        state.update_cmdline_suggestions();
        assert_eq!(state.cmdline_dialog.suggestions, vec!["array/".to_string()]);

        // Cleanup
        let _ = std::fs::remove_dir_all(temp_dir);
    }

    #[test]
    fn test_delete_dialog_filter_options() {
        let mut state = ZenthreeState::new();
        state.open_delete_confirm(PathBuf::from("/tmp/test_file.txt"));
        assert!(state.delete_confirm_dialog.open);
        assert_eq!(state.delete_confirm_dialog.filter_input, "");

        // Typing 'y' filters options to only Yes (2)
        state.delete_confirm_dialog.filter_input = "y".to_string();
        let query = state.delete_confirm_dialog.filter_input.trim().to_lowercase();
        assert!("yes".starts_with(&query));
        assert!(!"no".starts_with(&query));

        // Typing 'n' filters options to only No (1)
        state.delete_confirm_dialog.filter_input = "n".to_string();
        let query = state.delete_confirm_dialog.filter_input.trim().to_lowercase();
        assert!("no".starts_with(&query));
        assert!(!"yes".starts_with(&query));

        state.close_delete_confirm();
        assert!(!state.delete_confirm_dialog.open);
    }
}
