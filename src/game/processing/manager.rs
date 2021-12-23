use std::collections::HashMap;

use crate::input::{maps::KeyResult, KeyError};

use super::{ProcessedText, ProcessedTextFlags, TextInput, TextProcessor};

const MAX_ITERATIONS: usize = 50;

pub struct TextProcessorManager<T: TextProcessor> {
    processors: HashMap<String, T>,
}

impl<T: TextProcessor> TextProcessorManager<T> {
    #[allow(dead_code)] // TODO remove when able
    pub fn new() -> Self {
        Self {
            processors: Default::default(),
        }
    }

    pub fn insert(&mut self, description: String, processor: T) -> Option<T> {
        self.processors.insert(description, processor)
    }

    fn process_once(&mut self, input: TextInput) -> KeyResult<Option<ProcessedText>> {
        let mut to_process = Some(input);
        let mut any_processed = false;
        let mut to_remove = vec![];

        for (id, processor) in self.processors.iter_mut() {
            let to_consume = if let Some(processable) = to_process.take() {
                processable
            } else {
                break;
            };

            let (processed, flags) = match processor.process(to_consume.clone())? {
                None => {
                    to_process = Some(to_consume);
                    (false, ProcessedTextFlags::NONE)
                }
                Some(ProcessedText(Some(output), flags)) => {
                    to_process = Some(output);
                    (true, flags)
                }
                Some(ProcessedText(None, flags)) => (true, flags),
            };

            any_processed |= processed;

            if flags.contains(ProcessedTextFlags::DESTROYED) {
                to_remove.push(id.to_string());
            }
        }

        for id in to_remove {
            self.processors.remove(&id);
        }

        if !any_processed {
            Ok(None)
        } else {
            Ok(Some(ProcessedText(to_process, ProcessedTextFlags::NONE)))
        }
    }
}

impl<T: TextProcessor> TextProcessor for TextProcessorManager<T> {
    fn describe(&self) -> &str {
        "Manager"
    }

    fn process(&mut self, input: TextInput) -> KeyResult<Option<ProcessedText>> {
        // NOTE: Some Matchers may not match until a *subsequent* matcher processes the input, so
        // we continue processing in a loop until everybody is done with the processed output
        let mut to_process = input;
        let mut any_processed = false;

        for _ in 0..MAX_ITERATIONS {
            let result = self.process_once(to_process.clone())?;
            match result {
                None => {
                    return Ok(if any_processed {
                        Some(ProcessedText(Some(to_process), ProcessedTextFlags::NONE))
                    } else {
                        None
                    })
                }
                Some(ProcessedText(None, flags)) => {
                    // Some processor wants to remove the line;
                    // shortcut here to do so
                    return Ok(Some(ProcessedText(None, flags)));
                }
                Some(ProcessedText(Some(output), _)) => {
                    any_processed = true;
                    to_process = output;
                }
            }
        }

        Err(KeyError::InvalidInput(
            "Infinite recursion detected".to_string(),
        ))
    }
}
