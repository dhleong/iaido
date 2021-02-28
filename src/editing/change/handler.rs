use delegate::delegate;

use crate::editing::{Buffer, CursorPosition};

use super::{manager::ChangeManager, Change};

/// Utility struct for handling the performance of changes on a Buffer
pub struct ChangeHandler<'a> {
    buffer: &'a mut Box<dyn Buffer>,
    changes: &'a mut ChangeManager,
}

impl<'a> ChangeHandler<'a> {
    pub fn new(buffer: &'a mut Box<dyn Buffer>, changes: &'a mut ChangeManager) -> Self {
        Self { buffer, changes }
    }

    delegate! {
        to self.changes {
            pub fn clear(&mut self);
            pub fn push(&mut self, change: Change);
            pub fn take_last(&mut self) -> Option<Change>;
        }
    }

    pub fn undo(&mut self) -> Option<CursorPosition> {
        if let Some(change) = self.changes.take_last() {
            let redo = change.undo(self.buffer);
            let result = Some(redo.cursor);
            self.changes.push_redo(redo);
            result
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<CursorPosition> {
        if let Some(change) = self.changes.take_last_redo() {
            let undo = change.undo(self.buffer);
            let result = Some(undo.cursor);
            self.changes.push(undo);
            result
        } else {
            None
        }
    }
}
