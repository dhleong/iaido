use parse::ParseStream;
use quote::ToTokens;
use syn::{
    parse::{self, Parse},
    Error, Ident, Item,
};

pub struct Ns {
    item: Item,
    pub name: Ident,
}

impl Parse for Ns {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let item: Item = input.parse()?;

        let name = match item {
            Item::Struct(ref st) => st.ident.clone(),
            _ => {
                return Err(Error::new(
                    input.span(),
                    "#[apigen::ns] may only be used on a struct",
                ))
            }
        };

        Ok(Ns { item, name })
    }
}

impl ToTokens for Ns {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.item.to_tokens(tokens)
    }
}
