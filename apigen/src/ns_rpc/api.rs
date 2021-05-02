use proc_macro2::Span;
use quote::quote;
use syn::Ident;

use super::{request::NsRequest, response::NsResponse, NsRpc};
use crate::{rpc_fn::RpcFn, types::SynResult};

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

    pub fn ident_from_ns(ns_name: &Ident) -> Ident {
        Ident::new(format!("{}Api", ns_name).as_str(), Span::call_site())
    }

    fn ident(&self) -> Ident {
        Self::ident_from_ns(&self.ns_name)
    }
}

impl NsApi {
    pub fn to_tokens(&self) -> SynResult {
        let name = self.ident();
        let context = Ident::new("context", Span::call_site());

        let fns: Vec<proc_macro2::TokenStream> = self
            .rpc_fns
            .iter()
            .map(|f| f.to_api_handler_tokens())
            .collect();

        let mut match_arms: Vec<proc_macro2::TokenStream> = vec![];
        for f in &self.rpc_fns {
            match_arms.push(f.to_pattern_dispatch_tokens(&context, &self.ns_name)?);
        }

        let requests_ident = NsRequest::ident_from_ns(&self.ns_name);
        let responses_ident = NsResponse::ident_from_ns(&self.ns_name);

        let gen = quote! {
            struct #name;
            impl #name {
                #(#fns)*
            }

            impl crate::script::api::ApiHandler<
                #requests_ident,
                #responses_ident
            > for #name {
                fn handle(
                    &self,
                    #context: &mut crate::input::commands::CommandHandlerContext,
                    p: #requests_ident
                ) -> crate::input::maps::KeyResult<#responses_ident> {
                    match p {
                        #(#match_arms),*
                    }
                }
            }
        };

        Ok(gen)
    }
}
