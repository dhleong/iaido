pub mod memory;

use std::time::Duration;

use super::{BoxableKeymap, Key, KeyError};

pub trait KeySource {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
        self.poll_key_with_map(timeout, None)
    }
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        self.next_key_with_map(None)
    }

    fn poll_key_with_map(
        &mut self,
        timeout: Duration,
        keymap: Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<bool, KeyError>;

    fn next_key_with_map(
        &mut self,
        keymap: Option<Box<&mut dyn BoxableKeymap>>,
    ) -> Result<Option<Key>, KeyError>;
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

        fn poll_key_with_map(
            &mut self,
            timeout: Duration,
            keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>,
        ) -> Result<bool, crate::input::KeyError> {
            self.$base_source.poll_key_with_map(timeout, keymap)
        }
        fn next_key_with_map(
            &mut self,
            keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>,
        ) -> Result<Option<crate::input::Key>, crate::input::KeyError> {
            self.$base_source.next_key_with_map(keymap)
        }
    };
}

#[macro_export]
macro_rules! delegate_keysource_with_map {
    ($base_source:ident, &mut $keymap:ident) => {
        delegate::delegate! {
            to self.$base_source {
                fn poll_key_with_map(&mut self, timeout: Duration, keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>) -> Result<bool, KeyError>;
                fn next_key_with_map(&mut self, keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>) -> Result<Option<Key>, KeyError>;
            }
        }

        fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
            self.$base_source
                .next_key_with_map(Some(Box::new(&mut self.$keymap)))
        }

        fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
            self.$base_source
                .poll_key_with_map(timeout, Some(Box::new(&mut self.$keymap)))
        }
    };

    ($base_source:ident, $keymap:ident) => {
        delegate::delegate! {
            to self.$base_source {
                fn poll_key_with_map(&mut self, timeout: Duration, keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>) -> Result<bool, KeyError>;
                fn next_key_with_map(&mut self, keymap: Option<Box<&mut dyn crate::input::BoxableKeymap>>) -> Result<Option<Key>, KeyError>;
            }
        }

        fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
            self.$base_source
                .next_key_with_map(Some(Box::new(self.$keymap)))
        }

        fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
            self.$base_source
                .poll_key_with_map(timeout, Some(Box::new(self.$keymap)))
        }
    };
}
