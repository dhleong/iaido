use std::rc::Rc;
use std::sync::{Arc, Mutex};

use crate::connection::ReadValue;
use crate::editing::text::EditableLine;
use crate::input::completion::{
    BoxedSuggestions, CompletableContext, Completer, CompletionContext,
};
use crate::input::history::History;
use crate::input::maps::KeyResult;

use super::completion::{CompletionSource, GameCompletionsFactory, ProcessFlags};
use super::processing::alias::Alias;
use super::processing::manager::TextProcessorManager;
use super::processing::{ProcessedText, TextInput, TextProcessor};

pub struct GameEngine {
    pub aliases: TextProcessorManager<Alias>,
    pub completer: Option<Arc<Mutex<dyn CompletionSource + Send>>>,
    pub history: Option<History<String>>,
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

impl Completer for Arc<Mutex<dyn CompletionSource + Send>> {
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
            aliases: TextProcessorManager::new(),
            completer: Some(Arc::new(Mutex::new(GameCompletionsFactory::create()))),
            history: Some(Default::default()),
        }
    }
}

impl GameEngine {
    pub fn process_received(&mut self, value: ReadValue) -> Option<ReadValue> {
        if let ReadValue::Text(text) = &value {
            if let Some(completions) = self.completer.as_mut() {
                let text = text.to_string();
                let mut guard = completions.lock().unwrap();
                guard.process(text, ProcessFlags::RECEIVED);
            }
        }

        Some(value)
    }

    pub fn process_to_send(&mut self, value: String) -> KeyResult<Option<String>> {
        if let Some(completions) = self.completer.as_mut() {
            let text = value.to_string();
            let mut guard = completions.lock().unwrap();
            guard.process(text, ProcessFlags::SENT);
        }

        if let Some(history) = &mut self.history {
            history.insert(value.to_string());
        }

        match self.aliases.process(TextInput::Line(value.into())) {
            Ok(ProcessedText::Removed(_)) => Ok(None),
            Ok(ProcessedText::Processed(TextInput::Line(processed), _)) => {
                Ok(Some(processed.to_string()))
            }
            Ok(ProcessedText::Unprocessed(TextInput::Line(unprocessed))) => {
                Ok(Some(unprocessed.to_string()))
            }
            Ok(unhandled) => panic!("Unexpected result from alias processing: {:?}", unhandled),
            Err(e) => Err(e),
        }
    }

    /// Reset any configured state on this Engine; relevant when re-loading
    /// a script, for example, to clear previously-created state
    pub fn reset(&mut self) {
        self.aliases.clear();
    }
}
