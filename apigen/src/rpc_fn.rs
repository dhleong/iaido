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
}
