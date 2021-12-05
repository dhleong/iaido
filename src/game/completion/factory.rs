use super::flagged::FlaggedCompletionSource;
use super::multiplex::MultiplexCompletionSource;
use super::recency::RecencyCompletionSource;
use crate::game::completion::multiplex::weighted::WeightedRandomSelectorFactory;
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
            selector_factory: Box::new(WeightedRandomSelectorFactory::with_weights(vec![60, 40])),
        }
    }
}
