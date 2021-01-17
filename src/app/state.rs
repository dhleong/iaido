use crate::editing::{
    buffers::Buffers, motion::MotionContext, tabpages::Tabpages, window::Window, Buffer, Resizable,
    Size,
};

pub struct AppState {
    pub buffers: Buffers,
    pub tabpages: Tabpages,
}

impl AppState {
    pub fn current_buffer<'a>(&'a self) -> &'a Box<dyn Buffer> {
        self.tabpages
            .current_tab()
            .current_window()
            .current_buffer(&self.buffers)
    }

    pub fn current_buffer_mut<'a>(&'a mut self) -> &'a mut Box<dyn Buffer> {
        self.tabpages
            .current_tab()
            .current_window()
            .current_buffer_mut(&mut self.buffers)
    }
}

impl Default for AppState {
    fn default() -> Self {
        let buffers = Buffers::new();
        let tabpages = Tabpages::new(Size { w: 0, h: 0 });
        let mut app = Self { buffers, tabpages };

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
