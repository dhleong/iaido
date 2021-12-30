use super::{buffers::Buffers, ids::Ids, layout::conn::ConnLayout, source::BufferSource};
use super::{
    layout::SplitableLayout,
    window::{Window, WindowFlags},
};
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
        let mut output = Box::new(Window::with_focused(
            self.ids.next(),
            output_buffer_id,
            false,
        ));
        output.flags = WindowFlags::PROTECTED;

        let mut input = Box::new(Window::new(self.ids.next(), input_buffer.id()));
        input.flags = WindowFlags::PROTECTED;

        ConnLayout { output, input }
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

    pub fn has_edit_windows(&self) -> bool {
        // TODO ?
        self.layout.len() > 0
    }

    pub fn windows_count(&self) -> usize {
        self.layout.windows_count()
    }

    pub fn windows_for_buffer(&mut self, buffer_id: Id) -> impl Iterator<Item = &mut Box<Window>> {
        self.layout.windows_for_buffer(buffer_id)
    }

    pub fn close_window(&mut self, win_id: Id) {
        self.layout.close_window(win_id);
        if self.current == win_id {
            if let Some(next_focus) = self.layout.first_focus(FocusDirection::Down) {
                self.current = next_focus;
                self.by_id_mut(next_focus).unwrap().set_focused(true);
            }
        }
    }

    pub fn replace_window(&mut self, win_id: Id, layout: Box<dyn Layout>) {
        if self.current == win_id && !layout.contains_window(win_id) {
            self.current_window_mut().set_focused(false);
            if let Some(focus) = layout.current_focus() {
                self.current = focus;
            } else {
                panic!("Replacing focused window without any new focus");
            }
        }
        self.layout.replace_window(win_id, layout)
    }

    pub fn hsplit(&mut self) -> Id {
        self.split_with(|layout, old_id, new_window| {
            layout.hsplit(old_id, new_window);
        })
    }

    /// Like hsplit, but always splits at the top-most level
    pub fn split_bottom(&mut self) -> Id {
        self.split_with(|layout, _, new_window| {
            layout.add_window(new_window);
        })
    }

    /// Like hsplit, but always splits at the top-most level
    pub fn split_top(&mut self) -> Id {
        self.split_with(|layout, _, new_window| {
            layout.insert_window(0, new_window);
        })
    }

    pub fn vsplit(&mut self) -> Id {
        self.split_with(|layout, old_id, new_window| {
            layout.vsplit(old_id, new_window);
        })
    }

    fn split_with(&mut self, perform: impl FnOnce(&mut LinearLayout, Id, Box<Window>)) -> Id {
        let id: Id = self.ids.next();

        let old = self.window_for_split();
        let old_id = old.id;
        let old_focused = old.focused;
        old.set_focused(false);

        let buffer = old.buffer;
        let window = Window::with_focused(id, buffer, old_focused);
        let boxed = Box::new(window);

        if old_focused {
            self.current = id;
        }

        perform(&mut self.layout, old_id, boxed);
        id
    }

    pub fn move_focus(&mut self, direction: FocusDirection) {
        if let Some(next) = self.next_focus_window(direction) {
            self.set_focus(next);
        }
    }

    pub fn next_focus_window(&self, direction: FocusDirection) -> Option<Id> {
        self.layout.next_focus(self.current, direction)
    }

    pub fn set_focus(&mut self, id: Id) {
        let prev = self.current;
        self.current = id;
        self.layout.by_id_mut(prev).unwrap().focused = false;
        self.layout.by_id_mut(id).unwrap().focused = true;
    }

    /// Returns the Id of the focused window, if the buffer was found
    pub fn set_focus_to_buffer(&mut self, buffer_id: Id) -> Option<Id> {
        let win_id =
            if let Some(win_id) = self.windows_for_buffer(buffer_id).next().map(|win| win.id) {
                win_id
            } else {
                return None;
            };
        self.set_focus(win_id);
        Some(win_id)
    }

    fn window_for_split(&mut self) -> &mut Box<Window> {
        self.layout
            .by_id_for_split(self.current)
            .expect("Couldn't find current window")
    }
}

impl Resizable for Tabpage {
    fn resize(&mut self, new_size: Size) {
        self.size = new_size;
        self.layout.resize(new_size);
    }
}
