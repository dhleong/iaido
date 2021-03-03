pub mod memory;

use std::time::Duration;

use super::{Key, KeyError};

pub trait KeySource {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError>;
    fn next_key(&mut self) -> Result<Option<Key>, KeyError>;

    fn can_feed(&self) -> bool {
        false
    }
    fn feed_keys(&self, _keys: Vec<Key>) {
        panic!("KeySource does not support feed_keys");
    }
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

        fn can_feed(&self) -> bool {
            self.$base_source.can_feed()
        }
        fn feed_keys(&self, keys: Vec<crate::input::Key>) {
            self.$base_source.feed_keys(keys)
        }
    };
}
