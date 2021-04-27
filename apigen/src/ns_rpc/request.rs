use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{FnArg, Ident, Type};

use super::NsRpc;
use crate::{rpc_fn::RpcFn, types::is_command_context};

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

    fn extract_arg_types<'a, I>(inputs: I) -> Vec<Type>
    where
        I: Iterator<Item = &'a FnArg>,
    {
        inputs
            .filter_map(|p| match p {
                FnArg::Receiver(_) => None,
                FnArg::Typed(param) => {
                    if is_command_context(&param.ty) {
                        None
                    } else {
                        Some(param.ty.as_ref().clone())
                    }
                }
            })
            .collect()
    }
}

impl ToTokens for NsRequest {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let name = self.ident();
        let mut requests = vec![];

        for f in &self.rpc_fns {
            let name = f.item.sig.ident.clone();
            let args = Self::extract_arg_types(f.item.sig.inputs.iter());
            requests.push(if args.is_empty() {
                quote! { #name }
            } else {
                quote! { #name(#(#args),*) }
            });
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
