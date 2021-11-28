use std::fmt;

use super::{
    buffer::{MemoryBuffer, UndoableBuffer},
    ids::{Ids, BUFFER_ID_LOG, FIRST_USER_BUFFER_ID},
    source::BufferSource,
    Buffer, Id,
};

/// Manages all buffers (Hidden or not) in an app
pub struct Buffers {
    ids: Ids,
    all: Vec<Box<dyn Buffer>>,
}

impl Buffers {
    pub fn new() -> Buffers {
        let mut base = Buffers {
            ids: Ids::with_first(FIRST_USER_BUFFER_ID),
            all: Vec::new(),
        };
        base.create_with_id(BUFFER_ID_LOG);
        base.by_id_mut(BUFFER_ID_LOG)
            .unwrap()
            .set_source(BufferSource::Log);
        base
    }

    pub fn by_id(&self, id: Id) -> Option<&Box<dyn Buffer>> {
        self.all.iter().find(|buf| buf.id() == id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<dyn Buffer>> {
        self.all.iter_mut().find(|buf| buf.id() == id)
    }

    pub fn create(&mut self) -> &Box<dyn Buffer> {
        self.create_for_id();
        self.all.last().unwrap()
    }

    pub fn create_mut(&mut self) -> &mut Box<dyn Buffer> {
        self.create_for_id();
        self.all.last_mut().unwrap()
    }

    fn create_for_id(&mut self) -> Id {
        let id = self.ids.next();
        self.create_with_id(id);

        id
    }

    fn create_with_id(&mut self, id: Id) {
        let buffer = MemoryBuffer::new(id);
        let boxed = UndoableBuffer::wrap(Box::new(buffer));

        self.all.push(boxed);
    }

    pub fn most_recent_id(&self) -> Option<Id> {
        self.ids.most_recent()
    }

    pub fn remove(&mut self, id: Id) -> Option<Box<dyn Buffer>> {
        if let Some(index) = self.all.iter_mut().position(|buf| buf.id() == id) {
            Some(self.all.remove(index))
        } else {
            None
        }
    }

    #[cfg(test)]
    pub fn replace(&mut self, buffer: Box<dyn Buffer>) -> Box<dyn Buffer> {
        let id = buffer.id();
        let index = self.all.iter().position(|b| b.id() == id).unwrap();
        let old = self.all.swap_remove(index);
        self.all.push(UndoableBuffer::wrap(buffer));
        old
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
