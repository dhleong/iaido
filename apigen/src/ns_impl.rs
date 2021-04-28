use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    Ident, Item, Token,
};

use crate::rpc_fn::RpcFn;
use crate::{direct_fn::DirectFn, ns_rpc::NsRpc};
use crate::{lang::IaidoScriptingLang, methods::MethodConfig};

pub struct NsImpl {
    name: Ident,
    direct_fns: Vec<DirectFn>,
    rpc_fns: Vec<RpcFn>,
    etc: Vec<Item>,
}

impl Parse for NsImpl {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<Token![impl]>()?;
        let name: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut direct_fns = vec![];
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
                        direct_fns.push(DirectFn { item: f, config });
                    }
                }

                Ok(item) => etc.push(item),
                _ => break,
            }
        }

        Ok(NsImpl {
            name,
            direct_fns,
            etc,
            rpc_fns,
        })
    }
}

impl NsImpl {
    pub fn to_tokens<L: IaidoScriptingLang>(&self, language: &L) -> Result<TokenStream> {
        let NsImpl {
            name,
            direct_fns,
            etc,
            rpc_fns,
        } = self;

        let direct_fn_tokens = direct_fns.iter().map(|f| f.to_tokens(language));
        let rpc_delegates = rpc_fns.iter().map(|f| f.to_rpc_tokens(name, language));

        let rpc = NsRpc {
            ns_name: name.clone(),
            rpc_fns: rpc_fns.clone(),
        };

        let rpc_tokens = rpc.to_tokens()?;

        Ok(quote! {
            impl #name {
                #(#etc)*
                #(#direct_fn_tokens)*
                #(#rpc_delegates)*
            }
            #rpc_tokens
        })
    }
}
