pub mod memory;
pub use memory::MemoryBuffer;

use std::fmt;

use super::{
    motion::MotionRange,
    text::{TextLine, TextLines},
    CursorPosition, HasId,
};

pub trait Buffer: HasId + Send {
    fn lines_count(&self) -> usize;
    fn append(&mut self, text: TextLines);
    fn get(&self, line_index: usize) -> &TextLine;

    fn delete_range(&mut self, range: MotionRange);

    // convenience:
    fn checked_get(&self, line_index: usize) -> Option<&TextLine> {
        if !self.is_empty() && line_index < self.lines_count() {
            Some(self.get(line_index))
        } else {
            None
        }
    }

    fn is_empty(&self) -> bool {
        self.lines_count() == 0
    }

    fn get_char(&self, pos: CursorPosition) -> Option<&str> {
        let line = self.get(pos.line);
        let col_offset = pos.col as usize;

        let mut current_span = 0;
        loop {
            if current_span >= line.0.len() {
                // no more spans in this line
                break;
            }

            let span = &line.0[current_span];
            if span.width() > col_offset {
                return Some(&span.content[col_offset..col_offset + 1]);
            }

            current_span += 1;
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
}

impl fmt::Display for dyn Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffer#{}]", self.id())
    }
}
