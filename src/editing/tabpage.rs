use super::layout::Layout;
use super::window::Window;
use super::{buffers::Buffers, ids::Ids};
use super::{Id, Resizable, Size};

pub struct Tabpage {
    pub id: Id,
    ids: Ids,
    current: Id,
    size: Size,
    pub layout: Layout,
}

impl Tabpage {
    pub fn new(id: Id, buffers: &mut Buffers, size: Size) -> Self {
        let mut layout = Layout::vertical();

        let mut ids = Ids::new();
        let window_id = ids.next();
        let initial = Window::new(window_id, buffers.create().id());
        layout.split(Box::new(initial));

        Self {
            id,
            ids,
            current: 0,
            size,
            layout,
        }
    }

    pub fn current_window(&self) -> &Box<Window> {
        self.by_id(self.current).unwrap()
    }

    pub fn current_window_mut(&mut self) -> &mut Box<Window> {
        self.by_id_mut(self.current).unwrap()
    }

    pub fn by_id(&self, id: Id) -> Option<&Box<Window>> {
        self.layout.by_id(id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Window>> {
        self.layout.by_id_mut(id)
    }

    pub fn hsplit(&mut self) -> Id {
        let id: Id = self.ids.next();

        let old = self.current_window_mut();
        old.set_focused(false);

        let buffer = old.buffer;
        let window = Window::new(id, buffer);
        let boxed = Box::new(window);
        self.layout.split(boxed);
        self.current = id;

        id
    }
}

impl Resizable for Tabpage {
    fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.layout.resize(new_size);
    }
}
