use crate::editing::{Buffer, HasId};

pub struct MemoryBuffer {
    id: usize,
    lines: Vec<String>,
}

impl MemoryBuffer {
    pub fn new(id: usize) -> MemoryBuffer {
        MemoryBuffer {
            id,
            lines: Vec::new(),
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
        return self.lines.len();
    }
}
