use syn::{
    parse::{Parse, Result},
    Attribute, Expr, Token,
};

mod kw {
    syn::custom_keyword!(rpc);
}

#[derive(Clone)]
pub struct PropertyConfig {
    pub is_rpc: bool,
    pub rpc_args: Vec<Expr>,
}

impl Parse for PropertyConfig {
    fn parse(input: syn::parse::ParseStream) -> Result<Self> {
        let mut is_rpc = false;
        if input.peek(kw::rpc) {
            input.parse::<kw::rpc>()?;
            is_rpc = true;
        }

        let mut rpc_args: Vec<Expr> = vec![];
        if is_rpc {
            loop {
                if input.peek(Token![,]) {
                    input.parse::<Token![,]>()?;
                    let expr: Expr = input.parse()?;
                    rpc_args.push(expr);
                } else {
                    break;
                }
            }
        }

        return Ok(PropertyConfig { is_rpc, rpc_args });
    }
}

#[derive(Clone)]
pub struct MethodConfig {
    pub is_property: bool,
    pub is_rpc: bool,
    pub property_config: Option<PropertyConfig>,
}

impl MethodConfig {
    pub fn from(attrs: Vec<Attribute>) -> Result<Self> {
        let mut new = Self {
            is_property: false,
            is_rpc: false,
            property_config: None,
        };

        for attr in attrs {
            let name = attr.path.segments.last().unwrap().ident.to_string();
            match name.as_str() {
                "rpc" => new.is_rpc = true,
                "property" => {
                    new.is_property = true;
                    if let Ok(config) = attr.parse_args::<PropertyConfig>() {
                        new.property_config = Some(config);
                    }
                }
                _ => {} // ignore
            }
        }

        Ok(new)
    }
}
