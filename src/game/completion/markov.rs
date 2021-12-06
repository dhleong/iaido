use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::input::completion::{
    BoxedSuggestions, CompletableContext, Completer, Completion, CompletionContext,
};

use super::flagged::SimpleCompletionSource;
use super::tokens::CompletionTokenizable;

#[derive(Default)]
pub struct MarkovCompletionSource {
    trie: MarkovTrie<String>,
}

fn tokens(text: &str) -> Vec<String> {
    text.to_all_completion_tokens()
        .iter()
        .map(|s| s.to_lowercase().to_string())
        .collect()
}

impl Completer for MarkovCompletionSource {
    fn suggest(
        &self,
        _app: Box<&dyn CompletableContext>,
        context: CompletionContext,
    ) -> BoxedSuggestions {
        let partial = context.word().to_lowercase();
        let tokens_before = tokens(context.line_before_cursor());

        let nodes = self.trie.root.query(&tokens_before[..]);
        let mut sorted: Vec<&&MarkovNode<String>> = nodes
            .iter()
            .filter(|node| node.value.starts_with(&partial))
            .collect();
        sorted.sort_unstable_by_key(|node| node.incoming_count);

        let completions: Vec<Completion> = sorted
            .iter()
            .rev()
            .map(|node| context.create_completion(node.value.clone()))
            .collect();
        Box::new(completions.into_iter())
    }
}

impl SimpleCompletionSource for MarkovCompletionSource {
    fn process(&mut self, text: String) {
        let mut tokens = tokens(&text);
        self.trie.add(&mut tokens[..]);
    }
}

#[derive(Default)]
struct MarkovTrie<T> {
    root: MarkovTransitions<T>,
    max_depth: usize,
    stop_words: HashSet<T>,
}

impl<T: Default + Hash + Eq + Clone> MarkovTrie<T> {
    fn add(&mut self, sequence: &mut [T]) {
        if sequence.is_empty() {
            return;
        }
        self.root.add(sequence, &self.stop_words, self.max_depth);
    }
}

#[derive(Default)]
struct MarkovTransitions<T> {
    transitions: HashMap<T, MarkovNode<T>>,
}

impl<T: Default + Hash + Eq + Clone> MarkovTransitions<T> {
    fn add(&mut self, sequence: &mut [T], stop_words: &HashSet<T>, remaining_depth: usize) {
        let next_value = sequence[0].clone();
        if stop_words.contains(&next_value) {
            return;
        }

        let mut transition = self
            .transitions
            .entry(next_value.clone())
            .or_insert_with(|| MarkovNode::from(next_value));
        transition.incoming_count += 1;

        if let Some(new_remaining_depth) = remaining_depth.checked_sub(1) {
            if sequence.len() > 1 {
                transition
                    .transitions
                    .add(&mut sequence[1..], stop_words, new_remaining_depth);
            }
        }
    }

    fn query(&self, sequence: &[T]) -> Vec<&MarkovNode<T>> {
        if sequence.is_empty() {
            let nodes: Vec<&MarkovNode<T>> = self.transitions.values().collect();
            return nodes;
        }

        let next_value = sequence[0].clone();
        if let Some(transition) = self.transitions.get(&next_value) {
            return transition.transitions.query(&sequence[1..]);
        }

        vec![]
    }
}

struct MarkovNode<T> {
    pub value: T,
    pub incoming_count: usize,
    pub transitions: MarkovTransitions<T>,
}

impl<T: Default> From<T> for MarkovNode<T> {
    fn from(value: T) -> Self {
        MarkovNode {
            value,
            incoming_count: 0,
            transitions: MarkovTransitions::default(),
        }
    }
}
