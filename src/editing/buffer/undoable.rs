use delegate::delegate;
use editing::text::TextLines;

use crate::{
    editing::{
        self,
        change::{handler::ChangeHandler, manager::ChangeManager, UndoAction},
        motion::{MotionFlags, MotionRange},
        text::TextLine,
        CursorPosition, HasId, Id,
    },
    input::Key,
};

use super::{Buffer, CopiedRange};

pub struct UndoableBuffer {
    base: Box<dyn Buffer>,
    pub changes: ChangeManager,
}

impl UndoableBuffer {
    #[allow(unused)]
    pub fn wrap(base: Box<dyn Buffer>) -> Box<dyn Buffer> {
        if base.can_handle_change() {
            base
        } else {
            Box::new(UndoableBuffer::from(base))
        }
    }
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
            fn get_range(&self, range: MotionRange) -> CopiedRange;
            fn lines_count(&self) -> usize;
            fn clear(&mut self);
        }
    }

    delegate! {
        to self.changes {
            fn begin_change(&mut self, cursor: CursorPosition);
            fn push_change_key(&mut self, key: Key);
            fn end_change(&mut self);
        }
    }

    //
    // Handle change
    //

    fn can_handle_change(&self) -> bool {
        true
    }

    fn changes(&mut self) -> ChangeHandler {
        ChangeHandler::new(&mut self.base, &mut self.changes)
    }

    //
    // Undoable implementations
    //

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
            col: cursor.col + text.width(),
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
        if text.lines.is_empty() {
            // nop
            return;
        }

        let start = CursorPosition {
            line: line_index,
            col: 0,
        };
        let end = CursorPosition {
            line: line_index + text.lines.len() - 1,
            col: text.lines[text.lines.len() - 1].width(),
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
        self.changes.begin_change(cursor);

        self.changes
            .enqueue_undo(UndoAction::DeleteRange(copied.motion_range(cursor)));
        self.base.insert_range(cursor, copied);

        self.changes.end_change();
    }
}

#[cfg(test)]
pub mod tests {
    use crate::editing::buffer::memory::tests::TestableBuffer;
    use crate::editing::buffer::MemoryBuffer;

    use super::*;

    use indoc::indoc;

    pub fn buffer(s: &'static str) -> Box<dyn Buffer> {
        let mut buffer: Box<dyn Buffer> = Box::new(MemoryBuffer::new(0));
        buffer.append(s.into());
        UndoableBuffer::wrap(buffer)
    }

    #[cfg(test)]
    mod delete_range {
        use super::*;

        #[test]
        fn undo_delete_within_line() {
            let mut buffer = buffer(indoc! {"
                Take my love
            "});
            buffer.delete_range(((0, 4), (0, 7)).into());
            buffer.assert_visual_match("Take love");

            buffer.changes().undo();
            buffer.assert_visual_match("Take my love");
        }

        #[test]
        fn undo_partial_plus_full_line_delete() {
            let mut buffer = buffer(indoc! {"
                Take my love
                Take my
            "});
            buffer.delete_range(((0, 4), (1, 7)).into());
            buffer.assert_visual_match("Take");

            buffer.changes().undo();
            buffer.assert_visual_match(indoc! {"
                Take my love
                Take my
            "});
        }

        #[test]
        fn undo_multiline_partial() {
            let mut buffer = buffer(indoc! {"
                Take my love
                Take my land
                Take me where
            "});
            buffer.delete_range(((0, 7), (2, 7)).into());
            buffer.assert_visual_match("Take my where");

            buffer.changes().undo();
            buffer.assert_visual_match(indoc! {"
                Take my love
                Take my land
                Take me where
            "});
        }
    }

    #[cfg(test)]
    mod insert_range {
        use super::*;

        #[test]
        fn undo_single_partial() {
            let mut buffer = buffer(indoc! {"
                Take my love
            "});
            let range = buffer.delete_range(((0, 4), (0, 7)).into());
            buffer.assert_visual_match("Take love");

            buffer.insert_range((0, 4).into(), range);
            buffer.assert_visual_match("Take my love");

            buffer.changes().undo();
            buffer.assert_visual_match("Take love");
        }

        #[test]
        fn undo_partial_plus_full_line_insert() {
            let mut buffer = buffer(indoc! {"
                Take my love
                Take my land
            "});
            let range = buffer.delete_range(((0, 4), (1, 12)).into());
            buffer.assert_visual_match("Take");

            buffer.insert_range((0, 4).into(), range);
            buffer.assert_visual_match(indoc! {"
                Take my love
                Take my land
            "});

            buffer.changes().undo();
            buffer.assert_visual_match("Take");
        }

        #[test]
        fn undo_multiline_partial() {
            let mut buffer = buffer(indoc! {"
                Take my love
                Take my land
                Take me where
            "});
            let range = buffer.delete_range(((0, 7), (2, 7)).into());
            buffer.assert_visual_match("Take my where");

            buffer.insert_range((0, 7).into(), range);
            buffer.assert_visual_match(indoc! {"
                Take my love
                Take my land
                Take me where
            "});

            buffer.changes().undo();
            buffer.assert_visual_match("Take my where");
        }
    }

    #[cfg(test)]
    mod insert {
        use super::*;

        #[test]
        fn undo_simple() {
            let mut buffer = buffer(indoc! {"
                Take love
            "});
            buffer.insert((0, 4).into(), " my".into());
            buffer.assert_visual_match("Take my love");

            buffer.changes().undo();
            buffer.assert_visual_match("Take love");
        }
    }

    #[cfg(test)]
    mod insert_lines {
        use super::*;

        #[test]
        fn undo_insert_before() {
            let mut buffer = buffer(indoc! {"
                Take my land
            "});
            buffer.insert_lines(0, "Take my love".into());
            buffer.assert_visual_match(indoc! {"
                Take my love
                Take my land
            "});

            buffer.changes().undo();
            buffer.assert_visual_match("Take my land");
        }

        #[test]
        fn undo_insert_after() {
            let mut buffer = buffer(indoc! {"
                Take my love
            "});
            buffer.insert_lines(1, "Take my land".into());
            buffer.assert_visual_match(indoc! {"
                Take my love
                Take my land
            "});

            buffer.changes().undo();
            buffer.assert_visual_match("Take my love");
        }
    }

    #[cfg(test)]
    mod append {
        use super::*;

        #[test]
        fn undo_append_to_empty_buffer() {
            let mut buffer = buffer("");
            buffer.append("Take my love".into());
            buffer.assert_visual_match(indoc! {"
                Take my love
            "});

            buffer.changes().undo();
            assert_eq!(buffer.lines_count(), 0);
            buffer.assert_visual_match("");
        }
    }
}
