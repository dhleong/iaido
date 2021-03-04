use super::KeySource;
use crate::input::{keys::KeysParsable, Key};

/// A MemoryKeySource provides a fixed sequence of keys
/// from memory
pub struct MemoryKeySource {
    keys: Vec<Key>,
}

impl MemoryKeySource {
    pub fn empty() -> Self {
        Self {
            keys: Vec::default(),
        }
    }

    #[allow(unused)]
    pub fn from_keys<T: KeysParsable>(keys: T) -> Self {
        MemoryKeySource {
            keys: keys.into_keys(),
        }
    }
}

impl From<Vec<Key>> for MemoryKeySource {
    fn from(keys: Vec<Key>) -> Self {
        Self { keys }
    }
}

impl KeySource for MemoryKeySource {
    fn poll_key(&mut self, _timeout: std::time::Duration) -> Result<bool, crate::input::KeyError> {
        Ok(!self.keys.is_empty())
    }

    fn next_key(&mut self) -> Result<Option<Key>, crate::input::KeyError> {
        if self.keys.is_empty() {
            return Ok(None);
        }

        Ok(Some(self.keys.remove(0)))
    }

    fn feed_keys(&mut self, keys: Vec<Key>) {
        self.keys.splice(0..0, keys);
    }
}
