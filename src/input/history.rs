use std::collections::vec_deque::Iter;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::app::prompt::Prompt;

const DEFAULT_HISTORY_LIMIT: usize = 10000;

pub struct History<T> {
    items: VecDeque<T>,
    pub limit: usize,
}

impl<T> History<T> {
    pub fn with_limit(limit: usize) -> Self {
        Self {
            items: Default::default(),
            limit,
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.nth(0)
    }

    pub fn nth(&self, n: usize) -> Option<&T> {
        self.items.get(n)
    }

    pub fn insert(&mut self, item: T) {
        while self.items.len() >= self.limit {
            self.items.pop_back();
        }
        self.items.push_front(item);
    }

    #[allow(dead_code)] // NOTE: We use it in a test, and will need it later
    pub fn iter(&self) -> Iter<T> {
        self.items.iter()
    }
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self::with_limit(DEFAULT_HISTORY_LIMIT)
    }
}

#[derive(Default)]
pub struct StringHistories {
    histories: HashMap<String, History<String>>,
}

impl StringHistories {
    pub fn get(&mut self, key: String) -> &mut History<String> {
        self.histories
            .entry(key)
            .or_insert_with(|| Default::default())
    }

    pub fn take(&mut self, key: &String) -> History<String> {
        self.histories
            .remove(key)
            .unwrap_or_else(|| Default::default())
    }

    pub fn replace(&mut self, key: String, history: History<String>) {
        self.histories.insert(key, history);
    }

    pub fn get_most_recent(&self, key: &str) -> Option<&String> {
        if let Some(history) = self.histories.get(key) {
            history.first()
        } else {
            None
        }
    }

    pub fn maybe_insert(&mut self, key: String, entry: String) {
        // TODO This should actually *remove* older matching entries, per :help cmdline-history
        let history = self.get(key);
        if let Some(existing) = history.first() {
            if existing.to_owned() == entry {
                return;
            }
        }

        history.insert(entry)
    }
}

#[derive(Default)]
pub struct HistoryCursor {
    index: Option<usize>,
    stashed_input: Option<String>,
}

impl HistoryCursor {
    pub fn back<'h, T>(
        &mut self,
        prompt: &Prompt,
        prompt_len: usize,
        history: &'h History<T>,
    ) -> Option<&'h T> {
        if let Some(previous_index) = self.index {
            // TODO If stashed_input is not None, vim would search for a prefix match
            let next_index = previous_index + 1;
            if let Some(next) = history.iter().skip(next_index).next() {
                self.index = Some(next_index);
                Some(next)
            } else {
                None
            }
        } else if let Some(first_history) = history.nth(0) {
            self.stashed_input = Some((&prompt.buffer.get_contents()[prompt_len..]).to_string());
            self.index = Some(0);
            Some(first_history)
        } else {
            None
        }
    }

    pub fn forward<'h, T>(&mut self, history: &'h History<T>) -> Option<&'h T> {
        if let Some(previous_index) = self.index {
            // TODO If stashed_input is not None, vim would search for a prefix match
            if previous_index > 0 {
                let new_index = previous_index - 1;
                self.index = Some(new_index);
                history.nth(new_index)
            } else {
                self.index = None;
                None
            }
        } else {
            None
        }
    }

    pub fn take_stashed_input(&mut self) -> Option<String> {
        self.stashed_input.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_limit() {
        let mut history = History::<&str>::with_limit(2);
        history.insert("First");
        history.insert("Second");
        history.insert("Third");
        let contents: Vec<String> = history.iter().map(|s| s.to_string()).collect();
        assert_eq!(contents, vec!["Third", "Second"]);
    }
}
