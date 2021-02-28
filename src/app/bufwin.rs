use crate::{
    editing::{window::Window, Buffer},
    input::keys::KeysParsable,
};

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

    pub fn begin_keys_change<T: KeysParsable>(&mut self, initial_keys: T) {
        self.buffer.begin_change(self.window.cursor);
        for key in initial_keys.into_keys() {
            self.buffer.push_change_key(key);
        }
    }

    pub fn begin_insert_change<T: KeysParsable>(&mut self, initial_keys: T) {
        self.begin_keys_change(initial_keys);
        self.window.set_inserting(true);
    }

    pub fn redo(&mut self) -> bool {
        if let Some(cursor) = self.buffer.changes().redo() {
            self.window.cursor = self.window.clamp_cursor(self.buffer, cursor);
            true
        } else {
            false
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some(cursor) = self.buffer.changes().undo() {
            self.window.cursor = self.window.clamp_cursor(self.buffer, cursor);
            true
        } else {
            false
        }
    }
}
