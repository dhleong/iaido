use proc_macro::{self, TokenStream};
use quote::quote;
use syn::parse_macro_input;

mod methods;
mod ns_impl;
mod ns_rpc;
mod rpc_fn;
mod types;

use ns_impl::NsImpl;

#[proc_macro]
pub fn declare_ns(_input: TokenStream) -> TokenStream {
    let gen = quote! {};

    gen.into()
}

#[proc_macro_attribute]
pub fn ns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn ns_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let data = parse_macro_input!(item as NsImpl);
    unwrap_tokens(data.to_tokens())
}

fn unwrap_tokens(result: syn::parse::Result<proc_macro2::TokenStream>) -> TokenStream {
    match result {
        Ok(tokens) => tokens.into(),
        Err(diag) => diag.to_compile_error().into(),
    }
}
