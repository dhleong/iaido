use std::rc::Rc;

use super::{Buffer, HasId, Id, Size};

pub struct Window {
    pub id: Id,
    buffer: Rc<dyn Buffer>,
    pub size: Size,
}

impl Window {
    pub fn new(id: Id, buffer: Rc<dyn Buffer>) -> Self {
        Self {
            id,
            buffer,
            size: Size { w: 0, h: 0 },
        }
    }

    pub fn current_buffer(&self) -> Rc<dyn Buffer> {
        self.buffer.clone()
    }
}

impl HasId for Window {
    fn id(&self) -> Id {
        return self.id;
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[TuiWindow#{}]", self.id);
    }
}
