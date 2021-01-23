use std::time::Duration;

use super::{Key, KeyError, KeySource, KeymapContext};

pub mod vim;

pub struct KeyHandlerContext<'a, T> {
    context: Box<&'a mut dyn KeymapContext>,
    keymap: &'a mut T,
}

impl<'a, T> KeymapContext for KeyHandlerContext<'a, T> {
    fn state(&self) -> &crate::app::State {
        self.context.state()
    }
    fn state_mut(&mut self) -> &mut crate::app::State {
        self.context.state_mut()
    }
}

impl<'a, T> KeySource for KeyHandlerContext<'a, T> {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
        self.context.poll_key(timeout)
    }
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        self.context.next_key()
    }
}

pub type KeyResult = Result<(), KeyError>;
pub type KeyHandler<'a, T> = dyn Fn(KeyHandlerContext<'a, T>) -> KeyResult;

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

    ($state_type:ident |?mut $ctx_name:ident| $body:expr) => {{
        Box::new(
            |$ctx_name: crate::input::maps::KeyHandlerContext<$state_type>| {
                let result: crate::input::maps::KeyResult = $body;
                result
            },
        )
    }};
}
