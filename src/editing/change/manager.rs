use crate::editing::CursorPosition;

use super::{Change, UndoAction};

pub struct ChangeManager {
    change_depth: u16,
    current_change: Option<Change>,
    undo_stack: Vec<Change>,
}

impl Default for ChangeManager {
    fn default() -> Self {
        Self {
            change_depth: 0,
            current_change: None,
            undo_stack: Vec::default(),
        }
    }
}

impl ChangeManager {
    pub fn begin_change(&mut self, cursor: CursorPosition) {
        self.change_depth += 1;
        if self.current_change.is_none() {
            self.current_change = Some(Change::new(cursor));
        }
    }

    pub fn end_change(&mut self) {
        self.change_depth -= 1;
        if self.change_depth == 0 {
            if let Some(change) = self.current_change.take() {
                self.undo_stack.push(change);
            }
        }
    }

    pub fn enqueue_undo(&mut self, action: UndoAction) {
        self.current_change
            .as_mut()
            .expect("No active change")
            .undo_actions
            .push(action);
    }
}
