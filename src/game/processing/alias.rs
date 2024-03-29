use std::collections::HashMap;
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

type Processor = dyn Fn(Match) -> KeyResult<Option<TextLine>> + Send;

pub struct Alias {
    matcher: Matcher,
    processor: Box<Processor>,
    one_shot: bool,
}

impl Alias {
    pub fn compile_text(input: String, replacement: String) -> KeyResult<Alias> {
        Self::compile_fn(
            input,
            SubstitutionProcessor { replacement }.into_processor(),
        )
    }

    pub fn compile_fn(input: String, processor: Box<Processor>) -> KeyResult<Alias> {
        Ok(Alias {
            matcher: Matcher::compile(input)?,
            processor,
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

    fn process(&mut self, input: TextInput) -> KeyResult<ProcessedText> {
        match input {
            TextInput::Newline => Ok(ProcessedText::Unprocessed(input)),
            TextInput::Line(input_text) => {
                if let Some(found) = self.matcher.find(&input_text) {
                    let flags = if self.one_shot {
                        ProcessedTextFlags::DESTROYED
                    } else {
                        ProcessedTextFlags::NONE
                    };

                    let range = found.start..found.end;
                    let result = match (self.processor)(found) {
                        Ok(None) => ProcessedText::Removed(flags),
                        Ok(Some(mut output)) => {
                            let with_replacement = input_text.replacing_range(range, &mut output);

                            ProcessedText::Processed(TextInput::Line(with_replacement), flags)
                        }
                        Err(e) => return Err(e),
                    };

                    Ok(result)
                } else {
                    Ok(ProcessedText::Unprocessed(TextInput::Line(input_text)))
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
        Box::new(move |m| Ok(Some(m.expand(&self.replacement).into())))
    }
}

impl TextProcessorManager<Alias> {
    pub fn insert_text(&mut self, pattern: String, replacement: String) -> KeyResult {
        let alias = Alias::compile_text(pattern.to_string(), replacement)?;
        self.insert(pattern, alias);
        Ok(())
    }

    pub fn insert_fn(
        &mut self,
        pattern: String,
        replacement: Box<dyn Fn(HashMap<String, String>) -> KeyResult<Option<String>> + Send>,
    ) -> KeyResult {
        let alias = Alias::compile_fn(
            pattern.to_string(),
            Box::new(move |m| {
                let map: HashMap<String, String> = m
                    .groups
                    .iter()
                    .map(|(k, v)| (k.to_string(), v.to_string()))
                    .collect();
                if let Some(s) = replacement(map)? {
                    Ok(Some(s.into()))
                } else {
                    Ok(None)
                }
            }),
        )?;
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
        match processor
            .process(TextInput::Line(input.into()))
            .expect("Should process without error")
        {
            ProcessedText::Processed(TextInput::Line(output), flags) => (output.to_string(), flags),
            unexpected => panic!("Expected Processed result; got {:?}", unexpected),
        }
    }

    #[cfg(test)]
    mod alias {
        use super::*;

        #[test]
        fn text_without_vars() {
            let alias = Alias::compile_text("cook".to_string(), "braise".to_string())
                .expect("Alias should compile!");
            let (output, flags) = process(alias, "cook chorizo");
            assert_eq!(output, "braise chorizo");
            assert_eq!(flags, ProcessedTextFlags::NONE);
        }

        #[test]
        fn text_single() {
            let alias = Alias::compile_text("cook $1".to_string(), "Put $1 in a pan".to_string())
                .expect("Alias should compile!");
            let (output, flags) = process(alias, "cook chorizo");
            assert_eq!(output, "Put chorizo in a pan");
            assert_eq!(flags, ProcessedTextFlags::NONE);
        }

        #[test]
        fn text_single_with_trail() {
            let alias =
                Alias::compile_text("cook $1 rare".to_string(), "Sear $1 in a pan".to_string())
                    .expect("Alias should compile!");
            let (output, flags) = process(alias, "cook chorizo rare");
            assert_eq!(output, "Sear chorizo in a pan");
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
