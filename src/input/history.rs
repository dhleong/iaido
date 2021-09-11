use std::collections::HashMap;

pub struct History<T> {
    pub items: Vec<T>,
}

impl<T> History<T> {
    pub fn first(&self) -> Option<&T> {
        self.items.get(0)
    }

    pub fn insert(&mut self, item: T) {
        // TODO History limits
        self.items.insert(0, item)
    }
}

impl<T> Default for History<T> {
    fn default() -> Self {
        Self {
            items: Default::default(),
        }
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
