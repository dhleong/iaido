use tui::text;

use crate::editing::{Buffer, HasId};

pub struct MemoryBuffer<'a> {
    id: usize,
    content: text::Text<'a>,
}

impl<'a> MemoryBuffer<'a> {
    pub fn new(id: usize) -> MemoryBuffer<'a> {
        MemoryBuffer {
            id,
            content: text::Text { lines: Vec::new() },
        }
    }
}

impl<'a> HasId for MemoryBuffer<'a> {
    fn id(&self) -> usize {
        return self.id;
    }
}

impl<'a> Buffer for MemoryBuffer<'a> {
    fn lines_count(&self) -> usize {
        self.content.height()
    }

    fn append(&mut self, text: text::Text) {
        let mut lines = &self.content.lines;
    }
}
