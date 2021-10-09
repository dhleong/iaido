use std::fmt::Display;

use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Attribute;

pub struct DocString {
    content: String,
}

impl Display for DocString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.content)
    }
}

impl From<&Vec<Attribute>> for DocString {
    fn from(attrs: &Vec<Attribute>) -> Self {
        let content = attrs
            .iter()
            .filter_map(|attr| match attr.path.segments.first() {
                Some(ident) if ident.ident.to_string() == "doc" => Some(attr.tokens.to_string()),
                _ => None,
            })
            .fold(String::new(), |mut a, b| {
                if !a.is_empty() {
                    a.push_str("\n")
                }
                if !b.is_empty() {
                    let start = b.find("\"").unwrap_or(0) + 1; // Skip the "
                    let end = b.len() - 1; // Drop the trailing "
                    a.push_str(&b[start..end]);
                }
                a
            });
        Self { content }
    }
}

impl ToTokens for DocString {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.content.to_tokens(tokens)
    }
}
