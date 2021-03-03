pub mod memory;

use std::time::Duration;

use super::{Key, KeyError};

pub trait KeySource {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError>;
    fn next_key(&mut self) -> Result<Option<Key>, KeyError>;

    fn feed_keys(&mut self, keys: Vec<Key>);
}

#[macro_export]
macro_rules! delegate_keysource {
    ($base_source:ident) => {
        fn poll_key(&mut self, timeout: Duration) -> Result<bool, crate::input::KeyError> {
            self.$base_source.poll_key(timeout)
        }
        fn next_key(&mut self) -> Result<Option<crate::input::Key>, crate::input::KeyError> {
            self.$base_source.next_key()
        }

        fn feed_keys(&mut self, keys: Vec<crate::input::Key>) {
            self.$base_source.feed_keys(keys)
        }
    };
}
