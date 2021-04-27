use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, ItemFn};

use crate::ns_rpc::{NsApi, NsRequest, NsResponse};
use crate::types::is_command_context;
use crate::{methods::MethodConfig, types::SynResult};

#[derive(Clone)]
pub struct RpcFn {
    pub item: ItemFn,
    pub config: MethodConfig,
}

impl RpcFn {
    /// Generate the replacement function within the NS that forwards
    /// to, and unpacks the result of, the RPC call
    pub fn to_rpc_tokens(&self) -> TokenStream {
        let ItemFn { sig, .. } = self.item.clone();
        let name = sig.ident.clone();
        let return_type = sig.output.clone();

        quote! {
            fn #name(&self) #return_type {
                panic!("TODO: rpc call");
            }
        }
    }

    /// Generate the API handler function that actually invokes the provided block
    pub fn to_api_handler_tokens(&self) -> TokenStream {
        let ItemFn { sig, block, .. } = self.item.clone();
        let name = sig.ident.clone();
        let return_type = sig.output.clone();
        let params = sig.inputs;

        quote! {
            fn #name(#params) #return_type #block
        }
    }

    /// Generate the pattern-matching API dispatch arm
    pub fn to_pattern_dispatch_tokens(&self, context: &Ident, ns_name: &Ident) -> SynResult {
        let ItemFn { sig, .. } = &self.item;
        let name = sig.ident.clone();
        let params = sig.inputs.clone();
        let api_type = NsApi::ident_from_ns(ns_name);
        let request_type = NsRequest::ident_from_ns(ns_name);
        let response_type = NsResponse::ident_from_ns(ns_name);

        let mut had_context = false;
        let mut context_is_first = false;
        let mut had_self = false;
        let filtered_params: Vec<Ident> = params
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match p {
                FnArg::Receiver(_) => {
                    had_self = true;
                    return None;
                }
                FnArg::Typed(param) => {
                    if is_command_context(&param.ty) {
                        had_context = true;
                        context_is_first = i.to_owned() == 0;
                        return None;
                    }
                    match &param.pat.as_ref() {
                        &syn::Pat::Ident(ident) => Some(ident.ident.clone()),
                        _ => None,
                    }
                }
            })
            .collect();

        if had_self {
            return Err(syn::Error::new(
                name.span(),
                "RPC methods must not accept self",
            ));
        }

        if !had_context || !context_is_first {
            return Err(syn::Error::new(
                name.span(),
                "RPC methods MUST accept a CommandHandlerContext as the first param",
            ));
        }

        let invocation_params = if filtered_params.is_empty() {
            quote! {}
        } else {
            quote! { , #(#filtered_params),* }
        };

        Ok(quote! {
            #request_type::#name => {
                Ok(
                    #response_type::#name(
                        #api_type::#name(#context#invocation_params)
                    )
                )
            }
        })
    }
}
