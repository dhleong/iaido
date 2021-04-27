use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::Ident;

use super::{request::NsRequest, response::NsResponse, NsRpc};
use crate::rpc_fn::RpcFn;

#[derive(Clone)]
pub struct NsApi {
    pub ns_name: Ident,
    pub rpc_fns: Vec<RpcFn>,
}

impl NsApi {
    pub fn from(rpc: &NsRpc) -> Self {
        Self {
            ns_name: rpc.ns_name.clone(),
            rpc_fns: rpc.rpc_fns.clone(),
        }
    }

    fn ident(&self) -> Ident {
        Ident::new(format!("{}Api", self.ns_name).as_str(), Span::call_site())
    }
}

impl ToTokens for NsApi {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.ident();
        let fns: Vec<proc_macro2::TokenStream> = self
            .rpc_fns
            .iter()
            .map(|f| f.to_api_handler_tokens())
            .collect();

        let requests_ident = NsRequest::ident_from_ns(&self.ns_name);
        let responses_ident = NsResponse::ident_from_ns(&self.ns_name);

        let gen = quote! {
            struct #name;
            impl #name {
                #(#fns)*
            }

            impl #name {
                fn handle(
                    &self,
                    context: &mut crate::input::commands::CommandHandlerContext,
                    p: #requests_ident
                ) -> crate::input::maps::KeyResult<#responses_ident> {
                    panic!("TODO")
                }
            }
        };

        tokens.extend(gen);
    }
}
