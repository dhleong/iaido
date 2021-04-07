use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Error, Result};
use syn::Ident;

use crate::parse::CommandArg;

type GenerateArgParseFn = dyn Fn(ArgContext) -> TokenStream + Send;

pub struct ArgContext {
    pub command_name: Ident,
    pub arg: CommandArg,
    pub args_iter_name: Ident,
}

pub struct ArgParser {
    map: HashMap<String, Box<GenerateArgParseFn>>,
}

// ======= Arg-parsing helper macro =======================

macro_rules! build_arg_handler {
    ($map:ident ->) => {
        // base case
    };

    ($map:ident -> $type:ty => |$ctx:ident, $raw:ident| $parse:expr, $($tail:tt)*) => {
        $map.insert(stringify!($type).to_string(), Box::new(|$ctx| {
            let arg = $ctx.arg.name.clone();
            let args_iter = $ctx.args_iter_name;
            let parse = $parse;
            let gen = quote! {
                let #arg = if let Some($raw) = #args_iter.next() {
                    Some(#parse)
                } else {
                    None
                };
            };
            gen.into()
        }));
        build_arg_handler! { $map -> $($tail)* };
    };

    ($map:ident -> $type:ty => |$raw:ident| $parse:expr, $($tail:tt)*) => {
        build_arg_handler! { $map ->
            $type => |ctx, $raw| { quote! { $parse } },

            $($tail)*
        };
    };
}

// ======= Arg parsing implementation =====================

fn create_arg_parser() -> ArgParser {
    let mut map: HashMap<String, Box<GenerateArgParseFn>> = HashMap::new();

    build_arg_handler! { map ->
        String => |raw| raw.to_string(),
        PathBuf => |raw| std::path::PathBuf::from(raw),
        usize => |ctx, raw| {
            let command = ctx.command_name;
            let arg = ctx.arg.name.clone();
            quote! {
                match raw.parse::<usize>() {
                    Ok(v) => v,
                    Err(e) => {
                        return Err(
                            crate::input::KeyError::InvalidInput(format!(
                            "{}: argument `{}`: expected integer: {}",
                            stringify!(#command), stringify!(#arg), e
                        )));
                    }
                }
            }
        },
    }

    ArgParser { map }
}

// ======= ArgParser implementation =======================

impl ArgParser {
    pub fn parse(&self, context: ArgContext) -> Result<TokenStream> {
        if let Some(handler) = self.map.get(&context.arg.type_name) {
            let command_name = context.command_name.clone();
            let arg_name = context.arg.name.clone();
            let is_optional = context.arg.is_optional;

            let mut result = handler(context);

            if !is_optional {
                // NOTE: We *could* pre-generate the string for simpler generated
                // code, but I *think* we can share the string across all arg
                // types and reduce binary size this way:
                result.extend(quote! {
                    let #arg_name = if let Some(value) = #arg_name {
                        value
                    } else {
                        return Err(
                            crate::input::KeyError::InvalidInput(format!(
                            "{}: missing required argument `{}`",
                            stringify!(#command_name), stringify!(#arg_name)
                        )));
                    };
                });
            }

            Ok(result)
        } else {
            Err(Error::new(context.arg.span(), "Unsupported arg type"))
        }
    }
}

impl Default for ArgParser {
    fn default() -> Self {
        create_arg_parser()
    }
}
