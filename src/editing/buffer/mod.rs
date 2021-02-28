pub mod memory;
pub mod undoable;
mod util;

pub use memory::MemoryBuffer;
pub use undoable::UndoableBuffer;

use std::fmt;

use crate::{connection::ReadValue, input::completion::Completion};

use super::{
    change::handler::ChangeHandler,
    motion::MotionRange,
    source::BufferSource,
    text::{EditableLine, TextLine, TextLines},
    CursorPosition, HasId,
};

#[derive(Debug, Clone)]
pub struct CopiedRange {
    pub text: TextLines,

    /// If false, the first line of `text` was a partial line copy;
    /// if true, the whole first line was copied
    pub leading_newline: bool,

    /// If false, the last line of `text` was a partial line copy
    pub trailing_newline: bool,
}

impl Default for CopiedRange {
    fn default() -> Self {
        Self {
            text: TextLines::default(),
            leading_newline: false,
            trailing_newline: false,
        }
    }
}

impl CopiedRange {
    pub fn end_position(&self, start: CursorPosition) -> CursorPosition {
        if self.text.lines.is_empty() {
            return start;
        }

        let mut count = self.text.lines.len().checked_sub(1).unwrap_or(0);
        if self.leading_newline {
            count += 1;
        }

        let last_width = self.text.lines.last().unwrap().width() as u16;
        let end = CursorPosition {
            line: start.line + count,
            col: if count == 0 {
                start.col + last_width
            } else {
                last_width
            },
        };
        return end;
    }

    pub fn is_partial(&self) -> bool {
        !self.leading_newline && !self.trailing_newline
    }
}

pub trait Buffer: HasId + Send + Sync {
    // read access
    fn lines_count(&self) -> usize;
    fn get(&self, line_index: usize) -> &TextLine;

    // source
    fn source(&self) -> &BufferSource;
    fn set_source(&mut self, source: BufferSource);

    // undoable mutations
    fn delete_range(&mut self, range: MotionRange) -> CopiedRange;
    fn insert(&mut self, cursor: CursorPosition, text: TextLine);
    fn insert_lines(&mut self, line_index: usize, text: TextLines);
    fn insert_range(&mut self, cursor: CursorPosition, copied: CopiedRange);

    // this is a mutation, but generally not undoable
    fn clear(&mut self);

    //
    // Optional
    //

    // change handling
    fn can_handle_change(&self) -> bool {
        false
    }
    fn changes(&mut self) -> ChangeHandler {
        panic!("This Buffer implementation cannot handle changes");
    }

    //
    // Convenience methods, using the core interface above:
    //

    fn append(&mut self, text: TextLines) {
        self.insert_lines(self.lines_count(), text);
    }

    fn append_line(&mut self, text: String) {
        self.append_value(ReadValue::Text(text.into()));
        self.append_value(ReadValue::Newline);
    }

    fn append_value(&mut self, value: ReadValue) {
        match value {
            ReadValue::Newline => {
                self.append(TextLines::from(vec!["".into()]));
            }
            ReadValue::Text(text) => {
                let line = self.lines_count().checked_sub(1).unwrap_or(0);
                self.insert(
                    CursorPosition {
                        line,
                        col: self.get_line_width(line).unwrap_or(0) as u16,
                    },
                    text,
                );
            }
        };
    }

    fn checked_get(&self, line_index: usize) -> Option<&TextLine> {
        if !self.is_empty() && line_index < self.lines_count() {
            Some(self.get(line_index))
        } else {
            None
        }
    }

    fn get_contents(&self) -> String {
        let mut s = String::default();
        for i in 0..self.lines_count() {
            if i > 0 {
                s.push_str("\n");
            }
            s.push_str(self.get(i).to_string().as_str());
        }
        return s;
    }

    fn is_empty(&self) -> bool {
        self.lines_count() == 0
    }

    fn is_read_only(&self) -> bool {
        match self.source() {
            BufferSource::Connection(_) => true,
            _ => false,
        }
    }

    fn get_char(&self, pos: CursorPosition) -> Option<&str> {
        let line = self.get(pos.line);
        let mut col_offset = pos.col as usize;

        let mut current_span = 0;
        loop {
            if current_span >= line.0.len() {
                // no more spans in this line
                break;
            }

            let span = &line.0[current_span];
            let w = span.width();
            if w > col_offset {
                return Some(&span.content[col_offset..col_offset + 1]);
            }

            current_span += 1;
            col_offset -= w;
        }

        if col_offset == 0 {
            return Some("\n");
        }

        None
    }

    fn get_line_width(&self, line_index: usize) -> Option<usize> {
        self.checked_get(line_index)
            .and_then(|line| Some(line.width()))
    }

    fn last_index(&self) -> Option<usize> {
        self.lines_count().checked_sub(1)
    }

    fn apply_completion(&mut self, old: &Completion, new: &Completion) {
        self.delete_range(old.replacement_range().into());
        self.insert(new.start(), new.replacement.clone().into());
    }
}

impl fmt::Display for dyn Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffer#{}]", self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod copied_range {
        use super::*;

        #[cfg(test)]
        mod end_position {
            use super::*;

            #[test]
            fn single_line_partial() {
                let range = CopiedRange {
                    text: "my ".into(),
                    leading_newline: false,
                    trailing_newline: false,
                };
                assert_eq!(
                    range.end_position((0, 7).into()),
                    CursorPosition { line: 0, col: 10 }
                );
            }

            #[test]
            fn two_line_partial() {
                let range = CopiedRange {
                    text: "my love\nTake".into(),
                    leading_newline: false,
                    trailing_newline: false,
                };
                assert_eq!(
                    range.end_position((0, 7).into()),
                    CursorPosition { line: 1, col: 4 }
                );
            }

            #[test]
            fn multi_line_partial() {
                let range = CopiedRange {
                    text: "my love\nTake my land\nTake".into(),
                    leading_newline: false,
                    trailing_newline: false,
                };
                assert_eq!(
                    range.end_position((0, 7).into()),
                    CursorPosition { line: 2, col: 4 }
                );
            }

            #[test]
            fn leading_newline() {
                let range = CopiedRange {
                    text: "Take my land".into(),
                    leading_newline: true,
                    trailing_newline: false,
                };
                assert_eq!(
                    range.end_position((0, 7).into()),
                    CursorPosition { line: 1, col: 12 }
                );
            }

            #[test]
            fn trailing_newline() {
                let range = CopiedRange {
                    text: " my land".into(),
                    leading_newline: false,
                    trailing_newline: true,
                };
                assert_eq!(
                    range.end_position((0, 4).into()),
                    CursorPosition { line: 0, col: 12 }
                );
            }

            #[test]
            fn trailing_newline_plus_partial() {
                let range = CopiedRange {
                    text: " my love\nTake".into(),
                    leading_newline: false,
                    trailing_newline: true,
                };
                assert_eq!(
                    range.end_position((0, 4).into()),
                    CursorPosition { line: 1, col: 4 }
                );
            }

            #[test]
            fn trailing_and_leading() {
                let range = CopiedRange {
                    text: "Take my land\nTake...".into(),
                    leading_newline: true,
                    trailing_newline: true,
                };
                assert_eq!(
                    range.end_position((0, 4).into()),
                    CursorPosition { line: 2, col: 7 }
                );
            }
        }
    }
}
