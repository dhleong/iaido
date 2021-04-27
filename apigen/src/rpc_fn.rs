use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemFn;

use crate::methods::MethodConfig;

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
}
