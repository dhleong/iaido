use std::cell::{Ref, RefCell, RefMut};

use super::{buffers::Buffers, ids::Ids, tabpage::Tabpage, Id, Resizable, Size};

/// Manages all buffers (Hidden or not) in an app
pub struct Tabpages {
    pub current: Id,
    size: Size,
    ids: Ids,
    all: Vec<RefCell<Tabpage>>,
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

    pub fn current_tab(&self) -> Ref<Tabpage> {
        self.by_id(self.current).unwrap()
    }

    pub fn current_tab_mut(&mut self) -> RefMut<Tabpage> {
        self.by_id_mut(self.current).unwrap()
    }

    pub fn by_id(&self, id: Id) -> Option<Ref<Tabpage>> {
        self.all
            .iter()
            .find(|tab| tab.borrow().id == id)
            .and_then(|tab| Some(tab.borrow()))
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<RefMut<Tabpage>> {
        self.all
            .iter()
            .find(|tab| tab.borrow().id == id)
            .and_then(|tab| Some(tab.borrow_mut()))
    }

    pub fn create(&mut self, buffers: &mut Buffers) -> Id {
        let mut page_size = self.size;
        let tabs_count = self.all.len();
        if tabs_count > 0 {
            page_size.h -= 1;

            // resize an existing, single tab
            if tabs_count == 1 {
                if let Some(tab) = self.all.first_mut() {
                    tab.borrow_mut().resize(page_size);
                }
            }
        }

        let id = self.ids.next();
        let tabpage = Tabpage::new(id, buffers, page_size);

        self.all.push(RefCell::new(tabpage));

        id
    }
}

impl Resizable for Tabpages {
    fn resize(&mut self, new_size: Size) {
        let mut actual_size = new_size;
        if self.all.len() > 1 {
            actual_size.h -= 1;
        }
        for page in &self.all {
            page.borrow_mut().resize(actual_size);
        }
    }
}
