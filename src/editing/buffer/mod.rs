pub mod memory;
pub use memory::MemoryBuffer;

use std::fmt;

use super::{
    motion::MotionRange,
    text::{TextLine, TextLines},
    HasId,
};

pub trait Buffer: HasId {
    fn lines_count(&self) -> usize;
    fn append(&mut self, text: TextLines);
    fn get(&self, line_index: usize) -> &TextLine;

    fn delete_range(&mut self, range: MotionRange);

    // convenience:
    fn is_empty(&self) -> bool {
        self.lines_count() == 0
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
