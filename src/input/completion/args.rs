use super::Completer;
use crate::input::completion::empty::EmptyCompleter;

pub struct CommandArgsCompleter {
    delegates: Vec<Box<dyn Completer>>,
}

impl CommandArgsCompleter {
    pub fn new() -> Self {
        Self {
            delegates: Vec::new(),
        }
    }

    pub fn push(&mut self, delegate: Box<dyn Completer>) {
        self.delegates.push(delegate);
    }
}

impl Completer for CommandArgsCompleter {
    fn suggest(
        &self,
        app: Box<&dyn super::CompletableContext>,
        context: super::CompletionContext,
    ) -> super::BoxedSuggestions {
        // NOTE: word #0 would be the command
        let index = context.word_index().checked_sub(1).unwrap_or(0);
        if let Some(delegate) = self.delegates.get(index) {
            delegate.suggest(app, context)
        } else {
            EmptyCompleter.suggest(app, context)
        }
    }
}
