use super::multiplex::MultiplexCompletionSource;
use crate::game::completion::multiplex::weighted::WeightedRandomSelectorFactory;

pub struct GameCompletionsFactory;

impl GameCompletionsFactory {
    pub fn create() -> MultiplexCompletionSource {
        MultiplexCompletionSource {
            sources: vec![],
            selector_factory: Box::new(WeightedRandomSelectorFactory),
        }
    }
}
