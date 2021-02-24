use super::{buffers::Buffers, ids::Ids, layout::conn::ConnLayout, source::BufferSource};
use super::{layout::SplitableLayout, window::Window};
use super::{
    layout::{Layout, LinearLayout},
    FocusDirection,
};
use super::{Id, Resizable, Size};

pub struct Tabpage {
    pub id: Id,
    ids: Ids,
    current: Id,
    size: Size,
    pub layout: LinearLayout,
}

impl Tabpage {
    pub fn new(id: Id, buffers: &mut Buffers, size: Size) -> Self {
        let mut layout = LinearLayout::vertical();

        let mut ids = Ids::new();
        let window_id = ids.next();
        let initial = Window::new(window_id, buffers.create().id());
        layout.add_window(Box::new(initial));
        layout.resize(layout.size());

        Self {
            id,
            ids,
            current: 0,
            size,
            layout,
        }
    }

    pub fn new_connection(&mut self, buffers: &mut Buffers, output_buffer_id: Id) -> ConnLayout {
        let input_buffer = buffers.create_mut();
        input_buffer.set_source(BufferSource::ConnectionInputForBuffer(output_buffer_id));
        ConnLayout {
            output: Box::new(Window::new(self.ids.next(), output_buffer_id)),
            input: Box::new(Window::new(self.ids.next(), input_buffer.id())),
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

    pub fn windows_for_buffer(&mut self, buffer_id: Id) -> impl Iterator<Item = &mut Box<Window>> {
        self.layout.windows_for_buffer(buffer_id)
    }

    pub fn replace_window(&mut self, win_id: Id, layout: Box<dyn Layout>) {
        self.layout.replace_window(win_id, layout)
    }

    pub fn hsplit(&mut self) -> Id {
        let id: Id = self.ids.next();

        let old = self.current_window_mut();
        let old_id = old.id;
        old.set_focused(false);

        let buffer = old.buffer;
        let window = Window::new(id, buffer);
        let boxed = Box::new(window);
        self.layout.hsplit(old_id, boxed);
        self.current = id;

        id
    }

    pub fn vsplit(&mut self) -> Id {
        let id: Id = self.ids.next();

        let old = self.current_window_mut();
        let old_id = old.id;
        old.set_focused(false);

        let buffer = old.buffer;
        let window = Window::new(id, buffer);
        let boxed = Box::new(window);
        self.layout.vsplit(old_id, boxed);
        self.current = id;

        id
    }

    pub fn move_focus(&mut self, direction: FocusDirection) {
        let prev = self.current;
        if let Some(next) = self.layout.next_focus(prev, direction) {
            self.current = next;
            self.layout.by_id_mut(prev).unwrap().focused = false;
            self.layout.by_id_mut(next).unwrap().focused = true;
        }
    }
}

impl Resizable for Tabpage {
    fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.layout.resize(new_size);
    }
}
