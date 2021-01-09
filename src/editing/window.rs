use super::{buffers::Buffers, Buffer, HasId, Id, Resizable, Size};

pub struct Window {
    pub id: Id,
    pub buffer: Id,
    pub size: Size,
}

impl Window {
    pub fn new(id: Id, buffer_id: Id) -> Self {
        Self {
            id,
            buffer: buffer_id,
            size: Size { w: 0, h: 0 },
        }
    }

    pub fn current_buffer<'a>(&self, buffers: &'a Buffers) -> &'a Box<dyn Buffer> {
        buffers.by_id(self.buffer).unwrap()
    }

    pub fn current_buffer_mut<'a>(&self, buffers: &'a mut Buffers) -> &'a mut Box<dyn Buffer> {
        buffers.by_id_mut(self.buffer).unwrap()
    }
}

impl HasId for Window {
    fn id(&self) -> Id {
        return self.id;
    }
}

impl Resizable for Window {
    fn resize(&mut self, new_size: Size) {
        self.size = new_size
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[TuiWindow#{}]", self.id);
    }
}
