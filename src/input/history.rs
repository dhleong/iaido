use std::collections::vec_deque::Iter;
use std::collections::HashMap;
use std::collections::VecDeque;

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
        self.items.get(0)
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

    pub fn get_most_recent(&self, key: String) -> Option<&String> {
        if let Some(history) = self.histories.get(&key) {
            history.first()
        } else {
            None
        }
    }

    pub fn maybe_insert(&mut self, key: String, entry: String) {
        let history = self.get(key);
        if let Some(existing) = history.first() {
            if existing.to_owned() == entry {
                return;
            }
        }

        history.insert(entry)
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
