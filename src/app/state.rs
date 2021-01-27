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

pub struct AppState {
    pub running: bool,
    pub buffers: Buffers,
    pub tabpages: Tabpages,
    pub echo_buffer: Box<dyn Buffer>,
}

impl AppState {
    pub fn current_buffer<'a>(&'a self) -> &'a Box<dyn Buffer> {
        self.current_window().current_buffer(&self.buffers)
    }

    pub fn current_buffer_mut<'a>(&'a mut self) -> &'a mut Box<dyn Buffer> {
        self.tabpages
            .current_tab()
            .current_window()
            .current_buffer_mut(&mut self.buffers)
    }

    pub fn current_window<'a>(&'a self) -> &'a Box<Window> {
        self.current_tab().current_window()
    }

    pub fn current_window_mut<'a>(&'a mut self) -> &'a mut Box<Window> {
        self.current_tab_mut().current_window_mut()
    }

    pub fn current_tab<'a>(&'a self) -> &'a Box<Tabpage> {
        self.tabpages.current_tab()
    }

    pub fn current_tab_mut<'a>(&'a mut self) -> &'a mut Box<Tabpage> {
        self.tabpages.current_tab_mut()
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
        };

        // create the default tabpage
        let default_id = app.tabpages.create(&mut app.buffers);
        app.tabpages.current = default_id;

        app
    }
}

impl Resizable for AppState {
    fn resize(&mut self, new_size: Size) {
        self.tabpages.resize(new_size)
    }
}

impl MotionContext for AppState {
    fn buffer(&self) -> &Box<dyn Buffer> {
        self.current_buffer()
    }

    fn buffer_mut(&mut self) -> &mut Box<dyn Buffer> {
        self.current_buffer_mut()
    }

    fn cursor(&self) -> crate::editing::CursorPosition {
        self.window().cursor
    }

    fn window(&self) -> &Box<Window> {
        self.tabpages.current_tab().current_window()
    }

    fn window_mut(&mut self) -> &mut Box<Window> {
        self.tabpages.current_tab_mut().current_window_mut()
    }
}
