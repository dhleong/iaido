use std::rc::Rc;

use super::{buffer::memory::MemoryBuffer, Buffer};

/// Manages all buffers (Hidden or not) in an app
pub struct Buffers {
    all: Vec<Rc<dyn Buffer>>,
}

impl Buffers {
    pub fn new() -> Buffers {
        return Buffers { all: Vec::new() };
    }

    pub fn create(&mut self) -> Rc<dyn Buffer> {
        let id = self.all.len();
        let buffer = MemoryBuffer::new(id);
        let boxed = Rc::new(buffer);

        self.all.push(boxed.clone());

        return boxed;
    }
}
