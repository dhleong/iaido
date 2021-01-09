use std::{fmt, rc::Rc};

use super::{buffer::memory::MemoryBuffer, ids::Ids, Buffer, Id};

/// Manages all buffers (Hidden or not) in an app
pub struct Buffers {
    ids: Ids,
    all: Vec<Rc<dyn Buffer>>,
}

impl Buffers {
    pub fn new() -> Buffers {
        return Buffers {
            ids: Ids::new(),
            all: Vec::new(),
        };
    }

    pub fn by_id(&self, id: Id) -> Option<&Rc<dyn Buffer>> {
        self.all.iter().find(|buf| buf.id() == id)
    }

    pub fn create(&mut self) -> Rc<dyn Buffer> {
        let id = self.ids.next();
        let buffer = MemoryBuffer::new(id);
        let boxed = Rc::new(buffer);

        self.all.push(boxed.clone());

        return boxed;
    }
}

impl fmt::Display for Buffers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffers: count={}]", self.all.len())
    }
}
