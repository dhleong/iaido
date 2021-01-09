use std::{cell::Ref, cell::RefCell};

use super::buffers::Buffers;
use super::{Id, Window, WindowFactory};

pub struct Tabpage<'a, W: Window> {
    pub id: Id,
    current: Id,
    factory: &'a dyn WindowFactory<W>,
    windows: Vec<RefCell<W>>,
}

impl<'a, W: Window> Tabpage<'a, W> {
    pub fn new(id: Id, factory: &'a dyn WindowFactory<W>, buffers: &mut Buffers) -> Self {
        let mut windows: Vec<RefCell<W>> = Vec::new();

        let initial = factory.create(0, buffers.create());
        windows.push(RefCell::new(initial));

        Self {
            id,
            current: 0,
            factory,
            windows,
        }
    }

    pub fn current_window(&self) -> Ref<W> {
        self.by_id(self.current).unwrap()
    }

    pub fn by_id(&self, id: Id) -> Option<Ref<W>> {
        self.windows
            .iter()
            .find(|win| win.borrow().id() == id)
            .and_then(|win| Some(win.borrow()))
    }

    pub fn split(&mut self) -> Id {
        let id: Id = self.windows.len();
        let window = self
            .factory
            .create(id, self.current_window().current_buffer());

        let boxed = RefCell::new(window);
        self.windows.push(boxed);

        id
    }
}
