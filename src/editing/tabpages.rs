use std::cell::{Ref, RefCell, RefMut};

use super::{buffers::Buffers, ids::Ids, tabpage::Tabpage, Id};

/// Manages all buffers (Hidden or not) in an app
pub struct Tabpages {
    pub current: Id,
    ids: Ids,
    all: Vec<RefCell<Tabpage>>,
}

impl Tabpages {
    pub fn new() -> Tabpages {
        return Tabpages {
            current: 0,
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
        let id = self.ids.next();
        let tabpage = Tabpage::new(id, buffers);

        self.all.push(RefCell::new(tabpage));

        id
    }
}
