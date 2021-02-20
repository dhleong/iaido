use super::{buffers::Buffers, ids::Ids, tabpage::Tabpage, window::Window, Id, Resizable, Size};

/// Manages all buffers (Hidden or not) in an app
pub struct Tabpages {
    pub current: Id,
    size: Size,
    ids: Ids,
    all: Vec<Box<Tabpage>>,
}

impl Tabpages {
    pub fn new(size: Size) -> Tabpages {
        return Tabpages {
            current: 0,
            size,
            ids: Ids::new(),
            all: Vec::new(),
        };
    }

    pub fn len(&self) -> usize {
        self.all.len()
    }

    pub fn containing_window_mut(&mut self, window_id: usize) -> Option<&mut Box<Tabpage>> {
        for tabpage in &mut self.all {
            if let Some(_) = tabpage.by_id(window_id) {
                return Some(tabpage);
            }
        }

        None
    }

    pub fn current_tab(&self) -> &Box<Tabpage> {
        self.by_id(self.current).unwrap()
    }

    pub fn current_tab_mut(&mut self) -> &mut Box<Tabpage> {
        self.by_id_mut(self.current).unwrap()
    }

    pub fn by_id(&self, id: Id) -> Option<&Box<Tabpage>> {
        self.all.iter().find(|tab| tab.id == id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<Tabpage>> {
        self.all.iter_mut().find(|tab| tab.id == id)
    }

    pub fn windows_for_buffer(&mut self, buffer_id: Id) -> impl Iterator<Item = &mut Box<Window>> {
        self.all
            .iter_mut()
            .flat_map(move |tab| tab.windows_for_buffer(buffer_id))
    }

    pub fn create(&mut self, buffers: &mut Buffers) -> Id {
        let mut page_size = self.size;
        let tabs_count = self.all.len();
        if tabs_count > 0 {
            page_size.h -= 1;

            // resize an existing, single tab
            if tabs_count == 1 {
                if let Some(tab) = self.all.first_mut() {
                    tab.resize(page_size);
                }
            }
        }

        let id = self.ids.next();
        let tabpage = Tabpage::new(id, buffers, page_size);

        self.all.push(Box::new(tabpage));

        id
    }
}

impl Resizable for Tabpages {
    fn resize(&mut self, new_size: Size) {
        let mut actual_size = new_size;
        actual_size.h -= 1; // leave room for status line

        if self.all.len() > 1 {
            actual_size.h -= 1;
        }

        for page in &mut self.all {
            page.resize(actual_size);
        }
    }
}
