pub mod memory;

use std::time::Duration;

use super::{BoxableKeymap, Key, KeyError};

pub trait KeySource {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError>;
    fn next_key_with_map(
        &mut self,
        keymap: Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<Option<Key>, KeyError>;
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        self.next_key_with_map(None)
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
        fn next_key_with_map(
            &mut self,
            keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>,
        ) -> Result<Option<crate::input::Key>, crate::input::KeyError> {
            self.$base_source.next_key_with_map(keymap)
        }
    };
}
