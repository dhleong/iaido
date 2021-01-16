use crate::editing::{motion::MotionRange, text::TextLine, text::TextLines, Buffer, HasId};

pub struct MemoryBuffer {
    id: usize,
    content: TextLines,
}

impl MemoryBuffer {
    pub fn new(id: usize) -> MemoryBuffer {
        MemoryBuffer {
            id,
            content: TextLines::default(),
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

    fn append(&mut self, text: TextLines) {
        self.content.extend(text);
    }

    fn get(&self, line_index: usize) -> &TextLine {
        &self.content.lines[line_index]
    }

    fn delete_range(&mut self, range: MotionRange) {
        todo!("Delete: {:?}", range);
    }
}
