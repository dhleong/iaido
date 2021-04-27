use syn::Type;

pub type SynResult<T = proc_macro2::TokenStream> = syn::parse::Result<T>;

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
