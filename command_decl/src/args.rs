use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Error, Result};
use syn::Ident;

pub struct ArgContext {
    pub command_name: Ident,
    pub arg_name: Ident,
    pub arg_kind: String,
    pub args_iter_name: Ident,
    pub is_optional: bool,
}

type GenerateArgParseFn = dyn Fn(ArgContext) -> TokenStream + Send;

pub struct ArgParser {
    map: HashMap<String, Box<GenerateArgParseFn>>,
}

macro_rules! build_arg_handler {
    ($map:ident ->) => {
        // base case
    };

    ($map:ident -> $type:ty => |$ctx:ident| $parse:expr, $($tail:tt)*) => {
        $map.insert(stringify!($type).to_string(), Box::new(|$ctx| {
            $parse.into()
        }));
        build_arg_handler! { $map -> $($tail)* };
    };
}

fn create_arg_parser() -> ArgParser {
    let mut map: HashMap<String, Box<GenerateArgParseFn>> = HashMap::new();

    build_arg_handler! { map ->
        String => |ctx| {
            let args_iter_name = ctx.args_iter_name;
            quote! {
                if let Some(raw) = #args_iter_name.next() {
                    Some(raw.to_string())
                } else {
                    None
                }
            }
        },
    }

    ArgParser { map }
}

impl ArgParser {
    pub fn parse(&self, context: ArgContext) -> Result<TokenStream> {
        if let Some(handler) = self.map.get(&context.arg_kind) {
            let command_name = context.command_name.clone();
            let arg_name = context.arg_name.clone();
            let is_optional = context.is_optional;

            let mut result = handler(context);

            if is_optional {
                result.extend(quote! {
                    let #arg_name = if let Some(value) = #arg_name {
                        value
                    } else {
                        return Err(
                            crate::input::KeyError::InvalidInput(format!(
                            "{}: missing required argument `{}`",
                            stringify!(#command_name), stringify!(#arg_name)
                        )));
                    }
                });
            }

            Ok(result)
        } else {
            Err(Error::new(context.arg_name.span(), "Unsupported arg type"))
        }
    }
}

impl Default for ArgParser {
    fn default() -> Self {
        create_arg_parser()
    }
}
