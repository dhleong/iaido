use crate::game::completion::{CompletionSource, ProcessFlags};
use crate::input::completion::Completer;

pub trait SimpleCompletionSource: Completer {
    fn process(&mut self, text: String);
}

pub struct FlaggedCompletionSource<T: SimpleCompletionSource> {
    accepted_flags: ProcessFlags,
    delegate: T,
}

impl<T: SimpleCompletionSource> FlaggedCompletionSource<T> {
    pub fn accepting_flags(delegate: T, accepted_flags: ProcessFlags) -> Self {
        Self {
            accepted_flags,
            delegate,
        }
    }
}

impl<T: SimpleCompletionSource> Completer for FlaggedCompletionSource<T> {
    fn suggest(
        &self,
        app: Box<&dyn crate::input::completion::CompletableContext>,
        context: crate::input::completion::CompletionContext,
    ) -> crate::input::completion::BoxedSuggestions {
        self.delegate.suggest(app, context)
    }
}

impl<T: SimpleCompletionSource> CompletionSource for FlaggedCompletionSource<T> {
    fn process(&mut self, text: String, flags: ProcessFlags) {
        if self.accepted_flags.contains(flags) {
            self.delegate.process(text);
        }
    }
}
