use crate::editing::{
    buffer::MemoryBuffer,
    buffers::Buffers,
    motion::char::CharMotion,
    motion::{Motion, MotionContext},
    tabpage::Tabpage,
    tabpages::Tabpages,
    text::{TextLine, TextLines},
    window::Window,
    Buffer, Resizable, Size,
};

use super::{bufwin::BufWin, prompt::Prompt};

pub struct AppState {
    pub running: bool,
    pub buffers: Buffers,
    pub tabpages: Tabpages,
    pub echo_buffer: Box<dyn Buffer>,
    pub prompt: Prompt,
}

impl AppState {
    pub fn current_buffer<'a>(&'a self) -> &'a Box<dyn Buffer> {
        if self.prompt.window.focused {
            return &self.prompt.buffer;
        }

        self.current_window().current_buffer(&self.buffers)
    }

    pub fn current_buffer_mut<'a>(&'a mut self) -> &'a mut Box<dyn Buffer> {
        if self.prompt.window.focused {
            return &mut self.prompt.buffer;
        }

        // NOTE: if we just use self.current_window(), rust complains that we've already immutably
        // borrowed self.buffers, so we go the long way:
        self.tabpages
            .current_tab()
            .current_window()
            .current_buffer_mut(&mut self.buffers)
    }

    pub fn current_window<'a>(&'a self) -> &'a Box<Window> {
        if self.prompt.window.focused {
            return &self.prompt.window;
        }
        self.current_tab().current_window()
    }

    pub fn current_window_mut<'a>(&'a mut self) -> &'a mut Box<Window> {
        if self.prompt.window.focused {
            return &mut self.prompt.window;
        }
        self.current_tab_mut().current_window_mut()
    }

    pub fn current_tab<'a>(&'a self) -> &'a Box<Tabpage> {
        self.tabpages.current_tab()
    }

    pub fn current_tab_mut<'a>(&'a mut self) -> &'a mut Box<Tabpage> {
        self.tabpages.current_tab_mut()
    }

    pub fn current_bufwin<'a>(&'a mut self) -> BufWin<'a> {
        BufWin::new(
            if self.prompt.window.focused {
                &mut self.prompt.window
            } else {
                self.tabpages.current_tab_mut().current_window_mut()
            },
            &self.buffers,
        )
    }

    // ======= echo ===========================================

    pub fn clear_echo(&mut self) {
        self.echo_buffer.clear();
    }

    pub fn echo(&mut self, text: TextLines) {
        self.echo_buffer.append(text);
    }

    // ======= keymap conveniences ============================

    pub fn backspace(&mut self) {
        let motion = CharMotion::Backward(1);
        motion.delete_range(self);
        motion.apply_cursor(self);
    }

    pub fn insert_at_cursor(&mut self, text: TextLine) {
        let cursor = self.current_window().cursor;
        let buffer = self.current_buffer_mut();
        buffer.insert(cursor, text);
    }

    pub fn type_at_cursor(&mut self, ch: char) {
        self.insert_at_cursor(String::from(ch).into());
        self.current_window_mut().cursor.col += 1;
    }
}

impl Default for AppState {
    fn default() -> Self {
        let buffers = Buffers::new();
        let tabpages = Tabpages::new(Size { w: 0, h: 0 });
        let mut app = Self {
            running: true,
            buffers,
            tabpages,
            echo_buffer: Box::new(MemoryBuffer::new(0)),
            prompt: Prompt::default(),
        };

        // create the default tabpage
        let default_id = app.tabpages.create(&mut app.buffers);
        app.tabpages.current = default_id;

        app
    }
}

impl Resizable for AppState {
    fn resize(&mut self, new_size: Size) {
        self.tabpages.resize(new_size);
        self.prompt.resize(new_size);
    }
}

impl MotionContext for AppState {
    fn buffer(&self) -> &Box<dyn Buffer> {
        self.current_buffer()
    }

    fn buffer_mut(&mut self) -> &mut Box<dyn Buffer> {
        self.current_buffer_mut()
    }

    fn bufwin(&mut self) -> BufWin {
        self.current_bufwin()
    }

    fn cursor(&self) -> crate::editing::CursorPosition {
        self.window().cursor
    }

    fn window(&self) -> &Box<Window> {
        self.current_window()
    }

    fn window_mut(&mut self) -> &mut Box<Window> {
        self.current_window_mut()
    }
}
