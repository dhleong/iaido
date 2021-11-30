use crate::input::completion::Completer;
use std::rc::Rc;

pub struct GameEngine {
    pub completer: Option<Rc<dyn Completer>>,
}

impl Default for GameEngine {
    fn default() -> Self {
        // TODO Create a completer
        Self { completer: None }
    }
}
