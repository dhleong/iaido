use editing::Size;

use crate::editing::{self, window::Window, FocusDirection, Id, Resizable};

use super::Layout;

pub struct WinLayout {
    pub window: Box<Window>,
}

impl WinLayout {
    pub fn new(window: Box<Window>) -> Self {
        Self { window }
    }
}

impl Layout for WinLayout {
    fn by_id(&self, id: Id) -> Option<&Box<Window>> {
        if self.window.id == id {
            Some(&self.window)
        } else {
            None
        }
    }

    fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Window>> {
        if self.window.id == id {
            Some(&mut self.window)
        } else {
            None
        }
    }

    fn contains_window(&self, win_id: Id) -> bool {
        self.window.id == win_id
    }

    fn windows_for_buffer(
        &mut self,
        buffer_id: Id,
    ) -> Box<dyn Iterator<Item = &mut Box<Window>> + '_> {
        if self.window.buffer == buffer_id {
            Box::new(vec![self.window].iter_mut())
        } else {
            Box::new(vec![].iter_mut())
        }
    }

    fn next_focus(&self, current_id: Id, direction: FocusDirection) -> Option<Id> {
        None
    }

    fn first_focus(&self, direction: FocusDirection) -> Option<Id> {
        Some(self.window.id)
    }

    fn size(&self) -> Size {
        self.window.size
    }
}

impl Resizable for WinLayout {
    fn resize(&mut self, new_size: Size) {
        self.window.resize(new_size)
    }
}
