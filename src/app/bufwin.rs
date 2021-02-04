use crate::editing::{buffers::Buffers, window::Window};

/// A BufWin provides convenient mutable access functions on a Window
/// that require access to its associated buffer
pub struct BufWin<'a> {
    window: &'a mut Box<Window>,
    buffers: &'a Buffers,
}

impl<'a> BufWin<'a> {
    pub fn new(window: &'a mut Box<Window>, buffers: &'a Buffers) -> Self {
        Self { window, buffers }
    }

    pub fn scroll_lines(&mut self, virtual_lines: i32) {
        self.window.scroll_lines(self.buffers, virtual_lines);
    }
}
