use delegate::delegate;
use editing::text::TextLines;

use crate::editing::{
    self,
    change::{manager::ChangeManager, UndoAction},
    motion::{MotionFlags, MotionRange},
    text::TextLine,
    CursorPosition, HasId, Id,
};

use super::Buffer;

pub struct UndoableBuffer {
    base: Box<dyn Buffer>,
    changes: ChangeManager,
}

impl HasId for UndoableBuffer {
    delegate! {
        to self.base {
            fn id(&self) -> Id;
        }
    }
}

impl Buffer for UndoableBuffer {
    delegate! {
        to self.base {
            fn source(&self) -> &crate::editing::source::BufferSource;
            fn set_source(&mut self, source: crate::editing::source::BufferSource);
            fn get(&self, line_index: usize) -> &crate::editing::text::TextLine;
            fn lines_count(&self) -> usize;
            fn clear(&mut self);
        }
    }

    fn delete_range(&mut self, range: MotionRange) {
        self.changes.begin_change(range.0);
        // TODO this may be tricky to enqueue...

        self.base.delete_range(range);
        self.changes.end_change();
    }

    fn insert(&mut self, cursor: CursorPosition, text: TextLine) {
        self.changes.begin_change(cursor);
        let end = CursorPosition {
            line: cursor.line,
            col: cursor.col + text.width() as u16,
        };

        self.base.insert(cursor, text);

        self.changes
            .enqueue_undo(UndoAction::DeleteRange(MotionRange(
                cursor,
                end,
                MotionFlags::NONE,
            )));
        self.changes.end_change();
    }

    fn insert_lines(&mut self, line_index: usize, text: TextLines) {
        let start = CursorPosition {
            line: line_index,
            col: 0,
        };
        let end = CursorPosition {
            line: line_index + text.lines.len(),
            col: 0,
        };
        self.changes.begin_change(start);

        self.base.insert_lines(line_index, text);

        self.changes
            .enqueue_undo(UndoAction::DeleteRange(MotionRange(
                start,
                end,
                MotionFlags::LINEWISE,
            )));
        self.changes.end_change();
    }
}
