use crate::editing::{buffers::Buffers, tabpages::Tabpages, Window, WindowFactory};

pub struct App<'a, F, W>
where
    F: WindowFactory<W>,
    W: Window,
{
    pub windows: &'a F,
    pub buffers: Buffers,
    pub tabpages: Tabpages<'a, W>,
}

impl<'a, F, W> App<'a, F, W>
where
    F: WindowFactory<W>,
    W: Window,
{
    pub fn new(windows: &'a F) -> Self {
        let mut buffers = Buffers::new();
        let tabpages = Tabpages::new();
        let mut app = Self {
            windows: &windows,
            buffers: Buffers::new(),
            tabpages,
        };

        // create the default tabpage
        let default_id = app.tabpages.create(app.windows, &mut buffers);
        app.tabpages.current = default_id;

        app
    }
}
