pub mod handler;
pub mod manager;

use crate::input::Key;

use super::{buffer::CopiedRange, motion::MotionRange, Buffer, CursorPosition};

#[derive(Debug, Clone)]
pub enum UndoAction {
    DeleteRange(MotionRange),
    InsertRange(CursorPosition, CopiedRange),
}

#[derive(Debug, Clone)]
pub struct Change {
    /// Where this Change occurred (for redoing, if undone)
    pub cursor: CursorPosition,

    /// The Keys that triggered this change
    pub keys: Vec<Key>,

    /// Actions to be performed in *reverse order* to undo the change
    undo_actions: Vec<UndoAction>,
}

impl Change {
    pub fn new(cursor: CursorPosition) -> Self {
        Self {
            cursor,
            keys: Vec::default(),
            undo_actions: Vec::default(),
        }
    }

    pub fn undo(&self, buffer: &mut Box<dyn Buffer>) -> CursorPosition {
        let mut cursor = CursorPosition::default();

        for action in self.undo_actions.iter().rev() {
            cursor = match action {
                &UndoAction::DeleteRange(range) => {
                    buffer.delete_range(range);
                    range.0
                }
                &UndoAction::InsertRange(pos, ref text) => {
                    buffer.insert_range(pos, text.clone());
                    pos
                }
            };
        }

        cursor
    }
}
