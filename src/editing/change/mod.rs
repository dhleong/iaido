pub mod handler;
pub mod manager;

use crate::input::Key;

use super::{buffer::CopiedRange, motion::MotionRange, Buffer, CursorPosition};

#[derive(Debug, Clone, PartialEq)]
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

    /// Returns a Change for redoing this undo; the keys for the new
    /// Change are copied from this one
    pub fn undo(&self, buffer: &mut Box<dyn Buffer>) -> Change {
        let mut undo_actions = Vec::default();

        for action in self.undo_actions.iter().rev() {
            match action {
                &UndoAction::DeleteRange(range) => {
                    undo_actions.push(UndoAction::InsertRange(range.0, buffer.delete_range(range)));
                }
                &UndoAction::InsertRange(pos, ref text) => {
                    undo_actions.push(UndoAction::DeleteRange(text.motion_range(pos)));
                    buffer.insert_range(pos, text.clone());
                }
            };
        }

        Change {
            keys: self.keys.clone(),
            cursor: self.cursor,
            undo_actions,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use indoc::indoc;

    use crate::editing::buffer::memory::tests::TestableBuffer;
    use crate::editing::buffer::undoable::tests::buffer;
    use crate::editing::motion::MotionFlags;

    #[test]
    fn undo_insert_single_line() {
        let mut buf = buffer("");
        buf.append("Take my love".into());
        buf.append("Take my land".into());

        let change = buf.changes().take_last().unwrap();
        let mut undo = change.undo(&mut buf);
        let undo_action = undo.undo_actions.remove(0);
        assert_eq!(
            undo_action,
            UndoAction::InsertRange(
                (1, 0).into(),
                CopiedRange {
                    text: "Take my land".into(),
                    leading_newline: true,
                    trailing_newline: true,
                }
            )
        );

        undo.undo_actions.push(undo_action);
        let mut redo = undo.undo(&mut buf);
        let redo_action = redo.undo_actions.remove(0);
        assert_eq!(
            redo_action,
            UndoAction::DeleteRange(MotionRange(
                (1, 0).into(),
                (1, 12).into(),
                MotionFlags::LINEWISE,
            ))
        );

        buf.assert_visual_match(indoc! {"
            Take my love
            Take my land
        "});
    }
}
