use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, AttributeArgs};

mod direct_fn;
mod lang;
mod methods;
mod ns;
mod ns_impl;
mod ns_rpc;
mod rpc_fn;
mod types;

use lang::{language, ConfiguredNsImpl, IaidoScriptingLang};
use ns::Ns;
use ns_impl::{NsImpl, NsImplConfig};
use types::SynResult;

#[proc_macro_attribute]
pub fn ns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ns = parse_macro_input!(item as Ns);
    let language = language();
    language.wrap_ns(quote! { #ns }, &ns).into()
}

#[proc_macro_attribute]
pub fn ns_impl(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttributeArgs);
    let data = parse_macro_input!(item as NsImpl);
    match process_ns_impl(attr, data) {
        Ok(tokens) => tokens.into(),
        Err(diag) => diag.to_compile_error().into(),
    }
}

fn process_ns_impl(attr: AttributeArgs, data: NsImpl) -> SynResult {
    let config = NsImplConfig::from(attr)?;
    let configured = ConfiguredNsImpl::new(data, config);

    let language = language();
    let tokens = configured.to_tokens(&language)?;
    language.wrap_ns_impl(tokens, &configured)
}
