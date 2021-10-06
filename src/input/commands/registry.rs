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

#[derive(Default)]
pub struct CommandRegistry {
    commands: HashMap<String, CommandSpec>,
    docs: HashMap<String, &'static str>,
    abbreviations: HashMap<String, String>,
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

    pub fn declare_doc(&mut self, name: String, doc: &'static str) {
        self.docs.insert(name, doc);
    }

    pub fn names(&self) -> hash_map::Keys<String, CommandSpec> {
        self.commands.keys()
    }

    pub fn insert(&mut self, name: String, spec: CommandSpec) {
        self.commands.insert(name, spec);
    }

    pub fn expand_name<'a>(&'a self, name: &'a String) -> Option<&'a String> {
        if self.commands.get(name).is_some() {
            return Some(name);
        }

        if let Some(full_name) = self.abbreviations.get(name) {
            if self.commands.get(full_name).is_some() {
                return Some(full_name);
            }
        }

        None
    }

    pub fn get(&self, name: &String) -> Option<&CommandSpec> {
        if let Some(expanded) = self.expand_name(name) {
            return self.commands.get(expanded);
        }

        None
    }

    pub fn get_doc(&self, name: &String) -> Option<&&str> {
        if let Some(expanded) = self.expand_name(name) {
            return self.docs.get(expanded);
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
