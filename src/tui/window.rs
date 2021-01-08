use std::rc::Rc;

use crate::editing::{Buffer, Cursor, HasId, Id, Size, Window, WindowFactory};

pub struct TuiWindow {
    pub id: Id,
    pub buffer: Rc<dyn Buffer>,
    pub size: Size,
}

impl HasId for TuiWindow {
    fn id(&self) -> Id {
        return self.id;
    }
}

impl Window for TuiWindow {
    fn cursor(&self) -> Cursor {
        return Cursor::None;
    }

    fn current_buffer(&self) -> Rc<dyn Buffer> {
        return self.buffer.clone();
    }

    fn size(&self) -> Size {
        return self.size;
    }
}

impl std::fmt::Display for TuiWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[TuiWindow#{}]", self.id);
    }
}

pub struct TuiWindowFactory {}

impl WindowFactory<TuiWindow> for TuiWindowFactory {
    fn create(&self, id: Id, buffer: Rc<dyn Buffer>) -> TuiWindow {
        TuiWindow {
            id,
            buffer,
            size: Size { w: 0, h: 0 },
        }
    }
}
