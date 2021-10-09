use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Result;

use crate::parse::CommandArg;

type GenerateArgCompletionFn = dyn Fn(&CommandArg) -> TokenStream + Send;

pub struct CompletionManager {
    map: HashMap<String, Box<GenerateArgCompletionFn>>,
}

// ======= Completion-defining helper macro ===============

macro_rules! build_completion_manager {
    ($map:ident ->) => {
        // base case
    };

    ($map:ident -> $type:ty => $completer:expr, $($tail:tt)*) => {
        $map.insert(stringify!($type).to_string(), Box::new(|_ctx| {
            quote! {
                $completer
            }
        }));

        build_completion_manager! { $map -> $($tail)* };
    };

    ($map:ident -> $type:ty => |$ctx:ident| $completer:expr, $($tail:tt)*) => {
        $map.insert(stringify!($type).to_string(), Box::new(|$ctx| {
            quote! {
                $completer
            }
        }));

        build_completion_manager! { $map -> $($tail)* };
    };
}

// ======= Completion definition ==========================

fn create_completion_manager() -> CompletionManager {
    let mut map: HashMap<String, Box<GenerateArgCompletionFn>> = HashMap::new();

    build_completion_manager! { map ->
        PathBuf => crate::input::completion::file::FileCompleter,
        HelpQuery => crate::input::completion::help::HelpTopicCompleter,
    };

    CompletionManager { map }
}

// ======= CompletionManager implementation ===============

impl CompletionManager {
    pub fn declare_completer(&self, arg: &CommandArg) -> Result<TokenStream> {
        if let Some(factory) = self.map.get(&arg.type_name) {
            Ok(factory(arg))
        } else {
            Ok(quote! {
                crate::input::completion::empty::EmptyCompleter
            })
        }
    }
}

impl Default for CompletionManager {
    fn default() -> Self {
        create_completion_manager()
    }
}
