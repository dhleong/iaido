use crate::editing::{window::Window, Buffer};

/// A BufWin provides convenient mutable access functions on a Window
/// that require access to its associated buffer
pub struct BufWin<'a> {
    pub window: &'a mut Box<Window>,
    buffer: &'a Box<dyn Buffer>,
}

impl<'a> BufWin<'a> {
    pub fn new(window: &'a mut Box<Window>, buffer: &'a Box<dyn Buffer>) -> Self {
        Self { window, buffer }
    }

    pub fn scroll_lines(&mut self, virtual_lines: i32) {
        self.window.scroll_lines(self.buffer, virtual_lines);
    }
}
