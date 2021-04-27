use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

use super::NsRpc;
use crate::rpc_fn::RpcFn;

#[derive(Clone)]
pub struct NsResponse {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl NsResponse {
    pub fn from(rpc: &NsRpc) -> Self {
        Self {
            ns_name: rpc.ns_name.clone(),
            rpc_fns: rpc.rpc_fns.clone(),
        }
    }

    fn ident(&self) -> Ident {
        Ident::new(
            format!("{}ApiResponse", self.ns_name).as_str(),
            Span::call_site(),
        )
    }
}

impl ToTokens for NsResponse {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.ident();
        let mut responses = vec![];

        for f in &self.rpc_fns {
            // TODO: response args
            let name = f.item.sig.ident.clone();
            responses.push(name);
        }

        let gen = quote! {
            #[allow(non_camel_case_types)]
            enum #name {
                #(#responses),*
            }
        };

        tokens.extend(gen);
    }
}
