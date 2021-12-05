use super::multiplex::MultiplexCompletionSource;
use super::recency::RecencyCompletionSource;
use crate::game::completion::multiplex::weighted::WeightedRandomSelectorFactory;
use crate::game::completion::CompletionSource;

pub struct GameCompletionsFactory;

impl GameCompletionsFactory {
    pub fn create() -> MultiplexCompletionSource<Box<dyn CompletionSource>> {
        // TODO: Distinguish completion processing between sent and received
        let received_source = RecencyCompletionSource::default();
        MultiplexCompletionSource {
            sources: vec![Box::new(received_source)],
            selector_factory: Box::new(WeightedRandomSelectorFactory::with_weights(vec![100])),
        }
    }
}
