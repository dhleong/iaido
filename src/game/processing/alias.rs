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

impl Alias {
    #[allow(dead_code)] // TODO remove when able
    pub fn compile_simple(input: String, replacement: String) -> KeyResult<Alias> {
        Ok(Alias {
            matcher: Matcher::compile(input)?,
            processor: SubstitutionProcessor { replacement }.into_processor(),
            one_shot: false,
        })
    }
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

struct SubstitutionProcessor {
    replacement: String,
}

impl SubstitutionProcessor {
    pub fn into_processor(self) -> Box<Processor> {
        Box::new(move |m| Some(m.expand(&self.replacement).into()))
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::text::EditableLine;

    use super::*;

    #[test]
    fn simple_single() {
        let alias = Alias::compile_simple("cook $1".to_string(), "Put $1 in a pan".to_string())
            .expect("Alias should compile!");
        let ProcessedText(output, flags) = alias
            .process(TextInput::Line("cook chorizo".into()))
            .expect("Should process without error")
            .expect("Should have handled the input");
        let text = match output.expect("Should have output") {
            TextInput::Line(text) => text.to_string(),
            _ => panic!("Unexpected output value"),
        };
        assert_eq!(text, "Put chorizo in a pan");
        assert_eq!(flags, ProcessedTextFlags::NONE);
    }
}
