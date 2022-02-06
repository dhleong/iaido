use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

#[derive(Eq, PartialEq, Hash)]
pub enum Flag {
    NoEcho,
}

#[derive(Clone, Default)]
pub struct Flags(Arc<Mutex<HashSet<Flag>>>);

impl Flags {
    pub fn add(&mut self, flag: Flag) {
        self.0.lock().unwrap().insert(flag);
    }

    pub fn remove(&mut self, flag: Flag) {
        self.0.lock().unwrap().remove(&flag);
    }

    pub fn can_echo(&self) -> bool {
        !self.0.lock().unwrap().contains(&Flag::NoEcho)
    }
}
