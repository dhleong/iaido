use bitflags::bitflags;

mod factory;
mod flagged;
mod multiplex;
mod recency;
mod tokens;

use crate::input::completion::Completer;
pub use factory::GameCompletionsFactory;

bitflags! {
    pub struct ProcessFlags: u8 {
        const RECEIVED = 0b01;
        const SENT = 0b10;

        const ANY = 0b1111;
    }
}

pub trait CompletionSource: Completer {
    /// Feed the CompletionSource a line of text, typically received from
    /// the connection, for processing to power suggestions
    fn process(&mut self, text: String, flags: ProcessFlags);
}

impl<T: CompletionSource + ?Sized> CompletionSource for Box<T> {
    fn process(&mut self, text: String, flags: ProcessFlags) {
        (**self).process(text, flags)
    }
}
