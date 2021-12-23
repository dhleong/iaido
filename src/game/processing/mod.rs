use bitflags::bitflags;

mod alias;
pub mod manager;
mod matcher;

use crate::{editing::text::TextLine, input::maps::KeyResult};

#[derive(Debug, Clone, PartialEq)]
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

/// If the input was not processed, it is returned via Unprocessed
#[derive(Clone, Debug)]
pub enum ProcessedText {
    Unprocessed(TextInput),
    Processed(TextInput, ProcessedTextFlags),
    Removed(ProcessedTextFlags),
}

pub trait TextProcessor {
    fn describe(&self) -> &str;
    fn process(&mut self, input: TextInput) -> KeyResult<ProcessedText>;
}
