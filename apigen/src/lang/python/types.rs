#![cfg(feature = "python")]

use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
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

    let arg_type = python_type_from_simple(&simple)?;
    Ok(Some(quote! { #pat: #arg_type }))
}

fn python_type_from_simple(simple: &SimpleType) -> SynResult<TokenStream> {
    if simple.is_ref {
        // is there a better way to handle this?
        return Ok(quote! { rustpython_vm::pyobject::PyRef<#simple> });
    }

    Ok(match simple.name.as_str() {
        "Id" => quote! { usize },

        "Either" => match &simple.generic_types {
            Some(args) if args.len() == 2 => {
                let a = python_type_from_simple(&args[0])?;
                let b = python_type_from_simple(&args[1])?;
                quote! { rustpython_vm::pyobject::Either<#a, #b> }
            }
            _ => {
                return Err(Error::new_spanned(
                    simple,
                    "`Either` requires exactly 2 generic type parameters",
                ))
            }
        },
        "String" => quote! { rustpython_vm::builtins::PyStrRef },
        "ScriptingFnRef" => quote! { rustpython_vm::pyobject::PyObjectRef },
        _ => {
            let msg = format!(
                "Python does not support the type {}; try using a ref for API object types",
                simple.name,
            );
            return Err(Error::new_spanned(simple, msg));
        }
    })
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

    Ok(Some(python_conversion_simple(pat, &simple)?))
}

fn python_conversion_simple<T: ToTokens>(pat: T, simple: &SimpleType) -> SynResult<TokenStream> {
    if simple.is_ref {
        return Ok(quote! { &#pat });
    }

    Ok(match simple.name.as_str() {
        "Id" => quote! { #pat },

        "Either" => match &simple.generic_types {
            Some(arg_types) if arg_types.len() == 2 => {
                let ident_a = format_ident!("a");
                let ident_b = format_ident!("b");
                let convert_a = python_conversion_simple(ident_a.clone(), &arg_types[0])?;
                let convert_b = python_conversion_simple(ident_b.clone(), &arg_types[1])?;
                quote! {
                    match #pat {
                        rustpython_vm::pyobject::Either::A(#ident_a) => Either::A(#convert_a),
                        rustpython_vm::pyobject::Either::B(#ident_b) => Either::B(#convert_b),
                    }
                }
            }
            _ => {
                return Err(Error::new_spanned(
                    simple,
                    "Either requires exactly 2 generic type parameters",
                ))
            }
        },

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
    })
}
