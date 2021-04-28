#![allow(unused_imports)]
use crate::{methods::MethodConfig, ns::Ns};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Item;

use super::IaidoScriptingLang;

pub struct PythonScriptingLang;

#[cfg(not(feature = "python"))]
impl IaidoScriptingLang for PythonScriptingLang {}

#[cfg(feature = "python")]
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
        // TODO probably not here, but *somewhere* we need to add the
        // #[pyproperty] attribute to methods previously annotated with
        // #[property]... but, perhaps somehow conditionally based on
        // whether the script language's features are selected?
        quote! {
            #[rustpython_vm::pyimpl]
            #ns_impl
        }
    }

    fn wrap_fn(&self, f: TokenStream, config: &MethodConfig) -> TokenStream {
        if config.is_property {
            return quote! {
                #[pyproperty]
                #f
            };
        }

        f
    }
}
