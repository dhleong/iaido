use quote::{quote, ToTokens};
use syn::{Error, GenericArgument, PathArguments, Type};

const COMMAND_HANDLER_CONTEXT: &str = "CommandHandlerContext";

pub type SynResult<T = proc_macro2::TokenStream> = syn::parse::Result<T>;

pub struct SimpleType {
    pub name: String,
    tokens: Type,
    pub is_optional: bool,
}

impl SimpleType {
    pub fn from(type_ref: &Type) -> SynResult<SimpleType> {
        if is_command_context(type_ref) {
            return Ok(SimpleType {
                name: COMMAND_HANDLER_CONTEXT.to_owned(),
                tokens: type_ref.clone(),
                is_optional: false,
            });
        }

        match type_ref {
            Type::Path(stream) => {
                let first = &stream.path.segments[0];
                let type_ident = first.ident.clone();
                let type_name = type_ident.to_string();
                if type_name.find("Optional").is_some() {
                    match first.arguments {
                        PathArguments::AngleBracketed(ref args) => {
                            if let GenericArgument::Type(Type::Path(ref actual_type)) = args.args[0]
                            {
                                let actual_ident = &actual_type.path.segments[0].ident;
                                return Ok(SimpleType {
                                    name: actual_ident.to_string(),
                                    tokens: type_ref.clone(),
                                    is_optional: true,
                                });
                            }
                        }
                        _ => {}
                    }
                }

                Ok(SimpleType {
                    name: type_name,
                    tokens: type_ref.clone(),
                    is_optional: false,
                })
            }

            _ => Err(Error::new_spanned(type_ref, "Unexpected arg type")),
        }
    }
}

impl ToTokens for SimpleType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let type_tokens = &self.tokens;
        tokens.extend(quote! { #type_tokens })
    }
}

pub fn is_command_context(type_ref: &Type) -> bool {
    match type_ref {
        Type::Reference(ty) => match &ty.elem.as_ref() {
            &Type::Path(stream) => {
                let first = &stream.path.segments[0];
                let type_ident = &first.ident;
                type_ident.to_string() == "CommandHandlerContext"
            }

            _ => false,
        },

        _ => false,
    }
}
