use crate::args::{ArgContext, ArgParser};
use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use syn::{
    parse::{Parse, ParseStream},
    PathArguments,
};
use syn::{GenericArgument, Ident, Token, Type};

type SynResult<T> = syn::parse::Result<T>;

#[derive(Clone)]
pub struct CommandArg {
    pub name: Ident,
    span: Span,
    pub type_name: String,
    pub is_optional: bool,
}

impl CommandArg {
    fn new(name: Ident, raw_type: Type) -> SynResult<Self> {
        if let Type::Path(stream) = raw_type {
            let first = &stream.path.segments[0];
            let type_ident = first.ident.clone();
            let type_name = type_ident.to_string();
            if type_name.find("Optional").is_some() {
                match first.arguments {
                    PathArguments::AngleBracketed(ref args) => {
                        if let GenericArgument::Type(Type::Path(ref actual_type)) = args.args[0] {
                            let actual_ident = &actual_type.path.segments[0].ident;
                            return Ok(Self {
                                name,
                                span: actual_ident.span(),
                                type_name: actual_ident.to_string(),
                                is_optional: true,
                            });
                        }
                    }
                    _ => {}
                }
            }

            return Ok(Self {
                name: name.clone(),
                span: type_ident.span(),
                type_name,
                is_optional: false,
            });
        }

        Err(syn::Error::new(name.span(), "Unexpected arg type"))
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn to_tokens(
        &self,
        arg_parser: &ArgParser,
        command_name: Ident,
        args_iter_name: Ident,
    ) -> SynResult<TokenStream> {
        let context = ArgContext {
            command_name,
            args_iter_name,
            arg: self.clone(),
        };
        let block: proc_macro2::TokenStream = arg_parser.parse(context)?.into();

        Ok(block.into())
    }
}

impl Parse for CommandArg {
    fn parse(input: ParseStream) -> SynResult<Self> {
        input.parse::<Token![,]>()?;
        let name: Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let raw_type: Type = input.parse()?;
        return CommandArg::new(name, raw_type);
    }
}
