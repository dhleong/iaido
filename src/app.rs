use crate::editing::{buffers::Buffers, tabpages::Tabpages, Buffer, Resizable, Size};

pub struct App {
    pub buffers: Buffers,
    pub tabpages: Tabpages,
}

impl App {
    pub fn new() -> Self {
        let buffers = Buffers::new();
        let tabpages = Tabpages::new(Size { w: 0, h: 0 });
        let mut app = Self { buffers, tabpages };

        // create the default tabpage
        let default_id = app.tabpages.create(&mut app.buffers);
        app.tabpages.current = default_id;

        app
    }

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

impl Resizable for App {
    fn resize(&mut self, new_size: Size) {
        self.tabpages.resize(new_size)
    }
}
