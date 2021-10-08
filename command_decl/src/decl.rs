use super::args::ArgParser;
use super::completers::CompletionManager;
use super::doc::DocString;
use super::parse::CommandArg;
use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{parenthesized, Attribute, Block, Ident, Token};

struct OneCommandDecl {
    pub attrs: Vec<Attribute>,
    pub name: Ident,
    pub context_ident: Ident,
    pub args: Vec<CommandArg>,
    pub body: Block,
}

impl OneCommandDecl {
    pub fn to_tokens(
        &self,
        help_file_name: &str,
        arg_parser: &ArgParser,
        completions: &CompletionManager,
        registry_name: Ident,
    ) -> Result<TokenStream> {
        let OneCommandDecl {
            attrs,
            name,
            context_ident,
            args,
            body,
            ..
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

        let mut completers: Vec<proc_macro2::TokenStream> = vec![];
        for arg in args {
            let def = completions.declare_completer(arg)?;
            completers.push(
                quote! {
                    #registry_name.declare_arg(
                        stringify!(#name).to_string(),
                        Box::new(#def),
                    );
                }
                .into(),
            );
        }

        let doc: DocString = attrs.into();

        let gen = quote! {
            #registry_name.declare(
                stringify!(#name).to_string(),
                true,
                Box::new(|#context_ident| {
                    #arg_parse
                    #body
                })
            );

            #(#completers)*

            #registry_name.help.insert(
                crate::app::help::HelpTopic {
                    filename: #help_file_name,
                    topic: stringify!(#name),
                    doc: #doc,
                }
            );
        };

        Ok(gen.into())
    }
}

impl Parse for OneCommandDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;

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
            attrs,
            name,
            context_ident,
            args,
            body: input.parse()?,
        })
    }
}

pub struct CommandDecl {
    pub attrs: Vec<Attribute>,
    commands: Vec<OneCommandDecl>,
}

impl Parse for CommandDecl {
    fn parse(input: ParseStream) -> Result<Self> {
        let attrs = input.call(Attribute::parse_inner)?;
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

        Ok(CommandDecl { attrs, commands })
    }
}

impl CommandDecl {
    pub fn to_tokens(&self, module_name: Ident, registry_name: Ident) -> Result<TokenStream> {
        let mut output = TokenStream::new();
        let args = ArgParser::default();
        let completions = CompletionManager::default();
        let help_file_name = module_name.to_string().replace("declare_", "");

        let doc: DocString = (&self.attrs).into();
        let filename_doc: TokenStream = quote! {
            #registry_name.help.insert_filename_doc(#help_file_name, #doc);
        }
        .into();
        output.extend(filename_doc);

        for command in &self.commands {
            output.extend(command.to_tokens(
                &help_file_name,
                &args,
                &completions,
                registry_name.clone(),
            )?);
        }
        Ok(output)
    }
}
