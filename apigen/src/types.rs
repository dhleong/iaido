use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{
    Error, GenericArgument, PathArguments, ReturnType, Signature, Type, TypePath, TypeReference,
};

const COMMAND_HANDLER_CONTEXT: &str = "CommandHandlerContext";

pub type SynResult<T = proc_macro2::TokenStream> = syn::parse::Result<T>;

pub fn is_command_context(type_ref: &Type) -> bool {
    if let Ok(simple) = SimpleType::from(type_ref) {
        simple.is_ref && simple.name == COMMAND_HANDLER_CONTEXT
    } else {
        false
    }
}

/// Returns a Some with a TokenStream representing the result type of the
/// given signature, if it returns some sort of KeyResult, else None
#[allow(dead_code)]
pub fn result_type(signature: &Signature) -> Option<TokenStream> {
    if SimpleType::from_return(&signature.output)
        .and_then(|ty| Some(ty.name))
        .unwrap_or("".to_string())
        == "KeyResult"
    {
        // TODO: property extract the result type, if an explicit one is given
        Some(quote! { () })
    } else {
        None
    }
}

pub struct SimpleType {
    pub name: String,
    tokens: Type,
    pub is_optional: bool,
    pub is_ref: bool,
}

impl SimpleType {
    #[allow(dead_code)]
    pub fn from_return(output: &ReturnType) -> Option<SimpleType> {
        match output {
            ReturnType::Default => None,
            ReturnType::Type(_, ty) => Self::from(&ty.as_ref()).ok(),
        }
    }

    pub fn from(type_ref: &Type) -> SynResult<SimpleType> {
        match type_ref {
            Type::Path(stream) => Self::from_path(type_ref, stream),
            Type::Reference(TypeReference { elem, .. }) => {
                let base = Self::from(elem)?;
                Ok(Self {
                    is_ref: true,
                    ..base
                })
            }

            _ => Err(Error::new_spanned(type_ref, "Unexpected arg type")),
        }
    }

    fn from_path(type_ref: &Type, stream: &TypePath) -> SynResult<SimpleType> {
        if stream.path.segments.is_empty() {
            return Err(Error::new_spanned(stream, "Unexpected arg segment length"));
        }

        let first = &stream.path.segments[0];
        let type_ident = first.ident.clone();
        let type_name = type_ident.to_string();
        if type_name.find("Option").is_some() {
            match first.arguments {
                PathArguments::AngleBracketed(ref args) => {
                    if let GenericArgument::Type(Type::Path(ref actual_type)) = args.args[0] {
                        if actual_type.path.segments.is_empty() {
                            return Err(Error::new_spanned(
                                stream,
                                "Unexpected optional type length",
                            ));
                        }

                        let actual_ident = &actual_type.path.segments[0].ident;
                        return Ok(Self {
                            name: actual_ident.to_string(),
                            tokens: type_ref.clone(),
                            is_optional: true,
                            is_ref: false,
                        });
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            name: type_name,
            tokens: type_ref.clone(),
            is_optional: false,
            is_ref: false,
        })
    }
}

impl ToTokens for SimpleType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let type_tokens = &self.tokens;
        tokens.extend(quote! { #type_tokens })
    }
}
