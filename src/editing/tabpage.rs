use std::rc::Rc;

use super::buffers::Buffers;
use super::{Id, Window, WindowFactory};

pub struct Tabpage<T: Window> {
    pub id: Id,
    current: Id,
    factory: Rc<dyn WindowFactory<T>>,
    windows: Vec<Rc<T>>,
}

impl<T: Window> Tabpage<T> {
    pub fn new(id: Id, factory: Rc<dyn WindowFactory<T>>, buffers: &mut Buffers) -> Self {
        let mut windows: Vec<Rc<T>> = Vec::new();

        let initial = factory.create(0, buffers.create());
        windows.push(Rc::new(initial));

        Self {
            id,
            current: 0,
            factory,
            windows,
        }
    }

    pub fn current_window(&self) -> Rc<T> {
        self.windows[self.current].clone()
    }

    pub fn split(&mut self) -> Rc<T> {
        let id: Id = self.windows.len();
        let window = self
            .factory
            .create(id, self.current_window().current_buffer());

        let boxed = Rc::new(window);
        self.windows.push(boxed.clone());

        boxed
    }
}
