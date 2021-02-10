pub mod core;
pub mod registry;

use std::time::Duration;

use self::{core::declare_core, registry::CommandRegistry};

use super::{maps::KeyResult, Key, KeyError, KeySource, KeymapContext};

pub type CommandHandler = dyn Fn(&mut CommandHandlerContext<'_>) -> KeyResult;

pub struct CommandHandlerContext<'a> {
    pub context: Box<&'a mut dyn KeymapContext>,
    pub input: String,
}

impl<'a> CommandHandlerContext<'a> {
    pub fn new<T: KeymapContext>(context: &'a mut T, input: String) -> Self {
        Self {
            context: Box::new(context),
            input,
        }
    }
}

impl KeymapContext for CommandHandlerContext<'_> {
    fn state(&self) -> &crate::app::State {
        self.context.state()
    }
    fn state_mut(&mut self) -> &mut crate::app::State {
        self.context.state_mut()
    }
}

impl KeySource for CommandHandlerContext<'_> {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
        self.context.poll_key(timeout)
    }
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        self.context.next_key()
    }
}

pub fn create_builtin_commands() -> CommandRegistry {
    let mut registry = CommandRegistry::default();
    declare_core(&mut registry);
    return registry;
}
