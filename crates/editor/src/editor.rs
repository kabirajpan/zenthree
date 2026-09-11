use crate::buffer::{Buffer, Point};
use crate::selection::Selection;

pub trait IndentProvider {
    fn calculate_indent(
        &self,
        rope: &crate::rope::Rope,
        row: usize,
        col: usize,
        file_path: Option<&std::path::Path>,
    ) -> String;
}

#[derive(Clone)]
pub struct Editor {
    pub buffer: Buffer,
    pub selections: Vec<Selection>,
    pub history: crate::history::History,
}

impl Editor {
    pub fn new(buffer: Buffer) -> Self {
        Self {
            buffer,
            selections: vec![Selection::cursor(Point::zero())],
            history: crate::history::History::new(),
        }
    }

    pub fn buffer(&self) -> &Buffer {
        &self.buffer
    }

    pub fn buffer_mut(&mut self) -> &mut Buffer {
        &mut self.buffer
    }

    pub fn cursor(&self) -> Point {
        self.selections.last().map(|s| s.end).unwrap_or(Point::zero())
    }

    pub fn selection(&self) -> Selection {
        self.selections.last().copied().unwrap_or_else(|| Selection::cursor(Point::zero()))
    }

    pub fn set_cursor(&mut self, point: Point) {
        self.history.break_group();
        self.selections = vec![Selection::cursor(point)];
        self.normalize_selections();
    }

    pub fn set_selection(&mut self, selection: Selection) {
        self.history.break_group();
        self.selections = vec![selection];
        self.normalize_selections();
    }

    // -------------------------------------------------------------------------
    // Cursor Movements
    // -------------------------------------------------------------------------

    pub fn move_left(&mut self) {
        self.history.break_group();
        for i in 0..self.selections.len() {
            let selection = self.selections[i];
            if !selection.is_empty() {
                let (start, _) = selection.range();
                self.selections[i] = Selection::cursor(start);
            } else {
                let new_end = self.move_point_left(selection.end);
                self.selections[i] = Selection::cursor(new_end);
            }
        }
        self.normalize_selections();
    }

    pub fn move_right(&mut self) {
        self.history.break_group();
        for i in 0..self.selections.len() {
            let selection = self.selections[i];
            if !selection.is_empty() {
                let (_, end) = selection.range();
                self.selections[i] = Selection::cursor(end);
            } else {
                let new_end = self.move_point_right(selection.end);
                self.selections[i] = Selection::cursor(new_end);
            }
        }
        self.normalize_selections();
    }

    pub fn move_up(&mut self) {
        self.history.break_group();
        for i in 0..self.selections.len() {
            let new_end = self.move_point_up(self.selections[i].end);
            self.selections[i] = Selection::cursor(new_end);
        }
        self.normalize_selections();
    }

    pub fn move_down(&mut self) {
        self.history.break_group();
        for i in 0..self.selections.len() {
            let new_end = self.move_point_down(self.selections[i].end);
            self.selections[i] = Selection::cursor(new_end);
        }
        self.normalize_selections();
    }

    pub fn move_to_line_start(&mut self) {
        for i in 0..self.selections.len() {
            self.selections[i] = Selection::cursor(Point::new(self.selections[i].end.row, 0));
        }
        self.normalize_selections();
    }

    pub fn move_to_line_end(&mut self) {
        for i in 0..self.selections.len() {
            let row = self.selections[i].end.row;
            let len = self.buffer.line(row).map(|l| l.chars().count()).unwrap_or(0);
            self.selections[i] = Selection::cursor(Point::new(row, len));
        }
        self.normalize_selections();
    }

    pub fn move_to_top(&mut self) {
        self.set_cursor(Point::zero());
    }

    pub fn move_to_bottom(&mut self) {
        let last_row = self.buffer.line_count().saturating_sub(1);
        let col = self.buffer.line(last_row).map(|l| l.chars().count()).unwrap_or(0);
        self.set_cursor(Point::new(last_row, col));
    }

    // -------------------------------------------------------------------------
    // Selection Extensions
    // -------------------------------------------------------------------------

    pub fn extend_selection_left(&mut self) {
        for i in 0..self.selections.len() {
            let new_end = self.move_point_left(self.selections[i].end);
            self.selections[i].end = new_end;
        }
        self.normalize_selections();
    }

    pub fn extend_selection_right(&mut self) {
        for i in 0..self.selections.len() {
            let new_end = self.move_point_right(self.selections[i].end);
            self.selections[i].end = new_end;
        }
        self.normalize_selections();
    }

    pub fn extend_selection_up(&mut self) {
        for i in 0..self.selections.len() {
            let new_end = self.move_point_up(self.selections[i].end);
            self.selections[i].end = new_end;
        }
        self.normalize_selections();
    }

    pub fn extend_selection_down(&mut self) {
        for i in 0..self.selections.len() {
            let new_end = self.move_point_down(self.selections[i].end);
            self.selections[i].end = new_end;
        }
        self.normalize_selections();
    }

    pub fn extend_selection_to_line_start(&mut self) {
        for i in 0..self.selections.len() {
            self.selections[i].end = Point::new(self.selections[i].end.row, 0);
        }
        self.normalize_selections();
    }

    pub fn extend_selection_to_line_end(&mut self) {
        for i in 0..self.selections.len() {
            let row = self.selections[i].end.row;
            let len = self.buffer.line(row).map(|l| l.chars().count()).unwrap_or(0);
            self.selections[i].end = Point::new(row, len);
        }
        self.normalize_selections();
    }

    pub fn select_all(&mut self) {
        let last_row = self.buffer.line_count().saturating_sub(1);
        let last_col = self.buffer.line(last_row).map(|l| l.chars().count()).unwrap_or(0);
        self.selections = vec![Selection::new(Point::zero(), Point::new(last_row, last_col))];
    }

    // -------------------------------------------------------------------------
    // Undo / Redo & Clipboard
    // -------------------------------------------------------------------------

    pub fn undo(&mut self) -> bool {
        self.history.undo(&mut self.buffer, &mut self.selections)
    }

    pub fn redo(&mut self) -> bool {
        self.history.redo(&mut self.buffer, &mut self.selections)
    }

    pub fn can_undo(&self) -> bool {
        self.history.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.history.can_redo()
    }

    pub fn selected_text(&self) -> String {
        let mut texts = Vec::new();
        for selection in &self.selections {
            let (start, end) = selection.range();
            if start != end {
                let start_offset = self.buffer.point_to_offset(start);
                let end_offset = self.buffer.point_to_offset(end);
                texts.push(self.buffer.slice_bytes(start_offset.value(), end_offset.value()));
            }
        }
        if texts.is_empty() {
            if let Some(selection) = self.selections.last() {
                if let Some(line) = self.buffer.line(selection.end.row) {
                    let mut s = line.to_string();
                    if !s.ends_with('\n') {
                        s.push('\n');
                    }
                    return s;
                }
            }
            String::new()
        } else {
            texts.join("\n")
        }
    }

    pub fn delete_selection(&mut self) {
        let has_selection = self.selections.iter().any(|s| !s.is_empty());
        if !has_selection {
            return;
        }
        self.history.record_edit(&self.buffer, &self.selections, crate::history::EditKind::Other);
        let mut new_selections = Vec::new();
        for selection in &self.selections {
            let (start, end) = selection.range();
            if start != end {
                let start_offset = self.buffer.point_to_offset(start);
                let end_offset = self.buffer.point_to_offset(end);
                self.buffer.delete(start_offset, end_offset);
                new_selections.push(Selection::cursor(start));
            } else {
                new_selections.push(*selection);
            }
        }
        self.selections = new_selections;
        self.normalize_selections();
        self.history.break_group();
    }

    pub fn cut_selection(&mut self) -> String {
        let has_selection = self.selections.iter().any(|s| !s.is_empty());
        if !has_selection {
            if let Some(selection) = self.selections.last().copied() {
                let row = selection.end.row;
                if let Some(line) = self.buffer.line(row) {
                    let mut text = line.to_string();
                    if !text.ends_with('\n') {
                        text.push('\n');
                    }
                    self.history.record_edit(&self.buffer, &self.selections, crate::history::EditKind::Other);
                    let start_point = Point::new(row, 0);
                    let next_row = row + 1;
                    let end_point = if next_row < self.buffer.line_count() {
                        Point::new(next_row, 0)
                    } else {
                        Point::new(row, self.buffer.line(row).map(|l| l.chars().count()).unwrap_or(0))
                    };
                    let start_offset = self.buffer.point_to_offset(start_point);
                    let end_offset = self.buffer.point_to_offset(end_point);
                    self.buffer.delete(start_offset, end_offset);
                    let new_cursor = Point::new(row.min(self.buffer.line_count().saturating_sub(1)), 0);
                    self.set_cursor(new_cursor);
                    self.history.break_group();
                    return text;
                }
            }
            return String::new();
        }

        let text = self.selected_text();
        self.delete_selection();
        text
    }

    pub fn paste(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }
        self.history.record_edit(&self.buffer, &self.selections, crate::history::EditKind::Other);
        self.insert_internal(text, None, None);
        self.history.break_group();
    }

    // -------------------------------------------------------------------------
    // Editing Operations
    // -------------------------------------------------------------------------

    pub fn insert(
        &mut self,
        text: &str,
        file_path: Option<&std::path::Path>,
        indent_provider: Option<&dyn IndentProvider>,
    ) {
        let is_burst = text.chars().count() == 1 && text != " " && text != "\n" && text != "\t";
        let kind = if is_burst {
            crate::history::EditKind::Typing
        } else {
            crate::history::EditKind::Other
        };
        self.history.record_edit(&self.buffer, &self.selections, kind);
        self.insert_internal(text, file_path, indent_provider);
    }

    fn insert_internal(
        &mut self,
        text: &str,
        file_path: Option<&std::path::Path>,
        indent_provider: Option<&dyn IndentProvider>,
    ) {
        let opener = if text.chars().count() == 1 { self.get_closing_pair(text) } else { None };

        let mut new_selections = Vec::new();

        for selection in &self.selections {
            let (start, end) = selection.range();
            let start_offset = self.buffer.point_to_offset(start);
            let end_offset = self.buffer.point_to_offset(end);

            if start != end {
                // CASE 1: Active Selection (Range)
                if let Some(closer) = opener {
                    // Wrap selection with opener and closer
                    self.buffer.insert(start_offset, text);
                    let shifted_end_offset = crate::buffer::Offset(end_offset.value() + text.len());
                    self.buffer.insert(shifted_end_offset, closer);

                    let wrapped_start = self.buffer.offset_to_point(crate::buffer::Offset(start_offset.value() + text.len()));
                    let wrapped_end = self.buffer.offset_to_point(shifted_end_offset);
                    
                    if selection.start <= selection.end {
                        new_selections.push(Selection::new(wrapped_start, wrapped_end));
                    } else {
                        new_selections.push(Selection::new(wrapped_end, wrapped_start));
                    }
                } else {
                    // Overwrite selection
                    self.buffer.delete(start_offset, end_offset);
                    self.buffer.insert(start_offset, text);
                    
                    let new_cursor = self.buffer.offset_to_point(crate::buffer::Offset(start_offset.value() + text.len()));
                    new_selections.push(Selection::cursor(new_cursor));
                }
            } else {
                // CASE 2: Point Cursor
                let mut cursor_override = None;
                let text_to_insert = if text == "\n" {
                    // Smart Newline
                    let indent = if let Some(provider) = indent_provider {
                        provider.calculate_indent(
                            self.buffer.rope(),
                            start.row,
                            start.column,
                            file_path,
                        )
                    } else {
                        String::new()
                    };

                    let next_char = self.buffer
                        .slice_bytes(start_offset.value(), (start_offset.value() + 1).min(self.buffer.rope().len()))
                        .chars().next();
                    let prev_char = if start_offset.value() > 0 {
                        self.buffer.slice_bytes(start_offset.value() - 1, start_offset.value()).chars().next()
                    } else { None };

                    let is_split = (prev_char == Some('{') && next_char == Some('}')) ||
                                   (prev_char == Some('[') && next_char == Some(']')) ||
                                   (prev_char == Some('(') && next_char == Some(')'));

                    if is_split {
                        let parent_indent = if let Some(line) = self.buffer.line(start.row) {
                            line.chars().take_while(|c| c.is_whitespace()).collect::<String>()
                        } else { String::new() };
                        let result = format!("\n{}\n{}", indent, parent_indent);
                        cursor_override = Some(start_offset.value() + 1 + indent.len());
                        result
                    } else {
                        format!("\n{}", indent)
                    }
                } else if text.chars().count() == 1 {
                    // Smart Overwrite & Auto-Closing
                    let next_char = self.buffer
                        .slice_bytes(start_offset.value(), (start_offset.value() + 1).min(self.buffer.rope().len()))
                        .chars().next();
                    
                    let is_closer = text == ")" || text == "]" || text == "}" || text == "\"" || text == "'";
                    
                    if is_closer && next_char == Some(text.chars().next().unwrap()) {
                        cursor_override = Some(start_offset.value() + 1);
                        "".to_string()
                    } else if let Some(closer) = opener {
                        cursor_override = Some(start_offset.value() + 1);
                        format!("{}{}", text, closer)
                    } else {
                        text.to_string()
                    }
                } else {
                    text.to_string()
                };

                self.buffer.insert(start_offset, &text_to_insert);
                
                let final_offset = cursor_override.unwrap_or(start_offset.value() + text_to_insert.len());
                let new_cursor = self.buffer.offset_to_point(crate::buffer::Offset(final_offset));
                new_selections.push(Selection::cursor(new_cursor));
            }
        }

        self.selections = new_selections;
        self.normalize_selections();
    }

    fn get_closing_pair(&self, opener: &str) -> Option<&'static str> {
        match opener {
            "(" => Some(")"),
            "[" => Some("]"),
            "{" => Some("}"),
            "\"" => Some("\""),
            "'" => Some("'"),
            _ => None,
        }
    }

    pub fn backspace(&mut self) {
        self.history.record_edit(&self.buffer, &self.selections, crate::history::EditKind::Backspace);
        for i in 0..self.selections.len() {
            let selection = self.selections[i];
            let (start, end) = selection.range();

            if start != end {
                let start_offset = self.buffer.point_to_offset(start);
                let end_offset = self.buffer.point_to_offset(end);
                self.buffer.delete(start_offset, end_offset);
                self.selections[i] = Selection::cursor(start);
            } else {
                if start.column > 0 {
                    let prev_point = Point::new(start.row, start.column - 1);
                    let prev_offset = self.buffer.point_to_offset(prev_point);
                    let current_offset = self.buffer.point_to_offset(start);
                    self.buffer.delete(prev_offset, current_offset);
                    self.selections[i] = Selection::cursor(prev_point);
                } else if start.row > 0 {
                    let prev_row = start.row - 1;
                    let prev_len = self.buffer.line(prev_row).map(|l| l.chars().count()).unwrap_or(0);
                    let prev_point = Point::new(prev_row, prev_len);
                    let prev_offset = self.buffer.point_to_offset(prev_point);
                    let current_offset = self.buffer.point_to_offset(start);
                    self.buffer.delete(prev_offset, current_offset);
                    self.selections[i] = Selection::cursor(prev_point);
                }
            }
        }
        self.normalize_selections();
    }

    pub fn delete(&mut self) {
        self.history.record_edit(&self.buffer, &self.selections, crate::history::EditKind::Delete);
        for i in 0..self.selections.len() {
            let selection = self.selections[i];
            let (start, end) = selection.range();

            if start != end {
                let start_offset = self.buffer.point_to_offset(start);
                let end_offset = self.buffer.point_to_offset(end);
                self.buffer.delete(start_offset, end_offset);
                self.selections[i] = Selection::cursor(start);
            } else {
                let current_offset = self.buffer.point_to_offset(start);
                let next_point = self.move_point_right(start);
                let next_offset = self.buffer.point_to_offset(next_point);
                if current_offset != next_offset {
                    self.buffer.delete(current_offset, next_offset);
                }
            }
        }
        self.normalize_selections();
    }

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    fn move_point_left(&self, point: Point) -> Point {
        if point.column > 0 {
            Point::new(point.row, point.column - 1)
        } else if point.row > 0 {
            let prev_len = self.buffer.line(point.row - 1).map(|l| l.chars().count()).unwrap_or(0);
            Point::new(point.row - 1, prev_len)
        } else {
            point
        }
    }

    fn move_point_right(&self, point: Point) -> Point {
        if let Some(line) = self.buffer.line(point.row) {
            let char_count = line.chars().count();
            if point.column < char_count {
                Point::new(point.row, point.column + 1)
            } else if point.row + 1 < self.buffer.line_count() {
                Point::new(point.row + 1, 0)
            } else {
                point
            }
        } else {
            point
        }
    }

    fn move_point_up(&self, point: Point) -> Point {
        if point.row > 0 {
            let new_row = point.row - 1;
            let char_count = self.buffer.line(new_row).map(|l| l.chars().count()).unwrap_or(0);
            Point::new(new_row, point.column.min(char_count))
        } else {
            point
        }
    }

    fn move_point_down(&self, point: Point) -> Point {
        if point.row + 1 < self.buffer.line_count() {
            let new_row = point.row + 1;
            let char_count = self.buffer.line(new_row).map(|l| l.chars().count()).unwrap_or(0);
            Point::new(new_row, point.column.min(char_count))
        } else {
            point
        }
    }

    fn normalize_selections(&mut self) {
        if self.selections.is_empty() {
            self.selections.push(Selection::cursor(Point::zero()));
            return;
        }

        let line_count = self.buffer.line_count();
        for selection in &mut self.selections {
            selection.start.row = selection.start.row.min(line_count.saturating_sub(1));
            let start_len = self.buffer.line(selection.start.row).map(|l| l.chars().count()).unwrap_or(0);
            selection.start.column = selection.start.column.min(start_len);

            selection.end.row = selection.end.row.min(line_count.saturating_sub(1));
            let end_len = self.buffer.line(selection.end.row).map(|l| l.chars().count()).unwrap_or(0);
            selection.end.column = selection.end.column.min(end_len);
        }

        self.selections.sort_by_key(|s| s.range().0);

        let mut merged = Vec::new();
        for selection in self.selections.drain(..) {
            if merged.is_empty() {
                merged.push(selection);
            } else {
                let last_idx = merged.len() - 1;
                let last = merged[last_idx];
                let (l_start, l_end) = last.range();
                let (c_start, c_end) = selection.range();

                if c_start <= l_end {
                    let new_end = l_end.max(c_end);
                    if last.start <= last.end {
                        merged[last_idx] = Selection::new(l_start, new_end);
                    } else {
                        merged[last_idx] = Selection::new(new_end, l_start);
                    }
                } else {
                    merged.push(selection);
                }
            }
        }
        self.selections = merged;
    }

    pub fn replace_all(&mut self, new_text: &str) {
        self.history.record_edit(&self.buffer, &self.selections, crate::history::EditKind::Other);
        let old_cursor = self.cursor();
        self.buffer = Buffer::from_text(new_text);
        self.set_cursor(old_cursor);
        self.history.break_group();
    }
}

#[derive(Debug, Clone)]
pub struct EditEvent {
    pub start_byte: usize,
    pub old_end_byte: usize,
    pub new_end_byte: usize,
    pub start_position: Point,
    pub old_end_position: Point,
    pub new_end_position: Point,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selected_text_and_cut() {
        let mut editor = Editor::new(Buffer::from_text("Hello World\nSecond line"));
        editor.set_selection(Selection::new(Point::new(0, 0), Point::new(0, 5)));
        assert_eq!(editor.selected_text(), "Hello");

        let cut = editor.cut_selection();
        assert_eq!(cut, "Hello");
        assert_eq!(editor.buffer.to_string(), " World\nSecond line");

        // Undo cut
        assert!(editor.undo());
        assert_eq!(editor.buffer.to_string(), "Hello World\nSecond line");

        // Redo cut
        assert!(editor.redo());
        assert_eq!(editor.buffer.to_string(), " World\nSecond line");
    }

    #[test]
    fn test_paste_and_undo() {
        let mut editor = Editor::new(Buffer::from_text("foo"));
        editor.set_cursor(Point::new(0, 3));
        editor.paste(" bar");
        assert_eq!(editor.buffer.to_string(), "foo bar");

        assert!(editor.undo());
        assert_eq!(editor.buffer.to_string(), "foo");

        assert!(editor.redo());
        assert_eq!(editor.buffer.to_string(), "foo bar");
    }

    #[test]
    fn test_typing_and_undo() {
        let mut editor = Editor::new(Buffer::new());
        editor.insert("a", None, None);
        editor.insert("b", None, None);
        editor.insert("c", None, None);
        assert_eq!(editor.buffer.to_string(), "abc");

        // Undo burst typing
        assert!(editor.undo());
        assert_eq!(editor.buffer.to_string(), "");

        // Redo burst typing
        assert!(editor.redo());
        assert_eq!(editor.buffer.to_string(), "abc");
    }
}

