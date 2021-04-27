use syn::{
    parenthesized,
    parse::{Parse, Result},
    punctuated::Punctuated,
    Attribute, Expr, Token,
};

mod kw {
    syn::custom_keyword!(passing);
}

#[derive(Clone, Debug)]
pub struct RpcConfig {
    pub rpc_args: Vec<Expr>,
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
pub struct MethodConfig {
    pub is_property: bool,
    pub is_rpc: bool,
    pub rpc_config: Option<RpcConfig>,
}

impl MethodConfig {
    pub fn from(attrs: Vec<Attribute>) -> Result<Self> {
        let mut new = Self {
            is_property: false,
            is_rpc: false,
            rpc_config: None,
        };

        for attr in attrs {
            let name = attr.path.segments.last().unwrap().ident.to_string();
            match name.as_str() {
                "rpc" => {
                    new.is_rpc = true;
                    if let Ok(config) = attr.parse_args::<RpcConfig>() {
                        new.rpc_config = Some(config);
                    }
                }
                "property" => {
                    new.is_property = true;
                }
                _ => {} // ignore
            }
        }

        Ok(new)
    }
}
