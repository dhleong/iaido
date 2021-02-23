use std::collections::{hash_map, HashMap};

use super::CommandHandler;

pub struct CommandRegistry {
    commands: HashMap<String, Box<CommandHandler>>,
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

        self.commands.insert(name, handler);
    }

    pub fn names(&self) -> hash_map::Keys<String, Box<CommandHandler>> {
        self.commands.keys()
    }

    pub fn take(&mut self, name: &String) -> Option<(String, Box<CommandHandler>)> {
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

    // optional string arg
    ($r:ident -> pub fn $name:ident($context:ident, $arg:ident: Optional<String>) $body:expr, $($tail:tt)*) => {
        crate::command_decl! { $r ->
            pub fn $name($context) {
                let args = $context.args();
                let $arg = if args.len() < 1 {
                    None
                } else {
                    Some(args[0].to_string())
                };
                $body
            },
            $($tail)*
        }
    };

    // required string arg
    ($r:ident -> pub fn $name:ident($context:ident, $arg:ident: String) $body:expr, $($tail:tt)*) => {
        crate::command_decl! { $r ->
            pub fn $name($context, optional_arg: Optional<String>) {
                let $arg = if let Some(v) = optional_arg {
                    v
                } else {
                    return Err(crate::input::KeyError::InvalidInput(
                        format!(
                            "{}: requires 1 argument ({})",
                            stringify!($name), stringify!($arg)
                        )
                    ));
                };
                $body
            },
            $($tail)*
        }
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
