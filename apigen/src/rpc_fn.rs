use proc_macro2::TokenStream;
use quote::quote;
use syn::{Expr, ExprPath, FnArg, Ident, ItemFn, Path};

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
    pub fn to_rpc_tokens(&self, ns_name: &Ident) -> TokenStream {
        let ItemFn { sig, .. } = self.item.clone();
        let name = sig.ident.clone();
        let return_type = sig.output.clone();
        let api_type = NsApi::ident_from_ns(ns_name);
        let requests_type = NsRequest::ident_from_ns(ns_name);
        let responses_type = NsResponse::ident_from_ns(ns_name);
        let request_params = self.perform_request_param_names();
        let request_params_tokens = if request_params.is_empty() {
            quote! {}
        } else {
            quote! { (#(#request_params),*) }
        };

        quote! {
            fn #name(&self) #return_type {
                match self.api.perform(
                    #api_type,
                    #requests_type::#name#request_params_tokens
                ) {
                    Ok(#responses_type::#name(response)) => response,
                    Ok(unexpected) => panic!("Unexpected response: {:?}", unexpected),
                    Err(e) => std::panic::panic_any(e),
                }
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
        let api_type = NsApi::ident_from_ns(ns_name);
        let request_type = NsRequest::ident_from_ns(ns_name);
        let response_type = NsResponse::ident_from_ns(ns_name);

        let filtered_params = match self.unpack_request_param_names() {
            Ok(params) => params,
            Err(e) => return Err(e),
        };

        let unpack_params = if filtered_params.is_empty() {
            quote! {}
        } else {
            quote! { (#(#filtered_params),*) }
        };

        let invocation_params = if filtered_params.is_empty() {
            quote! {}
        } else {
            quote! { , #(#filtered_params),* }
        };

        Ok(quote! {
            #request_type::#name#unpack_params => {
                Ok(
                    #response_type::#name(
                        #api_type::#name(#context#invocation_params)
                    )
                )
            }
        })
    }

    fn perform_request_param_names(&self) -> Vec<Expr> {
        let params = &self.item.sig.inputs;
        params
            .iter()
            .enumerate()
            .filter_map(|(i, p)| match p {
                FnArg::Typed(param) => {
                    if is_command_context(&param.ty) {
                        return None;
                    }

                    if let Some(config) = &self.config.rpc_config {
                        if i - 1 < config.rpc_args.len() {
                            return Some(config.rpc_args[i - 1].clone());
                        }
                        panic!("well hell: {:?}", config);
                    }

                    match &param.pat.as_ref() {
                        &syn::Pat::Ident(ident) => Some(Expr::Path(ExprPath {
                            attrs: vec![],
                            qself: None,
                            path: Path::from(ident.ident.clone()),
                        })),
                        _ => None,
                    }
                }
                _ => None,
            })
            .collect()
    }

    fn unpack_request_param_names(&self) -> SynResult<Vec<Ident>> {
        let ItemFn { sig, .. } = &self.item;
        let name = sig.ident.clone();
        let params = sig.inputs.clone();

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

        Ok(filtered_params)
    }
}
