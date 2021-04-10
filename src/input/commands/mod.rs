pub mod colors;
pub mod connection;
pub mod core;
pub mod file;
pub mod log;
pub mod mapping;
pub mod registry;
pub mod window;

mod helpers;

use std::time::Duration;

use delegate::delegate;

use self::{
    colors::declare_colors, connection::declare_connection, core::declare_core, file::declare_file,
    log::declare_log, mapping::declare_mapping, registry::CommandRegistry, window::declare_window,
};

use super::{maps::KeyResult, BoxableKeymap, Key, KeyError, KeySource, KeymapContext};

pub type CommandHandler = dyn Fn(&mut CommandHandlerContext<'_>) -> KeyResult;

pub struct CommandHandlerContext<'a> {
    pub context: Box<&'a mut dyn KeymapContext>,
    pub keymap: Box<&'a mut dyn BoxableKeymap>,
    pub input: String,
}

impl<'a> CommandHandlerContext<'a> {
    pub fn new<T: KeymapContext, K: BoxableKeymap>(
        context: &'a mut T,
        keymap: &'a mut K,
        input: String,
    ) -> Self {
        Self {
            context: Box::new(context),
            keymap: Box::new(keymap),
            input,
        }
    }
}

impl CommandHandlerContext<'_> {
    pub fn args(&self) -> Vec<&str> {
        self.split_input().skip(1).collect()
    }

    pub fn command(&self) -> Option<&str> {
        if let Some(cmd) = self.split_input().next() {
            if cmd.is_empty() {
                None
            } else {
                Some(cmd)
            }
        } else {
            None
        }
    }

    fn split_input(&self) -> impl Iterator<Item = &str> {
        // TODO handle quoted input
        self.input.split(" ")
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
    delegate! {
        to self.context {
            fn poll_key_with_map(&mut self, timeout: Duration, keymap: Option<Box<&mut dyn BoxableKeymap>>) -> Result<bool, KeyError>;
            fn next_key_with_map(&mut self, keymap: Option<Box<&mut dyn BoxableKeymap>>) -> Result<Option<Key>, KeyError>;
        }
    }

    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        self.context
            .next_key_with_map(Some(Box::new(&mut self.keymap)))
    }

    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError> {
        self.context
            .poll_key_with_map(timeout, Some(Box::new(&mut self.keymap)))
    }
}

pub fn create_builtin_commands() -> CommandRegistry {
    let mut registry = CommandRegistry::default();
    declare_colors(&mut registry);
    declare_log(&mut registry);
    declare_mapping(&mut registry);
    declare_window(&mut registry);

    declare_connection(&mut registry);
    declare_file(&mut registry);
    declare_core(&mut registry);
    return registry;
}
