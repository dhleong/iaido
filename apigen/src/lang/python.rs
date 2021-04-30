#![allow(unused_imports)]
use crate::{methods::MethodConfig, ns::Ns};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, ItemFn, Visibility};

use super::IaidoScriptingLang;

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

    fn wrap_fn(&self, f: TokenStream, item: &ItemFn, config: &MethodConfig) -> TokenStream {
        if config.is_property {
            return quote! {
                #[pyproperty]
                #f
            };
        } else if config.is_method || (is_public(item) && config.is_rpc) {
            return quote! {
                #[pymethod]
                #f
            };
        }

        f
    }
}

fn is_public(item: &ItemFn) -> bool {
    if let Visibility::Public(_) = item.vis {
        true
    } else {
        false
    }
}
