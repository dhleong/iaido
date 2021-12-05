use super::flagged::FlaggedCompletionSource;
use super::multiplex::MultiplexCompletionSource;
use super::recency::RecencyCompletionSource;
use crate::game::completion::multiplex::word_index::WordIndexWeightedRandomSelector;
use crate::game::completion::{CompletionSource, ProcessFlags};

pub struct GameCompletionsFactory;

impl GameCompletionsFactory {
    pub fn create() -> MultiplexCompletionSource<Box<dyn CompletionSource>> {
        let received_source = FlaggedCompletionSource::accepting_flags(
            RecencyCompletionSource::default(),
            ProcessFlags::RECEIVED,
        );

        let sent_source = FlaggedCompletionSource::accepting_flags(
            RecencyCompletionSource::default(),
            ProcessFlags::SENT,
        );

        MultiplexCompletionSource {
            sources: vec![Box::new(sent_source), Box::new(received_source)],
            selector_factory: Box::new(WordIndexWeightedRandomSelector::with_weights_by_index(
                vec![
                    // First word? Prefer commandCompletions ALWAYS; We'll still
                    // fallback to output if commandCompletion doesn't have anything
                    vec![100, 0],
                    // Second word? Actually, prefer output a bit
                    // eg: get <thing>; enter <thing>; look <thing>
                    vec![35, 65],
                    // Otherwise, just split it evenly
                    vec![50, 50],
                ],
            )),
        }
    }
}
