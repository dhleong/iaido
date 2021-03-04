use std::time::Duration;

use crate::delegate_keysource;

use super::{
    source::memory::MemoryKeySource, Key, KeyError, KeySource, Keymap, KeymapContext,
    KeymapContextWithKeys,
};

pub mod actions;
pub mod vim;

pub struct KeyHandlerContext<'a, T> {
    context: Box<&'a mut dyn KeymapContext>,
    pub keymap: &'a mut T,
    key: Key,
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
    delegate_keysource! { context }
}

pub type KeyResult = Result<(), KeyError>;
pub type KeyHandler<T> = dyn Fn(KeyHandlerContext<'_, T>) -> KeyResult;

fn feed_keys<T: Keymap>(ctx: KeyHandlerContext<T>, keys: Vec<Key>) -> KeyResult {
    let source = MemoryKeySource::from(keys);
    let mut context = KeymapContextWithKeys::new(ctx.context, source);

    while context.keys.poll_key(Duration::from_millis(0))? {
        ctx.keymap.process(&mut context)?;
    }

    Ok(())
}

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
