use lang::{language, IaidoScriptingLang};
use proc_macro::{self, TokenStream};
use quote::quote;
use syn::parse_macro_input;

mod direct_fn;
mod lang;
mod methods;
mod ns;
mod ns_impl;
mod ns_rpc;
mod rpc_fn;
mod types;

use ns::Ns;
use ns_impl::NsImpl;

#[proc_macro_attribute]
pub fn ns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let language = language();
    let parsed = parse_macro_input!(item as Ns);
    language.wrap_ns(quote! { #parsed }, &parsed).into()
}

#[proc_macro_attribute]
pub fn ns_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let language = language();
    let data = parse_macro_input!(item as NsImpl);
    match data.to_tokens(&language) {
        Ok(tokens) => language.wrap_ns_impl(tokens).into(),
        Err(diag) => diag.to_compile_error().into(),
    }
}
