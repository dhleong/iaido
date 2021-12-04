use super::multiplex::MultiplexCompletionSource;
use crate::game::completion::multiplex::weighted::WeightedRandomSelectorFactory;
use crate::game::completion::CompletionSource;

pub struct GameCompletionsFactory;

impl GameCompletionsFactory {
    pub fn create() -> MultiplexCompletionSource<Box<dyn CompletionSource>> {
        MultiplexCompletionSource {
            sources: vec![],
            selector_factory: Box::new(WeightedRandomSelectorFactory::with_weights(vec![100])),
        }
    }
}
