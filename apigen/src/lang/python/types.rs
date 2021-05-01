use proc_macro2::TokenStream;
use quote::quote;
use syn::{Error, FnArg, PatType, Type};

use crate::types::{is_command_context, SimpleType, SynResult};

pub fn python_arg_name(arg: &FnArg) -> SynResult<Option<TokenStream>> {
    let PatType { ty, pat, .. } = match arg {
        FnArg::Typed(typed) => typed,
        _ => return Ok(None),
    };

    if is_command_context(ty) {
        // Never expose this to scripting
        return Ok(None);
    }

    Ok(Some(quote! { #pat }))
}

pub fn python_arg_from(arg: &FnArg) -> SynResult<Option<TokenStream>> {
    let PatType { ty, pat, .. } = match arg {
        FnArg::Typed(typed) => typed,
        _ => return Ok(None),
    };

    if is_command_context(ty) {
        // Never expose this to scripting
        return Ok(None);
    }

    let simple = SimpleType::from(&ty.as_ref())?;
    if simple.is_optional {
        return Err(Error::new_spanned(arg, "Optional args not supported yet"));
    }

    Ok(Some(match simple.name.as_str() {
        "Id" => quote! { #pat: usize },

        "String" => quote! { #pat: rustpython_vm::builtins::PyStrRef },
        "ScriptingFnRef" => quote! { #pat: rustpython_vm::pyobject::PyObjectRef },
        _ => {
            return Err(Error::new_spanned(
                simple,
                "Python does not support this type",
            ))
        }
    }))
}

pub fn python_conversion(arg: &FnArg) -> SynResult<Option<TokenStream>> {
    let PatType { ty, pat, .. } = match arg {
        FnArg::Typed(typed) => typed,
        _ => return Ok(None),
    };

    if is_command_context(ty) {
        // Never expose this to scripting
        return Ok(None);
    }

    let simple = SimpleType::from(&ty.as_ref())?;
    if simple.is_optional {
        return Err(Error::new_spanned(arg, "Optional args not supported yet"));
    }

    Ok(Some(match simple.name.as_str() {
        "Id" => quote! { #pat },

        "String" => quote! { #pat.to_string() },
        "ScriptingFnRef" => quote! {
            {
                let mut lock = self.fns.lock().unwrap();
                lock.create_ref(crate::script::fns::NativeFn::Py(#pat))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                simple,
                "Python does not support this type",
            ))
        }
    }))
}
