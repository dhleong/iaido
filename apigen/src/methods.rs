use syn::{
    parse::{Parse, Result},
    Attribute, Expr, Token,
};

mod kw {
    syn::custom_keyword!(rpc);
}

#[derive(Clone)]
pub struct RpcConfig {
    pub rpc_args: Vec<Expr>,
}

impl Parse for RpcConfig {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut rpc_args: Vec<Expr> = vec![];
        loop {
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
                let expr: Expr = input.parse()?;
                rpc_args.push(expr);
            } else {
                break;
            }
        }

        return Ok(RpcConfig { rpc_args });
    }
}

#[derive(Clone)]
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
