use proc_macro2::TokenStream;

mod python;

use python::PythonScriptingLang;
use syn::ItemFn;

use crate::{methods::MethodConfig, ns::Ns};

pub trait IaidoScriptingLang {
    fn wrap_ns(&self, ns: TokenStream, _item: &Ns) -> TokenStream {
        ns
    }
    fn wrap_ns_impl(&self, ns_impl: TokenStream) -> TokenStream {
        ns_impl
    }
    fn wrap_fn(&self, f: TokenStream, _item: &ItemFn, _config: &MethodConfig) -> TokenStream {
        f
    }
}

pub struct ScriptingLangDelegate {
    languages: Vec<Box<dyn IaidoScriptingLang>>,
}

impl Default for ScriptingLangDelegate {
    fn default() -> Self {
        Self {
            languages: vec![Box::new(PythonScriptingLang)],
        }
    }
}

impl IaidoScriptingLang for ScriptingLangDelegate {
    fn wrap_ns(&self, ns: TokenStream, item: &Ns) -> TokenStream {
        let mut tokens = ns;
        for lang in &self.languages {
            tokens = lang.wrap_ns(tokens, item);
        }
        tokens
    }

    fn wrap_ns_impl(&self, ns_impl: TokenStream) -> TokenStream {
        let mut tokens = ns_impl;
        for lang in &self.languages {
            tokens = lang.wrap_ns_impl(tokens);
        }
        tokens
    }

    fn wrap_fn(&self, f: TokenStream, item: &ItemFn, config: &MethodConfig) -> TokenStream {
        let mut tokens = f;
        for lang in &self.languages {
            tokens = lang.wrap_fn(tokens, item, config);
        }
        tokens
    }
}

pub fn language() -> ScriptingLangDelegate {
    ScriptingLangDelegate::default()
}
