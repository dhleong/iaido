use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

use crate::rpc_fn::RpcFn;

#[derive(Clone)]
struct NsRequest {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl NsRequest {
    fn from(rpc: &NsRpc) -> Self {
        Self {
            ns_name: rpc.ns_name.clone(),
            rpc_fns: rpc.rpc_fns.clone(),
        }
    }

    fn ident(&self) -> Ident {
        Ident::new(
            format!("{}ApiRequest", self.ns_name).as_str(),
            Span::call_site(),
        )
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

#[derive(Clone)]
pub struct NsResponse {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

#[derive(Clone)]
pub struct NsRpc {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl ToTokens for NsRpc {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let requests = NsRequest::from(self);

        let gen = quote! {
            #requests
        };

        tokens.extend(gen);
    }
}
