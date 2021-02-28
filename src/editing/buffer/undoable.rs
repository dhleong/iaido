use delegate::delegate;
use editing::text::TextLines;

use crate::editing::{
    self,
    change::{manager::ChangeManager, UndoAction},
    motion::{MotionFlags, MotionRange},
    text::TextLine,
    CursorPosition, HasId, Id,
};

use super::{Buffer, CopiedRange};

pub struct UndoableBuffer {
    base: Box<dyn Buffer>,
    pub changes: ChangeManager,
}

impl From<Box<dyn Buffer>> for UndoableBuffer {
    fn from(base: Box<dyn Buffer>) -> Self {
        Self {
            base,
            changes: ChangeManager::default(),
        }
    }
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

    fn delete_range(&mut self, range: MotionRange) -> CopiedRange {
        self.changes.begin_change(range.0);
        let deleted = self.base.delete_range(range);
        self.changes
            .enqueue_undo(UndoAction::InsertRange(range.0, deleted.clone()));
        self.changes.end_change();

        deleted
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

    fn insert_range(&mut self, cursor: CursorPosition, copied: CopiedRange) {
        let end = CursorPosition {
            line: cursor.line + copied.inserted_lines(),
            col: copied.text.lines.last().unwrap().width() as u16,
        };
        let flags = if copied.is_partial() {
            MotionFlags::NONE
        } else {
            MotionFlags::LINEWISE
        };

        self.changes.begin_change(cursor);

        self.base.insert_range(cursor, copied);

        self.changes
            .enqueue_undo(UndoAction::DeleteRange(MotionRange(cursor, end, flags)));
        self.changes.end_change();
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::buffer::MemoryBuffer;

    use super::*;

    use indoc::indoc;

    fn buffer(s: &'static str) -> UndoableBuffer {
        let mut buffer: Box<dyn Buffer> = Box::new(MemoryBuffer::new(0));
        buffer.append(s.into());
        UndoableBuffer::from(buffer)
    }

    #[cfg(test)]
    mod delete_range {
        use super::*;

        use crate::editing::buffer::memory::tests::{assert_visual_match, TestableBuffer};

        #[test]
        fn undo_delete_within_line() {
            let mut buffer = buffer(indoc! {"
                Take my love
            "});
            buffer.delete_range(((0, 4), (0, 7)).into());
            assert_visual_match(&buffer, "Take love");

            let last_change = buffer.changes.take_last().unwrap();
            let mut boxed: Box<dyn Buffer> = Box::new(buffer);
            last_change.undo(&mut boxed);
            boxed.assert_visual_match("Take my love");
        }

        #[test]
        fn undo_partial_plus_full_line_delete() {
            let mut buffer = buffer(indoc! {"
                Take my love
                Take my
            "});
            buffer.delete_range(((0, 4), (1, 7)).into());
            assert_visual_match(&buffer, "Take");

            let last_change = buffer.changes.take_last().unwrap();
            let mut boxed: Box<dyn Buffer> = Box::new(buffer);
            last_change.undo(&mut boxed);
            boxed.assert_visual_match(indoc! {"
                Take my love
                Take my
            "});
        }
    }
}
