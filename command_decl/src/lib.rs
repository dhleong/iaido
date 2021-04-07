mod args;

use args::{ArgContext, ArgParser};
use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use proc_macro2_diagnostics::{Diagnostic, SpanDiagnosticExt};
use quote::quote;
use syn::{parenthesized, parse_macro_input, Block, GenericArgument, Ident, Token, Type};
use syn::{
    parse::{Parse, ParseStream},
    PathArguments,
};

type TokensResult = Result<TokenStream, Diagnostic>;
type SynResult<T> = syn::parse::Result<T>;

struct CommandArg {
    pub name: Ident,
    pub kind: Type,
}

impl CommandArg {
    // TODO
    pub fn to_tokens(
        &self,
        arg_parser: &ArgParser,
        command: Ident,
        args_ident: Ident,
    ) -> TokensResult {
        let CommandArg { name, kind } = self;
        let context = self.context(command, args_ident)?;
        let block: proc_macro2::TokenStream = arg_parser.parse(context)?.into();

        let gen = quote! {
            // $crate::args::command_arg { #command@#args_ident -> #name: #kind };
            let #name: #kind = #block;
        };

        Ok(gen.into())
    }

    fn context(
        &self,
        command_name: Ident,
        args_iter_name: Ident,
    ) -> Result<ArgContext, Diagnostic> {
        let CommandArg { name, kind } = self;

        if let Type::Path(stream) = kind {
            let first = &stream.path.segments[0];
            let type_name = first.ident.clone();
            if type_name.to_string().find("Optional").is_some() {
                match first.arguments {
                    PathArguments::AngleBracketed(ref args) => {
                        if let GenericArgument::Type(Type::Path(ref actual_type)) = args.args[0] {
                            return Ok(ArgContext {
                                command_name,
                                args_iter_name,
                                arg_name: name.clone(),
                                arg_kind: actual_type.path.segments[0].ident.to_string(),
                                is_optional: true,
                            });
                        }
                    }
                    _ => {}
                }
            }

            return Ok(ArgContext {
                command_name,
                args_iter_name,
                arg_name: name.clone(),
                arg_kind: type_name.to_string(),
                is_optional: false,
            });
        }

        Err(name.span().error("Unexpected arg type"))
    }
}

impl Parse for CommandArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.parse::<Token![,]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let kind: Type = input.parse()?;
        return Ok(CommandArg { name, kind });
    }
}

struct OneCommandDecl {
    pub name: Ident,
    pub context_ident: Ident,
    pub args: Vec<CommandArg>,
    pub body: Block,
}

impl OneCommandDecl {
    pub fn to_tokens(&self, arg_parser: &ArgParser, registry_name: Ident) -> TokensResult {
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
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.parse::<Token![pub]>()?;
        input.parse::<Token![fn]>()?;
        let name: Ident = input.parse()?;

        let parens;
        parenthesized!(parens in input);
        let context_ident: Ident = parens.parse()?;

        let mut args = vec![];
        loop {
            let arg_result: SynResult<CommandArg> = parens.parse();
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
    fn parse(input: ParseStream) -> SynResult<Self> {
        let registry_name: Ident = input.parse()?;
        input.parse::<Token![-]>()?;
        input.parse::<Token![>]>()?;

        let mut commands: Vec<OneCommandDecl> = vec![];

        loop {
            // consume an optional comma
            let _ = input.parse::<Token![,]>();

            let command: SynResult<OneCommandDecl> = input.parse();
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

fn impl_command_decl(decl: &CommandDecl) -> TokensResult {
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
        Err(diag) => diag.emit_as_expr_tokens().into(),
    }
}
