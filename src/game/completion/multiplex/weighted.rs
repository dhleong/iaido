use super::{MultiplexSelector, MultiplexSelectorFactory};
use crate::input::completion::{Completion, CompletionContext};

struct WeightedRandomSelector;

impl MultiplexSelector for WeightedRandomSelector {
    fn select(&mut self, _: Vec<Option<&Completion>>) -> usize {
        return 0;
    }
}

pub struct WeightedRandomSelectorFactory;

impl MultiplexSelectorFactory for WeightedRandomSelectorFactory {
    fn create(&self, _context: CompletionContext) -> Box<dyn MultiplexSelector> {
        Box::new(WeightedRandomSelector)
    }
}
