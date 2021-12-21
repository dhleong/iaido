use bitflags::bitflags;

mod alias;
mod matcher;

use crate::{editing::text::TextLine, input::maps::KeyResult};

#[derive(Debug, PartialEq)]
pub enum TextInput {
    Newline,
    Line(TextLine),
}

bitflags! {
    pub struct ProcessedTextFlags: u8 {
        /// The TextProcessor should be destroyed; it should no longer
        /// process any input
        const DESTROYED = 0b01;

        const NONE = 0b0;
    }
}

/// If the TextInput is None, the text should be removed
pub struct ProcessedText(Option<TextInput>, ProcessedTextFlags);

pub trait TextProcessor {
    fn describe(&self) -> &str;
    fn process(&self, input: TextInput) -> KeyResult<Option<ProcessedText>>;
}
