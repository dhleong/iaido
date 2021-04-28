#![allow(unused_imports)]
use crate::methods::MethodConfig;
use proc_macro2::TokenStream;
use quote::quote;

use super::IaidoScriptingLang;

pub struct PythonScriptingLang;

#[cfg(not(feature = "python"))]
impl IaidoScriptingLang for PythonScriptingLang {}

#[cfg(feature = "python")]
impl IaidoScriptingLang for PythonScriptingLang {
    fn wrap_ns(&self, ns: TokenStream) -> TokenStream {
        quote! {
            #[rustpython_vm::pyclass(module="iaido", name)]
            #ns
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
