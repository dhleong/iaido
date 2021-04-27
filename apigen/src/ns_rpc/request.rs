use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

use super::NsRpc;
use crate::rpc_fn::RpcFn;

#[derive(Clone)]
pub struct NsRequest {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl NsRequest {
    pub fn from(rpc: &NsRpc) -> Self {
        Self {
            ns_name: rpc.ns_name.clone(),
            rpc_fns: rpc.rpc_fns.clone(),
        }
    }

    pub fn ident_from_ns(ns_name: &Ident) -> Ident {
        Ident::new(format!("{}ApiRequest", ns_name).as_str(), Span::call_site())
    }

    fn ident(&self) -> Ident {
        Self::ident_from_ns(&self.ns_name)
    }
}

impl ToTokens for NsRequest {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.ident();
        let mut requests = vec![];

        for f in &self.rpc_fns {
            // TODO: request args
            let name = f.item.sig.ident.clone();
            requests.push(name);
        }

        let gen = quote! {
            #[allow(non_camel_case_types)]
            enum #name {
                #(#requests),*
            }
        };

        tokens.extend(gen);
    }
}
