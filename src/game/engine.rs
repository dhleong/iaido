use std::rc::Rc;
use std::sync::Mutex;

use crate::connection::ReadValue;
use crate::editing::text::EditableLine;
use crate::input::completion::{
    BoxedSuggestions, CompletableContext, Completer, CompletionContext,
};

use super::completion::{CompletionSource, GameCompletionsFactory};

pub struct GameEngine {
    pub completer: Option<Rc<Mutex<dyn CompletionSource>>>,
}

impl Completer for Rc<Mutex<dyn CompletionSource>> {
    fn suggest(
        &self,
        app: Box<&dyn CompletableContext>,
        context: CompletionContext,
    ) -> BoxedSuggestions {
        let completer = self.lock().unwrap();
        completer.suggest(app, context)
    }
}

impl Default for GameEngine {
    fn default() -> Self {
        Self {
            completer: Some(Rc::new(Mutex::new(GameCompletionsFactory::create()))),
        }
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
