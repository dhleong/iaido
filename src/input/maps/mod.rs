use std::time::Duration;

use crate::delegate_keysource_with_map;

use super::{
    commands::CommandHandlerContext, source::memory::MemoryKeySource, BoxableKeymap, Key, KeyError,
    KeySource, Keymap, KeymapContext, KeymapContextWithKeys,
};

pub mod actions;
pub mod vim;

pub struct KeyHandlerContext<'a, T: BoxableKeymap> {
    context: Box<&'a mut dyn KeymapContext>,
    pub keymap: &'a mut T,
    key: Key,
}

impl<T: BoxableKeymap + Keymap> KeyHandlerContext<'_, T> {
    pub fn feed_keys(mut self, keys: Vec<Key>) -> Result<Self, KeyError> {
        let source = MemoryKeySource::from(keys);
        let mut context = KeymapContextWithKeys::new(self.context, source);

        while context.keys.poll_key(Duration::from_millis(0))? {
            self.keymap.process(&mut context)?;
        }

        self.context = context.base;

        Ok(self)
    }
}

impl<'a, T: BoxableKeymap> KeymapContext for KeyHandlerContext<'a, T> {
    fn state(&self) -> &crate::app::State {
        self.context.state()
    }
    fn state_mut(&mut self) -> &mut crate::app::State {
        self.context.state_mut()
    }
}

impl<'a, T: BoxableKeymap> KeySource for KeyHandlerContext<'a, T> {
    delegate_keysource_with_map!(context, keymap);
}

pub type KeyResult<T = ()> = Result<T, KeyError>;
pub type KeyHandler<T> = dyn Fn(KeyHandlerContext<'_, T>) -> KeyResult;
pub type UserKeyHandler = dyn Fn(CommandHandlerContext<'_>) -> KeyResult;

/// Syntactic sugar for declaring a key handler
#[macro_export]
macro_rules! key_handler {
    ($state_type:ident |$ctx_name:ident| $body:expr) => {{
        Box::new(
            |mut $ctx_name: crate::input::maps::KeyHandlerContext<$state_type>| {
                let result: crate::input::maps::KeyResult = $body;
                result
            },
        )
    }};

    ($state_type:ident move |$ctx_name:ident| $body:expr) => {{
        Box::new(
            move |mut $ctx_name: crate::input::maps::KeyHandlerContext<$state_type>| {
                let result: crate::input::maps::KeyResult = $body;
                result
            },
        )
    }};

    ($state_type:ident |?mut $ctx_name:ident| $body:expr) => {{
        Box::new(
            |$ctx_name: crate::input::maps::KeyHandlerContext<$state_type>| {
                let result: crate::input::maps::KeyResult = $body;
                result
            },
        )
    }};

    ($state_type:ident move |?mut $ctx_name:ident| $body:expr) => {{
        Box::new(
            move |$ctx_name: crate::input::maps::KeyHandlerContext<$state_type>| {
                let result: crate::input::maps::KeyResult = $body;
                result
            },
        )
    }};
}
