use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    braced,
    parse::{Parse, ParseStream, Result},
    AttributeArgs, Error, Ident, Item, ItemFn, Meta, NestedMeta, Token,
};

use crate::{direct_fn::DirectFn, ns_rpc::NsRpc};
use crate::{lang::IaidoScriptingLang, methods::MethodConfig};
use crate::{rpc_fn::RpcFn, types::SynResult};

pub struct NsImplConfig {
    pub is_module: bool,
    pub module_name: String,
}

impl NsImplConfig {
    pub fn from(attr: AttributeArgs) -> SynResult<Self> {
        let mut config = Self {
            is_module: false,
            module_name: "iaido".to_string(),
        };
        for meta in attr {
            let is_module = match meta {
                NestedMeta::Meta(Meta::Path(path)) => {
                    path.segments[0].ident.to_string() == "module"
                }
                _ => false,
            };

            if is_module {
                config.is_module = true;
            }
        }
        Ok(config)
    }
}

pub struct NsImpl {
    pub name: Ident,
    pub direct_fns: Vec<DirectFn>,
    pub rpc_fns: Vec<RpcFn>,
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

                    if config.is_property_setter {
                        validate_setter(&f, &mut rpc_fns, &mut direct_fns)?;
                    }

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

        let direct_fn_tokens = map_or_err(direct_fns.iter(), |f| f.to_tokens(language))?;
        let rpc_delegates = map_or_err(rpc_fns.iter(), |f| f.to_rpc_tokens(name, language))?;

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

fn map_or_err<E, Iter, F>(iter: Iter, f: F) -> SynResult<Vec<TokenStream>>
where
    Iter: Iterator<Item = E>,
    F: Fn(E) -> SynResult<TokenStream>,
{
    let mut results = vec![];
    for item in iter {
        results.push(f(item)?);
    }
    Ok(results)
}

fn validate_setter(
    f: &ItemFn,
    rpc_fns: &mut Vec<RpcFn>,
    direct_fns: &mut Vec<DirectFn>,
) -> SynResult<()> {
    let setter_name = f.sig.ident.to_string();
    if !setter_name.starts_with("set_") {
        return Err(Error::new_spanned(
            &f.sig.ident,
            "Property setter should start with set_",
        ));
    }

    let expected = setter_name.replace("set_", "");

    for f in rpc_fns {
        if f.item.sig.ident.to_string() == expected {
            f.config.has_property_setter = true;
            return Ok(());
        }
    }

    for f in direct_fns {
        if f.item.sig.ident.to_string() == expected {
            f.config.has_property_setter = true;
            return Ok(());
        }
    }

    Err(Error::new_spanned(
        f,
        format!(
            "No associated getter #[property] for {}; make sure the getter is declared before this setter",
            f.sig.ident,
        )
    ))
}
