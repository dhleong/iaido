pub mod manager;

use crate::input::Key;

use super::{
    motion::MotionRange,
    text::{TextLine, TextLines},
    Buffer, CursorPosition,
};

pub enum UndoAction {
    DeleteRange(MotionRange),
    Insert(CursorPosition, TextLine),
    InsertLines(usize, TextLines),
}

pub struct Change {
    /// Where this Change occurred (for redoing, if undone)
    cursor: CursorPosition,

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

    pub fn undo(&self, buffer: &mut Box<dyn Buffer>) {
        for action in self.undo_actions.iter().rev() {
            match action {
                &UndoAction::DeleteRange(range) => buffer.delete_range(range),
                &UndoAction::Insert(pos, ref text) => buffer.insert(pos, text.clone()),
                &UndoAction::InsertLines(pos, ref lines) => buffer.insert_lines(pos, lines.clone()),
            };
        }
    }
}
