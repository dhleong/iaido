use std::fmt;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    parenthesized,
    parse::{Parse, Result},
    punctuated::Punctuated,
    Attribute, Expr, Token,
};

mod kw {
    syn::custom_keyword!(passing);
    syn::custom_keyword!(setter);
}

#[derive(Clone)]
pub struct RpcConfig {
    pub rpc_args: Vec<Expr>,
}

impl fmt::Debug for RpcConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tokens = TokenStream::new();
        for arg in &self.rpc_args {
            arg.to_tokens(&mut tokens);
        }
        write!(f, "Rpc({})", tokens)
    }
}

impl Parse for RpcConfig {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut rpc_args: Vec<Expr> = vec![];
        if input.peek(kw::passing) {
            input.parse::<kw::passing>()?;

            let args;
            parenthesized!(args in input);

            let terminated: Punctuated<Expr, Token![,]> = args.parse_terminated(Expr::parse)?;
            rpc_args = terminated.iter().map(|expr| expr.clone()).collect();
        }

        return Ok(RpcConfig { rpc_args });
    }
}

#[derive(Clone, Debug)]
pub struct PropertyConfig {
    pub is_setter: bool,
}

impl Parse for PropertyConfig {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut config = Self { is_setter: false };
        if input.peek(kw::setter) {
            input.parse::<kw::setter>()?;
            config.is_setter = true;
        }

        Ok(config)
    }
}

#[derive(Clone, Debug)]
pub struct MethodConfig {
    pub is_method: bool,
    pub is_property: bool,
    pub is_property_setter: bool,
    pub is_rpc: bool,
    pub has_property_setter: bool,
    pub rpc_config: Option<RpcConfig>,
}

impl MethodConfig {
    pub fn from(attrs: Vec<Attribute>) -> Result<Self> {
        let mut new = Self {
            is_method: false,
            is_property: false,
            is_property_setter: false,
            is_rpc: false,
            has_property_setter: false,
            rpc_config: None,
        };

        for attr in attrs {
            let name = attr.path.segments.last().unwrap().ident.to_string();
            match name.as_str() {
                "method" => {
                    new.is_method = true;
                }
                "rpc" => {
                    new.is_rpc = true;
                    if let Ok(config) = attr.parse_args::<RpcConfig>() {
                        new.rpc_config = Some(config);
                    }
                }
                "property" => {
                    new.is_property = true;
                    if let Ok(config) = attr.parse_args::<PropertyConfig>() {
                        new.is_property_setter = config.is_setter;
                    }
                }
                _ => {} // ignore
            }
        }

        Ok(new)
    }
}
