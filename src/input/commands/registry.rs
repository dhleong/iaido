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
macro_rules! command_arg_completer {
    ($r:ident@$name:ident -> PathBuf) => {
        $r.declare_arg(
            stringify!($name).to_string(),
            Box::new(crate::input::completion::file::FileCompleter),
        );
    };

    ($r:ident@$name:ident -> $unsupported:ty) => {};
}

#[macro_export]
macro_rules! command_arg {
    // NOTE: all arg types are parsed as optional; there is a single
    // rule at the end that handles required args

    ($name:ident@$args:ident -> $arg:ident: Optional<PathBuf>) => {
        let $arg = if let Some(raw) = $args.next() {
            Some(std::path::PathBuf::from(raw))
        } else {
            None
        };
    };

    ($name:ident@$args:ident -> $arg:ident: Optional<String>) => {
        let $arg = if let Some(raw) = $args.next() {
            Some(raw.to_string())
        } else {
            None
        };
    };

    ($name:ident@$args:ident -> $arg:ident: Optional<usize>) => {
        let $arg = if let Some(raw) = $args.next() {
            match raw.parse::<usize>() {
                Ok(v) => Some(v),
                Err(e) => {
                    return Err(crate::input::KeyError::InvalidInput(format!(
                        "{}: argument `{}`: expected integer: {}",
                        stringify!($name), stringify!($arg), e
                    )));
                }
            }
        } else {
            None
        };
    };

    // non-optional args:
    ($name:ident@$args:ident -> $arg:ident: $type:ident) => {
        crate::command_arg!($name@$args -> $arg: Optional<$type>);
        let $arg = if let Some(value) = $arg {
            value
        } else {
            return Err(crate::input::KeyError::InvalidInput(format!(
                "{}: missing required argument `{}`",
                stringify!($name), stringify!($arg)
            )));
        };
    };
}

#[macro_export]
macro_rules! command_decl {
    // base case:
    ($r:ident ->) => {
        // as elsewhere, this import makes things work easier, but
        // breaks completion for now
        // #[allow(unused_imports)]
        // use crate::input::KeymapContext;
    };

    // simple case: no special args handling
    ($r:ident -> pub fn $name:ident($context:ident) $body:expr, $($tail:tt)*) => {
        $r.declare(
            stringify!($name).to_string(),
            true,
            Box::new(|$context| $body),
        );
        crate::command_decl! { $r -> $($tail)* }
    };

    // 1 or more args
    ($r:ident -> pub fn $name:ident($context:ident, $($arg:ident: $($type:tt)+),+) $body:expr, $($tail:tt)*) => {
        crate::command_decl! { $r ->
            pub fn $name($context) {
                let args_vec = $context.args();
                let mut args = args_vec.iter();
                $(crate::command_arg!($name@args -> $arg: $($type)+)),+;

                $body
            },
            $($tail)*
        }
        $(crate::command_arg_completer!($r@$name -> $($type)+)),+;
    };
}

#[macro_export]
macro_rules! declare_commands {
    ($name:ident { $( $SPEC:tt )* }) => {
        pub fn $name(registry: &mut crate::input::commands::registry::CommandRegistry) {
            crate::command_decl! { registry -> $($SPEC)* }
        }
    }
}
