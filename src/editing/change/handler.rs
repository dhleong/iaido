use crate::editing::{Buffer, CursorPosition};

use super::manager::ChangeManager;

/// Utility struct for handling the performance of changes on a Buffer
pub struct ChangeHandler<'a> {
    buffer: &'a mut Box<dyn Buffer>,
    changes: &'a mut ChangeManager,
}

impl<'a> ChangeHandler<'a> {
    pub fn new(buffer: &'a mut Box<dyn Buffer>, changes: &'a mut ChangeManager) -> Self {
        Self { buffer, changes }
    }

    pub fn undo(&mut self) -> Option<CursorPosition> {
        if let Some(change) = self.changes.take_last() {
            Some(change.undo(self.buffer))
        } else {
            None
        }
    }
}
