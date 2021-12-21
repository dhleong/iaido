use crate::{editing::text::TextLine, input::maps::KeyResult};

use super::{
    matcher::{Match, Matcher},
    ProcessedText, ProcessedTextFlags, TextInput, TextProcessor,
};

type Processor = dyn Fn(Match) -> Option<TextLine>;

pub struct Alias {
    matcher: Matcher,
    processor: Box<Processor>,
    one_shot: bool,
}

impl TextProcessor for Alias {
    fn describe(&self) -> &str {
        &self.matcher.description
    }

    fn process(&self, input: TextInput) -> KeyResult<Option<ProcessedText>> {
        match input {
            TextInput::Newline => Ok(Some(ProcessedText(Some(input), ProcessedTextFlags::NONE))),
            TextInput::Line(text) => {
                if let Some(found) = self.matcher.find(text) {
                    let flags = if self.one_shot {
                        ProcessedTextFlags::DESTROYED
                    } else {
                        ProcessedTextFlags::NONE
                    };
                    let output = match (self.processor)(found) {
                        None => None,
                        Some(text) => Some(TextInput::Line(text)),
                    };
                    Ok(Some(ProcessedText(output, flags)))
                } else {
                    Ok(None)
                }
            }
        }
    }
}
