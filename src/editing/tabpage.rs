use std::{cell::Ref, cell::RefCell};

use super::buffers::Buffers;
use super::window::Window;
use super::{Id, Resizable, Size};

pub struct Tabpage {
    pub id: Id,
    current: Id,
    size: Size,
    windows: Vec<RefCell<Window>>,
}

impl Tabpage {
    pub fn new(id: Id, buffers: &mut Buffers, size: Size) -> Self {
        let mut windows: Vec<RefCell<Window>> = Vec::new();

        let initial = Window::new(0, buffers.create());
        windows.push(RefCell::new(initial));

        Self {
            id,
            current: 0,
            size,
            windows,
        }
    }

    pub fn current_window(&self) -> Ref<Window> {
        self.by_id(self.current).unwrap()
    }

    pub fn by_id(&self, id: Id) -> Option<Ref<Window>> {
        self.windows
            .iter()
            .find(|win| win.borrow().id == id)
            .and_then(|win| Some(win.borrow()))
    }

    pub fn split(&mut self) -> Id {
        let id: Id = self.windows.len();
        let window = Window::new(id, self.current_window().current_buffer());
        let boxed = RefCell::new(window);
        self.windows.push(boxed);

        id
    }
}

impl Resizable for Tabpage {
    fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        // TODO window layouts
        for win in &self.windows {
            win.borrow_mut().resize(new_size)
        }
    }
}
