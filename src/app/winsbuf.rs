use crate::{
    connection::ReadValue,
    editing::{text::TextLines, window::Window, Buffer},
};

/// A WinsBuf provides convenient mutable access functions to
/// every window that has a view onto a Buffer
pub struct WinsBuf<'a> {
    pub windows: Vec<&'a mut Box<Window>>,
    pub buffer: &'a mut Box<dyn Buffer>,
}

impl<'a> WinsBuf<'a> {
    pub fn new(windows: Vec<&'a mut Box<Window>>, buffer: &'a mut Box<dyn Buffer>) -> Self {
        Self { windows, buffer }
    }

    pub fn append(&mut self, value: TextLines) {
        self.adjusting_cursor(|me| {
            me.buffer.append(value);
        });
    }

    pub fn append_line(&mut self, line: String) {
        self.adjusting_cursor(|me| {
            me.buffer.append_line(line);
        });
    }

    pub fn append_value(&mut self, value: ReadValue) {
        self.adjusting_cursor(|me| {
            me.buffer.append_value(value);
        });
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn first_window(
        &mut self,
        predicate: impl Fn(&Box<Window>) -> bool,
    ) -> Option<&mut &'a mut Box<Window>> {
        self.windows
            .iter_mut()
            .filter(|win| predicate(win))
            .min_by_key(|win| win.id)
    }

    #[inline]
    fn adjusting_cursor(&mut self, action: impl FnOnce(&mut WinsBuf)) {
        let lines_before = self.buffer.lines_count();
        action(self);
        let lines_after = self.buffer.lines_count();

        if lines_before < lines_after {
            for win in &mut self.windows {
                if win.cursor.line == lines_before.checked_sub(1).unwrap_or(0) {
                    win.cursor.line = lines_after - 1;
                }
            }
        }
    }
}
