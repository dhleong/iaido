mod factory;
mod multiplex;
mod recency;

use crate::input::completion::Completer;
pub use factory::GameCompletionsFactory;

pub trait CompletionSource: Completer {
    /// Feed the CompletionSource a line of text, typically received from
    /// the connection, for processing to power suggestions
    fn process(&mut self, text: String);
}

impl<T: CompletionSource + ?Sized> CompletionSource for Box<T> {
    fn process(&mut self, text: String) {
        (**self).process(text)
    }
}
