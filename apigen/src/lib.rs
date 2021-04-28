use lang::language;
use proc_macro::{self, TokenStream};
use syn::parse_macro_input;

mod lang;
mod methods;
mod ns_impl;
mod ns_rpc;
mod rpc_fn;
mod types;

use ns_impl::NsImpl;

#[proc_macro_attribute]
pub fn ns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let language = language();
    language.wrap_ns(item.into()).into()
}

#[proc_macro_attribute]
pub fn ns_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let data = parse_macro_input!(item as NsImpl);
    let language = language();
    match data.to_tokens() {
        Ok(tokens) => language.wrap_ns_impl(tokens).into(),
        Err(diag) => diag.to_compile_error().into(),
    }
}
