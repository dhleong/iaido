pub mod memory;
pub use memory::MemoryBuffer;

use std::fmt;

use crate::input::completion::Completion;

use super::{
    motion::MotionRange,
    text::{EditableLine, TextLine, TextLines},
    CursorPosition, HasId,
};

pub trait Buffer: HasId + Send + Sync {
    fn lines_count(&self) -> usize;
    fn append(&mut self, text: TextLines);
    fn clear(&mut self);
    fn get(&self, line_index: usize) -> &TextLine;

    fn delete_range(&mut self, range: MotionRange);
    fn insert(&mut self, cursor: CursorPosition, text: TextLine);

    // convenience:
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

    fn get_char(&self, pos: CursorPosition) -> Option<&str> {
        let line = self.get(pos.line);
        let col_offset = pos.col as usize;

        let mut seen_width = 0;
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
            seen_width += w;
        }

        if seen_width == col_offset {
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
        self.delete_range(old.replacement_range());
        self.insert(new.start(), new.replacement.clone().into());
    }
}

impl fmt::Display for dyn Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffer#{}]", self.id())
    }
}
