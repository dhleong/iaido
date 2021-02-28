use crate::editing::{window::Window, Buffer};

/// A BufWin provides convenient mutable access functions on a Window
/// that require access to its associated buffer
pub struct BufWin<'a> {
    pub window: &'a mut Box<Window>,
    pub buffer: &'a mut Box<dyn Buffer>,
}

impl<'a> BufWin<'a> {
    pub fn new(window: &'a mut Box<Window>, buffer: &'a mut Box<dyn Buffer>) -> Self {
        Self { window, buffer }
    }

    pub fn scroll_lines(&mut self, virtual_lines: i32) {
        self.window.scroll_lines(self.buffer, virtual_lines);
    }

    pub fn undo(&mut self) {
        if let Some(cursor) = self.buffer.change().undo() {
            self.window.cursor = cursor;
        }
    }
}
