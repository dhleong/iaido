use std::collections::vec_deque::Iter;
use std::collections::HashMap;
use std::collections::VecDeque;

const DEFAULT_HISTORY_LIMIT: usize = 10000;

pub struct History<T> {
    items: VecDeque<T>,
    pub limit: usize,
}

impl<T: PartialEq> History<T> {
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
        // Per :help cmdline-history, inserting items that exactly match existing
        // items first removes the existing item
        if let Some(index) = self.items.iter().position(|candidate| &item == candidate) {
            self.items.remove(index);
        }

        while self.items.len() >= self.limit {
            self.items.pop_back();
        }
        self.items.push_front(item);
    }

    pub fn iter(&self) -> Iter<T> {
        self.items.iter()
    }
}

impl<T: Eq> Default for History<T> {
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
        self.get(key).insert(entry)
    }
}

pub trait HistoryCursorable {
    fn filter_history(&self, candidate: &Self) -> bool;
}

impl HistoryCursorable for String {
    fn filter_history(&self, candidate: &String) -> bool {
        (&self[..]).filter_history(&&candidate[..])
    }
}

impl HistoryCursorable for &str {
    fn filter_history(&self, candidate: &&str) -> bool {
        candidate.starts_with(self)
    }
}

#[derive(Default)]
pub struct HistoryCursor<T: HistoryCursorable + PartialEq> {
    index: Option<usize>,
    stashed_input: Option<T>,
}

impl<T: HistoryCursorable + PartialEq> HistoryCursor<T> {
    pub fn back<'h, 'a: 'h>(
        &mut self,
        stash_input: impl Fn() -> T,
        history: &'h History<T>,
    ) -> Option<&'h T> {
        let next_index = if let Some(previous_index) = self.index {
            previous_index + 1
        } else {
            self.stashed_input = Some(stash_input());
            0
        };

        let mut iter = history
            .iter()
            .filter(|candidate| self.filter_history(candidate))
            .skip(next_index);
        if let Some(next) = iter.next() {
            self.index = Some(next_index);
            Some(next)
        } else {
            None
        }
    }

    pub fn forward<'h, 'a: 'h>(&'a mut self, history: &'h History<T>) -> Option<&'h T> {
        if let Some(previous_index) = self.index {
            if previous_index > 0 {
                let new_index = previous_index - 1;
                self.index = Some(new_index);
                history
                    .iter()
                    .filter(|candidate| self.filter_history(candidate))
                    .nth(new_index)
            } else {
                self.index = None;
                self.stashed_input.as_ref()
            }
        } else {
            None
        }
    }

    fn filter_history(&self, candidate: &T) -> bool {
        // NOTE: If stashed_input is not empty, vim searches for a prefix match:
        if let Some(stashed) = self.stashed_input.as_ref() {
            stashed.filter_history(candidate)
        } else {
            true
        }
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

    #[test]
    fn inserts_deduped() {
        let mut history = History::<&str>::default();
        history.insert("First");
        history.insert("Second");
        history.insert("Third");
        let contents: Vec<String> = history.iter().map(|s| s.to_string()).collect();
        assert_eq!(contents, vec!["Third", "Second", "First"]);

        history.insert("First");
        let contents: Vec<String> = history.iter().map(|s| s.to_string()).collect();
        assert_eq!(contents, vec!["First", "Third", "Second"]);
    }

    mod cursor_tests {
        use super::*;

        #[test]
        fn navigate_simple() {
            let mut history = History::<&str>::with_limit(2);
            history.insert("First");
            history.insert("Second");

            let stash_input = || "";

            let mut cursor = HistoryCursor::<&str>::default();
            let entry = cursor.back(stash_input, &history);
            assert_eq!(entry.unwrap().to_owned(), "Second");

            let entry = cursor.back(stash_input, &history);
            assert_eq!(entry.unwrap().to_owned(), "First");

            // No more history returns None
            assert_eq!(cursor.back(stash_input, &history), None);

            let entry = cursor.forward(&history);
            assert_eq!(entry.unwrap().to_owned(), "Second");

            // We had no input in the prompt, so we get a blank string back:
            let entry = cursor.forward(&history);
            assert_eq!(entry.unwrap().to_owned(), "");

            // And no further to go:
            assert_eq!(cursor.forward(&history), None);
            assert_eq!(cursor.forward(&history), None);

            // If we go back again with new content...
            let stash_input = || "Sec";
            let entry = cursor.back(stash_input, &history);
            assert_eq!(entry.unwrap().to_owned(), "Second");

            // ... we should return to the new content
            let entry = cursor.forward(&history);
            assert_eq!(entry.unwrap().to_owned(), "Sec");
        }

        #[test]
        fn navigate_filtered() {
            let mut history = History::<&str>::with_limit(2);
            history.insert("First");
            history.insert("Second");

            let stash_input = || "Fir";

            let mut cursor = HistoryCursor::<&str>::default();
            let entry = cursor.back(stash_input, &history);
            assert_eq!(entry.unwrap().to_owned(), "First");

            // No more filtered history returns None
            assert_eq!(cursor.back(stash_input, &history), None);
            assert_eq!(cursor.back(stash_input, &history), None);

            // Skip back over Second to our stashed input
            let entry = cursor.forward(&history);
            assert_eq!(entry.unwrap().to_owned(), "Fir");

            // And no further to go:
            assert_eq!(cursor.forward(&history), None);
            assert_eq!(cursor.forward(&history), None);
        }
    }
}
