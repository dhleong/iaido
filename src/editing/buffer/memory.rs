use tui::text;

use crate::editing::{Buffer, HasId};

pub struct MemoryBuffer {
    id: usize,
    content: text::Text<'static>,
}

impl MemoryBuffer {
    pub fn new(id: usize) -> MemoryBuffer {
        MemoryBuffer {
            id,
            content: text::Text { lines: Vec::new() },
        }
    }
}

impl HasId for MemoryBuffer {
    fn id(&self) -> usize {
        return self.id;
    }
}

impl Buffer for MemoryBuffer {
    fn lines_count(&self) -> usize {
        self.content.height()
    }

    fn append(&mut self, text: text::Text<'static>) {
        self.content.extend(text::Text::from(text));
    }

    fn get(&self, line_index: usize) -> &text::Spans<'static> {
        &self.content.lines[line_index]
    }
}
