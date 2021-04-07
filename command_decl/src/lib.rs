mod args;
mod parse;

use args::ArgParser;
use parse::CommandArg;
use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parenthesized, parse_macro_input, Block, Ident, Token};

struct OneCommandDecl {
    pub name: Ident,
    pub context_ident: Ident,
    pub args: Vec<CommandArg>,
    pub body: Block,
}

impl OneCommandDecl {
    pub fn to_tokens(&self, arg_parser: &ArgParser, registry_name: Ident) -> Result<TokenStream> {
        let OneCommandDecl {
            name,
            context_ident,
            args,
            body,
        } = self;

        let arg_parse = if args.is_empty() {
            quote! {}
        } else {
            let mut gen = quote! {
                let args_vec = #context_ident.args();
                let mut args = args_vec.iter();
            };

            let args_ident = Ident::new("args", Span::call_site());
            for arg in args {
                let stmt = arg.to_tokens(arg_parser, name.clone(), args_ident.clone())?;
                let tokens: proc_macro2::TokenStream = stmt.into();
                gen.extend(tokens);
            }

            gen
        };

        let gen = quote! {
            #registry_name.declare(
                stringify!(#name).to_string(),
                true,
                Box::new(|#context_ident| {
                    #arg_parse
                    #body
                })
            );
        };

        Ok(gen.into())
    }
}

impl Parse for OneCommandDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![pub]>()?;
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let parens;
        parenthesized!(parens in input);
        let context_ident: Ident = parens.parse()?;

        let mut args = vec![];
        loop {
            let arg_result: Result<CommandArg> = parens.parse();
            if let Ok(arg) = arg_result {
                args.push(arg);
            } else {
                break;
            }
        }

        Ok(OneCommandDecl {
            name,
            context_ident,
            args,
            body: input.parse()?,
        })
    }
}

struct CommandDecl {
    pub registry_name: Ident,
    pub commands: Vec<OneCommandDecl>,
}

impl Parse for CommandDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        let registry_name: Ident = input.parse()?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![>]>()?;

        let mut commands: Vec<OneCommandDecl> = vec![];

        loop {
            // consume an optional comma
            let _ = input.parse::<Token![,]>();

            let command: Result<OneCommandDecl> = input.parse();
            if let Ok(command) = command {
                commands.push(command);
            } else {
                break;
            }
        }

        Ok(CommandDecl {
            registry_name,
            commands,
        })
    }
}

fn impl_command_decl(decl: &CommandDecl) -> Result<TokenStream> {
    let mut output = TokenStream::new();
    let args = ArgParser::default();
    for command in &decl.commands {
        output.extend(command.to_tokens(&args, decl.registry_name.clone())?);
    }
    Ok(output)
}

#[proc_macro]
pub fn command_decl(input: TokenStream) -> TokenStream {
    let decl = parse_macro_input!(input as CommandDecl);
    match impl_command_decl(&decl) {
        Ok(tokens) => tokens.into(),
        Err(diag) => diag.to_compile_error().into(),
    }
}
