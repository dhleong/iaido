use std::fmt;

use super::{buffer::MemoryBuffer, ids::Ids, Buffer, Id};

/// Manages all buffers (Hidden or not) in an app
pub struct Buffers {
    ids: Ids,
    all: Vec<Box<dyn Buffer>>,
}

impl Buffers {
    pub fn new() -> Buffers {
        return Buffers {
            ids: Ids::new(),
            all: Vec::new(),
        };
    }

    pub fn by_id(&self, id: Id) -> Option<&Box<dyn Buffer>> {
        self.all.iter().find(|buf| buf.id() == id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<dyn Buffer>> {
        self.all.iter_mut().find(|buf| buf.id() == id)
    }

    pub fn create(&mut self) -> &Box<dyn Buffer> {
        let id = self.ids.next();
        let buffer = MemoryBuffer::new(id);
        let boxed = Box::new(buffer);

        self.all.push(boxed);

        self.all.last().unwrap()
    }
}

impl fmt::Display for Buffers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffers: count={}]", self.all.len())
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub trait TestableBuffers {
        fn with_buffer(buffer: Box<dyn Buffer>) -> Buffers;
    }

    impl TestableBuffers for Buffers {
        fn with_buffer(buffer: Box<dyn Buffer>) -> Buffers {
            let mut b = Buffers::new();
            b.all.push(buffer);
            b
        }
    }
}
