pub mod colors;
pub mod connection;
pub mod core;
pub mod file;
pub mod help;
pub mod log;
pub mod mapping;
pub mod registry;
pub mod script;
pub mod window;

mod helpers;

use std::time::Duration;

use self::{
    colors::declare_colors, connection::declare_connection, core::declare_core, file::declare_file,
    help::declare_help, log::declare_log, mapping::declare_mapping, registry::CommandRegistry,
    script::declare_script, window::declare_window,
};
use crate::delegate_keysource_with_map;

use super::{
    maps::KeyResult, source::memory::MemoryKeySource, BoxableKeymap, Key, KeyError, KeySource,
    KeymapConfig, KeymapContext, KeymapContextWithKeys,
};

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

    pub fn new_blank<T: KeymapContext, K: BoxableKeymap>(
        context: &'a mut T,
        keymap: &'a mut K,
    ) -> Self {
        Self::new(context, keymap, "".to_string())
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
        self.input.split_ascii_whitespace()
    }

    pub fn feed_keys(&mut self, keys: Vec<Key>, config: KeymapConfig) -> KeyResult {
        let source = MemoryKeySource::from(keys);
        let mut context = KeymapContextWithKeys::new(Box::new(&mut self.context), source, config);

        while context.keys.poll_key(Duration::from_millis(0))? {
            self.keymap.process_keys(&mut context)?;
        }

        Ok(())
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
    delegate_keysource_with_map!(context, &mut keymap);
}

pub fn create_builtin_commands() -> CommandRegistry {
    let mut registry = CommandRegistry::default();
    declare_colors(&mut registry);
    declare_log(&mut registry);
    declare_mapping(&mut registry);
    declare_script(&mut registry);
    declare_window(&mut registry);

    declare_connection(&mut registry);
    declare_file(&mut registry);
    declare_core(&mut registry);
    declare_help(&mut registry);
    return registry;
}
