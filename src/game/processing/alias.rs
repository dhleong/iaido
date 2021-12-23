use std::fmt::Display;

use crate::{
    editing::text::{EditableLine, TextLine},
    input::maps::KeyResult,
};

use super::{
    manager::TextProcessorManager,
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
    pub fn compile_text(input: String, replacement: String) -> KeyResult<Alias> {
        Ok(Alias {
            matcher: Matcher::compile(input)?,
            processor: SubstitutionProcessor { replacement }.into_processor(),
            one_shot: false,
        })
    }
}

impl Display for Alias {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Alias:{}]", self.matcher.description)
    }
}

impl TextProcessor for Alias {
    fn describe(&self) -> &str {
        &self.matcher.description
    }

    fn process(&mut self, input: TextInput) -> KeyResult<Option<ProcessedText>> {
        match input {
            TextInput::Newline => Ok(Some(ProcessedText(Some(input), ProcessedTextFlags::NONE))),
            TextInput::Line(input_text) => {
                if let Some(found) = self.matcher.find(input_text) {
                    let flags = if self.one_shot {
                        ProcessedTextFlags::DESTROYED
                    } else {
                        ProcessedTextFlags::NONE
                    };

                    let result = match (self.processor)(found.clone()) {
                        None => None,
                        Some(mut output) => {
                            let with_replacement = found
                                .input
                                .replacing_range(found.start..found.end, &mut output);

                            Some(TextInput::Line(with_replacement))
                        }
                    };

                    Ok(Some(ProcessedText(result, flags)))
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

impl TextProcessorManager<Alias> {
    #[allow(dead_code)] // TODO remove when able
    pub fn insert_text(&mut self, pattern: String, replacement: String) -> KeyResult {
        let alias = Alias::compile_text(pattern.to_string(), replacement)?;
        self.insert(pattern, alias);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::text::EditableLine;

    use super::*;

    fn process<T: TextProcessor>(
        mut processor: T,
        input: &'static str,
    ) -> (String, ProcessedTextFlags) {
        let ProcessedText(output, flags) = processor
            .process(TextInput::Line(input.into()))
            .expect("Should process without error")
            .expect("Should have handled the input");
        let text = match output.expect("Should have output") {
            TextInput::Line(text) => text.to_string(),
            _ => panic!("Unexpected output value"),
        };
        (text, flags)
    }

    #[cfg(test)]
    mod alias {
        use super::*;

        #[test]
        fn text_single() {
            let alias = Alias::compile_text("cook $1".to_string(), "Put $1 in a pan".to_string())
                .expect("Alias should compile!");
            let (output, flags) = process(alias, "cook chorizo");
            assert_eq!(output, "Put chorizo in a pan");
            assert_eq!(flags, ProcessedTextFlags::NONE);
        }

        #[test]
        fn after_line_start() {
            let alias = Alias::compile_text("cook $1".to_string(), "heat up $1".to_string())
                .expect("Alias should compile!");
            let (output, _) = process(alias, "happily cook chorizo");
            assert_eq!(output, "happily heat up chorizo");
        }
    }

    #[cfg(test)]
    mod manager {
        use super::*;

        fn define(manager: &mut TextProcessorManager<Alias>, pattern: &str, replacement: &str) {
            manager
                .insert_text(pattern.to_string(), replacement.to_string())
                .expect("Pattern should compile");
        }

        #[test]
        fn multi_process() {
            let mut manager: TextProcessorManager<Alias> = TextProcessorManager::new();
            define(&mut manager, "in a $1", "into a HOT pan");
            define(&mut manager, "cook $1", "Put $1 in a pan");

            let (output, _) = process(manager, "cook chorizo");
            assert_eq!(output, "Put chorizo into a HOT pan");
        }
    }
}
