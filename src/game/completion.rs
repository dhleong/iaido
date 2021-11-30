use crate::input::completion::Completer;

pub trait CompletionSource: Completer {
    /// Feed the CompletionSource a line of text, typically received from
    /// the connection, for processing to power suggestions
    fn process(&mut self, text: String);
}
