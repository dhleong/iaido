use super::{buffers::Buffers, Buffer, HasId, Id, Resizable, Size};

pub struct Window {
    pub id: Id,
    pub buffer: Id,
    pub size: Size,
    pub scrolled_lines: u32,
    pub scroll_offset: u16,
}

impl Window {
    pub fn new(id: Id, buffer_id: Id) -> Self {
        Self {
            id,
            buffer: buffer_id,
            size: Size { w: 0, h: 0 },
            scrolled_lines: 0,
            scroll_offset: 0,
        }
    }

    pub fn current_buffer<'a>(&self, buffers: &'a Buffers) -> &'a Box<dyn Buffer> {
        buffers.by_id(self.buffer).unwrap()
    }

    pub fn current_buffer_mut<'a>(&self, buffers: &'a mut Buffers) -> &'a mut Box<dyn Buffer> {
        buffers.by_id_mut(self.buffer).unwrap()
    }

    pub fn set_scroll(&mut self, lines: u32, offset: u16) {
        self.scrolled_lines = lines;
        self.scroll_offset = offset;
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
