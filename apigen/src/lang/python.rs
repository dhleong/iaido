#![allow(unused_imports)]
use crate::{methods::MethodConfig, ns::Ns, types::SynResult};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, Item, ItemFn, Signature, Visibility};

use super::IaidoScriptingLang;

mod types;

pub struct PythonScriptingLang;

#[cfg(not(feature = "python"))]
impl IaidoScriptingLang for PythonScriptingLang {}

// #[cfg(feature = "python")]
impl IaidoScriptingLang for PythonScriptingLang {
    fn wrap_ns(&self, ns: TokenStream, item: &Ns) -> TokenStream {
        let ns_name = &item.name;
        quote! {
            #[rustpython_vm::pyclass(module="iaido", name)]
            #ns

            impl rustpython_vm::pyobject::PyValue for #ns_name {
                fn class(
                    _vm: &rustpython_vm::VirtualMachine
                ) -> &rustpython_vm::builtins::PyTypeRef {
                    use rustpython_vm::pyobject::StaticType;
                    Self::static_type()
                }
            }
        }
    }

    fn wrap_ns_impl(&self, ns_impl: TokenStream) -> TokenStream {
        quote! {
            #[rustpython_vm::pyimpl]
            #ns_impl
        }
    }

    fn wrap_fn(
        &self,
        f: TokenStream,
        item: &ItemFn,
        config: &MethodConfig,
    ) -> SynResult<TokenStream> {
        if config.is_property {
            return Ok(quote! {
                #[pyproperty]
                #f
            });
        } else if config.is_method || (is_public(item) && config.is_rpc) {
            let ItemFn { sig, .. } = item;
            let name = &sig.ident;
            let py_name = Ident::new(format!("{}_py", name).as_str(), name.span());
            let name_string = name.to_string();

            let args = self.convert_args_to_python(sig)?;
            let converted = self.arg_conversions_from_python(sig)?;

            return Ok(quote! {
                #f

                #[pymethod(name = #name_string)]
                fn #py_name(&self, #(#args),*) {
                    self.#name(#(#converted),*)
                }
            });
        }

        Ok(f)
    }
}

// #[cfg(feature = "python")]
impl PythonScriptingLang {
    fn convert_args_to_python(&self, sig: &Signature) -> SynResult<Vec<TokenStream>> {
        map_args(sig, types::python_arg_from)
    }

    fn arg_conversions_from_python(&self, sig: &Signature) -> SynResult<Vec<TokenStream>> {
        map_args(sig, types::python_conversion)
    }
}

fn map_args<F>(sig: &Signature, f: F) -> SynResult<Vec<TokenStream>>
where
    F: Fn(&FnArg) -> SynResult<Option<TokenStream>>,
{
    let mut args = vec![];
    for arg in &sig.inputs {
        if let Some(replacement) = f(arg)? {
            args.push(replacement);
        }
    }
    Ok(args)
}

fn is_public(item: &ItemFn) -> bool {
    if let Visibility::Public(_) = item.vis {
        true
    } else {
        false
    }
}
