use std::collections::{hash_map, HashMap};

use crate::input::completion::{args::CommandArgsCompleter, Completer};

use super::CommandHandler;

pub struct CommandSpec {
    pub handler: Box<CommandHandler>,
    pub completer: CommandArgsCompleter,
}

impl CommandSpec {
    pub fn handler(handler: Box<CommandHandler>) -> Self {
        Self {
            handler,
            completer: CommandArgsCompleter::new(),
        }
    }

    pub fn push_arg_completer(&mut self, completer: Box<dyn Completer>) {
        self.completer.push(completer);
    }
}

pub struct CommandRegistry {
    commands: HashMap<String, CommandSpec>,
    abbreviations: HashMap<String, String>,
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self {
            commands: HashMap::new(),
            abbreviations: HashMap::new(),
        }
    }
}

impl CommandRegistry {
    pub fn declare(&mut self, name: String, abbreviate: bool, handler: Box<CommandHandler>) {
        if abbreviate {
            for i in 0..&name.len() - 1 {
                self.abbreviations
                    .insert(name[0..i].to_string(), name.clone());
            }
        }

        self.insert(name, CommandSpec::handler(handler));
    }

    pub fn declare_arg(&mut self, name: String, completer: Box<dyn Completer>) {
        let spec = self
            .commands
            .get_mut(&name)
            .expect("Adding arg for not-yet-declared command");
        spec.push_arg_completer(completer);
    }

    pub fn names(&self) -> hash_map::Keys<String, CommandSpec> {
        self.commands.keys()
    }

    pub fn insert(&mut self, name: String, spec: CommandSpec) {
        self.commands.insert(name, spec);
    }

    pub fn get(&self, name: &String) -> Option<&CommandSpec> {
        if let Some(handler) = self.commands.get(name) {
            return Some(handler);
        }

        if let Some(full_name) = self.abbreviations.get(name) {
            if let Some(handler) = self.commands.get(full_name) {
                return Some(handler);
            }
        }

        None
    }

    pub fn take(&mut self, name: &String) -> Option<(String, CommandSpec)> {
        if let Some(handler) = self.commands.remove(name) {
            return Some((name.clone(), handler));
        }

        if let Some(full_name) = self.abbreviations.get(name) {
            if let Some(handler) = self.commands.remove(full_name) {
                return Some((full_name.clone(), handler));
            }
        }

        return None;
    }
}

#[macro_export]
macro_rules! declare_commands {
    ($name:ident { $( $SPEC:tt )* }) => {
        pub fn $name(registry: &mut crate::input::commands::registry::CommandRegistry) {
            command_decl::command_decl! { registry -> $($SPEC)* }
        }
    }
}
