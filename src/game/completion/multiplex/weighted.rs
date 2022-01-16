use crate::game::completion::multiplex::{MultiplexSelector, MultiplexSelectorFactory};
use crate::input::completion::{Completion, CompletionContext};
use rand::Rng;

pub trait RandomnessSource: Clone {
    fn next_percentage(&mut self) -> u8;
}

#[derive(Clone)]
pub struct ThreadRngRandomnessSource;
impl RandomnessSource for ThreadRngRandomnessSource {
    fn next_percentage(&mut self) -> u8 {
        rand::thread_rng().gen_range(0..=100)
    }
}

struct WeightedRandomSelector<T: RandomnessSource + Send> {
    pub weights: Vec<u8>,
    pub random: T,
}

impl<T: RandomnessSource + Send> MultiplexSelector for WeightedRandomSelector<T> {
    fn select(&mut self, candidates: Vec<Option<&Completion>>) -> usize {
        let mut die_roll = self.random.next_percentage();
        let mut sorted_weight_indexes: Vec<usize> = (0..self.weights.len()).collect();
        sorted_weight_indexes.sort_by_key(|idx| self.weights[idx.to_owned()]);
        loop {
            let mut empty_weight = 0;
            let mut last_weight = 0;

            for i in 0..self.weights.len() {
                let weight_index = sorted_weight_indexes[i];
                let weight = self.weights[weight_index];
                let total_weight = last_weight + weight;

                if die_roll <= total_weight {
                    if candidates[weight_index].is_none() {
                        empty_weight += weight;
                    } else {
                        return weight_index;
                    }
                }

                last_weight = total_weight;
            }

            die_roll = if let Some(next_roll) = die_roll.checked_sub(empty_weight) {
                next_roll
            } else {
                break;
            };

            if empty_weight == 0 {
                break;
            }
        }

        panic!("{} not in range of any weight!", die_roll);
    }
}

pub struct WeightedRandomSelectorFactory<T: 'static + RandomnessSource + Send> {
    pub weights: Vec<u8>,
    pub random: T,
}

impl WeightedRandomSelectorFactory<ThreadRngRandomnessSource> {
    pub fn with_weights(weights: Vec<u8>) -> Self {
        if weights.iter().sum::<u8>() != 100 {
            panic!("Weights must sum to 100; received {:?}", weights);
        }

        Self {
            weights,
            random: ThreadRngRandomnessSource,
        }
    }
}

impl<T: RandomnessSource + Send> MultiplexSelectorFactory for WeightedRandomSelectorFactory<T> {
    fn create(&self, _context: CompletionContext) -> Box<dyn MultiplexSelector + Send> {
        Box::new(WeightedRandomSelector {
            weights: self.weights.clone(),
            random: self.random.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;

    #[derive(Clone)]
    pub struct StaticRandomnessSource {
        values: Vec<u8>,
    }
    impl StaticRandomnessSource {
        pub fn with_values(values: Vec<u8>) -> Self {
            Self { values }
        }
    }

    impl RandomnessSource for StaticRandomnessSource {
        fn next_percentage(&mut self) -> u8 {
            if self.values.is_empty() {
                0
            } else {
                self.values.remove(0)
            }
        }
    }

    fn candidates(strings: Vec<&str>) -> Vec<Option<Completion>> {
        strings
            .iter()
            .map(|s| {
                if s.is_empty() {
                    None
                } else {
                    Some(Completion {
                        line_index: 0,
                        start: 0,
                        end: 0,
                        replacement: s.to_string(),
                    })
                }
            })
            .collect()
    }

    fn select(
        selector: &mut Box<dyn MultiplexSelector + Send>,
        candidate_strings: Vec<&str>,
    ) -> usize {
        let candidates = candidates(candidate_strings);
        selector.select(
            candidates
                .iter()
                .map(|cand| {
                    if let Some(cand) = cand {
                        Some(cand)
                    } else {
                        None
                    }
                })
                .collect(),
        )
    }

    #[test]
    pub fn weighted_selection() {
        let random = StaticRandomnessSource::with_values(vec![39, 41, 42, 20, 2]);
        let mut selector = WeightedRandomSelectorFactory {
            weights: vec![60, 40],
            random,
        }
        .create(CompletionContext::from(&mut window("")));

        // 0.39 - below 40 should go to second source
        assert_eq!(select(&mut selector, vec!["taco", "alpastor"]), 1);

        // 0.41 - above 40 should go to first source
        assert_eq!(select(&mut selector, vec!["taco", "alpastor"]), 0);
        // 0.42
        assert_eq!(select(&mut selector, vec!["taco", "alpastor"]), 0);
        // 0.20 -  resume second
        assert_eq!(select(&mut selector, vec!["taco", "alpastor"]), 1);
        // 0.02 -  second is empty; go with first
        assert_eq!(select(&mut selector, vec!["taco", ""]), 0);
    }
}
