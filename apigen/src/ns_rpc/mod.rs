use quote::{quote, ToTokens};
use syn::Ident;

use crate::{
    ns_rpc::{api::NsApi, request::NsRequest, response::NsResponse},
    rpc_fn::RpcFn,
};

mod api;
mod request;
mod response;

#[derive(Clone)]
pub struct NsRpc {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl ToTokens for NsRpc {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.rpc_fns.is_empty() {
            // unlikely, but... y'know
            return;
        }

        let requests = NsRequest::from(self);
        let responses = NsResponse::from(self);
        let api = NsApi::from(self);

        let gen = quote! {
            #requests
            #responses
            #api
        };

        tokens.extend(gen);
    }
}
