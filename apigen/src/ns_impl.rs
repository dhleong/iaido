use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident, Item, Token,
};

use crate::methods::MethodConfig;
use crate::ns_rpc::NsRpc;
use crate::rpc_fn::RpcFn;

pub struct NsImpl {
    name: Ident,
    etc: Vec<Item>,
    rpc_fns: Vec<RpcFn>,
}

impl Parse for NsImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![impl]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut rpc_fns = vec![];
        let mut etc = vec![];

        loop {
            let item_result: Result<Item> = content.parse();
            match item_result {
                Ok(Item::Fn(mut f)) => {
                    let config = MethodConfig::from(f.attrs)?;
                    f.attrs = vec![];

                    if config.is_rpc {
                        rpc_fns.push(RpcFn { item: f, config });
                    } else {
                        etc.push(Item::Fn(f));
                    }
                }

                Ok(item) => etc.push(item),
                _ => break,
            }
        }

        Ok(NsImpl { name, etc, rpc_fns })
    }
}

impl NsImpl {
    pub fn to_tokens(&self) -> Result<TokenStream> {
        let NsImpl { name, etc, rpc_fns } = self;

        let rpc = NsRpc {
            ns_name: name.clone(),
            rpc_fns: rpc_fns.clone(),
        };

        let rpc_delegates = rpc_fns.iter().map(|f| f.to_rpc_tokens());

        Ok(quote! {
            impl #name {
                #(#etc)*
                #(#rpc_delegates)*
            }
            #rpc
        })
    }
}
