use proc_macro::{self, TokenStream};
use quote::quote;

#[proc_macro]
pub fn declare_ns(_input: TokenStream) -> TokenStream {
    let gen = quote! {};

    gen.into()
    // let data = parse_macro_input!(input as DeclareCommands);
    // match data.to_tokens() {
    //     Ok(tokens) => tokens,
    //     Err(diag) => diag.to_compile_error().into(),
    // }
}

#[proc_macro_attribute]
pub fn ns(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn ns_impl(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
