mod args;
mod completers;
mod decl;
mod doc;
mod parse;

use proc_macro::{self, TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::parse::{Parse, ParseStream, Result};
use syn::{braced, parse_macro_input, Ident};

use decl::CommandDecl;

struct DeclareCommands {
    pub module_name: Ident,
    pub decls: CommandDecl,
}

impl DeclareCommands {
    pub fn to_tokens(&self) -> Result<TokenStream> {
        let module_name = &self.module_name;
        let registry_name = Ident::new("registry", Span::call_site());
        let body = self
            .decls
            .to_tokens(module_name.clone(), registry_name.clone())?;
        let body_tokens: proc_macro2::TokenStream = body.into();

        let gen = quote! {
            pub fn #module_name(#registry_name: &mut crate::input::commands::registry::CommandRegistry) {
                #body_tokens
            }
        };

        Ok(gen.into())
    }
}

impl Parse for DeclareCommands {
    fn parse(input: ParseStream) -> Result<Self> {
        let module_name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let decls: CommandDecl = content.parse()?;
        Ok(DeclareCommands { module_name, decls })
    }
}

#[proc_macro]
pub fn declare_commands(input: TokenStream) -> TokenStream {
    let data = parse_macro_input!(input as DeclareCommands);
    match data.to_tokens() {
        Ok(tokens) => tokens,
        Err(diag) => diag.to_compile_error().into(),
    }
}
