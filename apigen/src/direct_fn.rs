use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::{lang::IaidoScriptingLang, methods::MethodConfig, types::SynResult};

/// A DirectFn is one which is invoked directly, as opposed to
/// RpcFn, which must perform an RPC call
pub struct DirectFn {
    pub item: ItemFn,
    pub config: MethodConfig,
}

impl DirectFn {
    pub fn to_tokens<L: IaidoScriptingLang>(&self, language: &L) -> SynResult<TokenStream> {
        let DirectFn { item, config } = self;
        let tokens = quote! {
            #item
        };
        language.wrap_fn(tokens, &item, config)
    }
}
