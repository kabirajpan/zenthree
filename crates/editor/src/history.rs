use crate::buffer::Buffer;
use crate::selection::Selection;
use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditKind {
    Typing,
    Backspace,
    Delete,
    Other,
}

#[derive(Clone)]
pub struct HistoryEntry {
    pub buffer: Buffer,
    pub selections: Vec<Selection>,
    pub kind: EditKind,
    pub timestamp: Instant,
}

#[derive(Clone)]
pub struct History {
    undo_stack: VecDeque<HistoryEntry>,
    redo_stack: VecDeque<HistoryEntry>,
    max_depth: usize,
    in_undo_redo: bool,
}

impl History {
    pub const DEFAULT_MAX_DEPTH: usize = 500;
    const GROUPING_TIMEOUT: Duration = Duration::from_millis(800);

    pub fn new() -> Self {
        Self::with_max_depth(Self::DEFAULT_MAX_DEPTH)
    }

    pub fn with_max_depth(max_depth: usize) -> Self {
        Self {
            undo_stack: VecDeque::new(),
            redo_stack: VecDeque::new(),
            max_depth,
            in_undo_redo: false,
        }
    }

    /// Call this before modifying the buffer.
    /// Passes the state of the buffer and selections prior to the change.
    pub fn record_edit(&mut self, current_buffer: &Buffer, current_selections: &[Selection], kind: EditKind) {
        if self.in_undo_redo {
            return;
        }

        // Clear redo stack on new user edits
        self.redo_stack.clear();

        let now = Instant::now();

        // Check if this edit should be grouped with the previous edit
        if let Some(last) = self.undo_stack.back_mut() {
            if last.kind == kind && kind != EditKind::Other && now.duration_since(last.timestamp) < Self::GROUPING_TIMEOUT {
                // Keep the original snapshot from before the burst began, just update timestamp
                last.timestamp = now;
                return;
            }
        }

        self.undo_stack.push_back(HistoryEntry {
            buffer: current_buffer.clone(),
            selections: current_selections.to_vec(),
            kind,
            timestamp: now,
        });

        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.pop_front();
        }
    }

    /// Break consecutive grouping (e.g., when moving cursor or clicking)
    pub fn break_group(&mut self) {
        if let Some(last) = self.undo_stack.back_mut() {
            last.kind = EditKind::Other;
        }
    }

    /// Undo: restores buffer and selections to previous state
    pub fn undo(&mut self, buffer: &mut Buffer, selections: &mut Vec<Selection>) -> bool {
        if let Some(entry) = self.undo_stack.pop_back() {
            self.in_undo_redo = true;
            // Push current state onto redo stack
            self.redo_stack.push_back(HistoryEntry {
                buffer: buffer.clone(),
                selections: selections.clone(),
                kind: EditKind::Other,
                timestamp: Instant::now(),
            });
            if self.redo_stack.len() > self.max_depth {
                self.redo_stack.pop_front();
            }

            *buffer = entry.buffer;
            *selections = entry.selections;
            self.in_undo_redo = false;
            true
        } else {
            false
        }
    }

    /// Redo: reapplies undone change
    pub fn redo(&mut self, buffer: &mut Buffer, selections: &mut Vec<Selection>) -> bool {
        if let Some(entry) = self.redo_stack.pop_back() {
            self.in_undo_redo = true;
            // Push current state onto undo stack
            self.undo_stack.push_back(HistoryEntry {
                buffer: buffer.clone(),
                selections: selections.clone(),
                kind: EditKind::Other,
                timestamp: Instant::now(),
            });
            if self.undo_stack.len() > self.max_depth {
                self.undo_stack.pop_front();
            }

            *buffer = entry.buffer;
            *selections = entry.selections;
            self.in_undo_redo = false;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}
