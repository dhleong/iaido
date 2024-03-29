use crate::input::completion::CompletionContext;

use super::{weighted::WeightedRandomSelectorFactory, MultiplexSelectorFactory};

pub struct WordIndexWeightedRandomSelector {
    weights_by_index: Vec<Vec<u8>>,
}

impl WordIndexWeightedRandomSelector {
    pub fn with_weights_by_index(weights_by_index: Vec<Vec<u8>>) -> Self {
        Self { weights_by_index }
    }
}

impl MultiplexSelectorFactory for WordIndexWeightedRandomSelector {
    fn create(&self, context: CompletionContext) -> Box<dyn super::MultiplexSelector + Send> {
        let index = context.word_index().max(self.weights_by_index.len() - 1);
        let weights = &self.weights_by_index[index];
        WeightedRandomSelectorFactory::with_weights(weights.clone()).create(context)
    }
}
