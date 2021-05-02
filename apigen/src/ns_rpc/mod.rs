use quote::quote;
use syn::Ident;

use crate::{rpc_fn::RpcFn, types::SynResult};

mod api;
mod request;
mod response;

pub use api::NsApi;
pub use request::NsRequest;
pub use response::NsResponse;

#[derive(Clone)]
pub struct NsRpc {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl NsRpc {
    pub fn to_tokens(&self) -> SynResult {
        if self.rpc_fns.is_empty() {
            // unlikely, but... y'know
            return Ok(quote! {});
        }

        let requests = NsRequest::from(self);
        let responses = NsResponse::from(self);
        let api = NsApi::from(self);

        let mut gen = quote! {
            #requests
            #responses
        };

        gen.extend(api.to_tokens()?);

        Ok(gen)
    }
}
