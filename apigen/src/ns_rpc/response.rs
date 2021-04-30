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

    pub fn ident_from_ns(ns_name: &Ident) -> Ident {
        Ident::new(
            format!("{}ApiResponse", ns_name).as_str(),
            Span::call_site(),
        )
    }

    fn ident(&self) -> Ident {
        Self::ident_from_ns(&self.ns_name)
    }
}

impl ToTokens for NsResponse {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.ident();
        let mut responses = vec![];

        for f in &self.rpc_fns {
            let name = f.item.sig.ident.clone();
            match &f.item.sig.output {
                syn::ReturnType::Default => {
                    responses.push(quote! {#name});
                }
                syn::ReturnType::Type(_, ty) => {
                    responses.push(quote! {
                        #name(#ty)
                    });
                }
            };
        }

        let gen = quote! {
            #[allow(non_camel_case_types)]
            #[derive(Clone, Debug)]
            enum #name {
                #(#responses),*
            }
        };

        tokens.extend(gen);
    }
}
