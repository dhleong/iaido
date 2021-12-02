use std::rc::Rc;
use std::sync::Mutex;

use crate::connection::ReadValue;
use crate::editing::text::EditableLine;

use super::completion::CompletionSource;

pub struct GameEngine {
    pub completer: Option<Rc<Mutex<dyn CompletionSource>>>,
}

impl Default for GameEngine {
    fn default() -> Self {
        // TODO Create a completer
        Self { completer: None }
    }
}

impl GameEngine {
    pub fn process_received(&mut self, value: ReadValue) -> Option<ReadValue> {
        if let ReadValue::Text(text) = &value {
            if let Some(completions) = self.completer.as_mut() {
                let text = text.to_string();
                let mut guard = completions.lock().unwrap();
                guard.process(text);
            }
        }

        return Some(value);
    }
}
