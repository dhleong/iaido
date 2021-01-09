use std::cell::{Ref, RefCell, RefMut};

use super::{buffers::Buffers, ids::Ids, tabpage::Tabpage, Id, Window, WindowFactory};

/// Manages all buffers (Hidden or not) in an app
pub struct Tabpages<'a, W: Window> {
    pub current: Id,
    ids: Ids,
    all: Vec<RefCell<Tabpage<'a, W>>>,
}

impl<'a, W: Window> Tabpages<'a, W> {
    pub fn new() -> Tabpages<'a, W> {
        return Tabpages {
            current: 0,
            ids: Ids::new(),
            all: Vec::new(),
        };
    }

    pub fn current_tab(&self) -> Ref<Tabpage<W>> {
        self.by_id(self.current).unwrap()
    }

    pub fn current_tab_mut(&mut self) -> RefMut<'a, Tabpage<W>> {
        self.by_id_mut(self.current).unwrap()
    }

    pub fn by_id(&self, id: Id) -> Option<Ref<Tabpage<W>>> {
        self.all
            .iter()
            .find(|tab| tab.borrow().id == id)
            .and_then(|tab| Some(tab.borrow()))
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<RefMut<'a, Tabpage<W>>> {
        self.all
            .iter()
            .find(|tab| tab.borrow().id == id)
            .and_then(|tab| Some(tab.borrow_mut()))
    }

    pub fn create(&mut self, windows: &'a dyn WindowFactory<W>, buffers: &mut Buffers) -> Id {
        let id = self.ids.next();
        let tabpage = Tabpage::new(id, windows, buffers);

        self.all.push(RefCell::new(tabpage));

        id
    }
}
