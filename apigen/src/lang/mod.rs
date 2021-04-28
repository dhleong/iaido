use proc_macro2::TokenStream;

mod python;

use python::PythonScriptingLang;

use crate::methods::MethodConfig;

pub trait IaidoScriptingLang {
    fn wrap_ns(&self, ns: TokenStream) -> TokenStream {
        ns
    }
    fn wrap_ns_impl(&self, ns_impl: TokenStream) -> TokenStream {
        ns_impl
    }
    fn wrap_fn(&self, f: TokenStream, _config: &MethodConfig) -> TokenStream {
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
    fn wrap_ns(&self, ns: TokenStream) -> TokenStream {
        let mut tokens = ns;
        for lang in &self.languages {
            tokens = lang.wrap_ns(tokens);
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

    fn wrap_fn(&self, f: TokenStream, config: &MethodConfig) -> TokenStream {
        let mut tokens = f;
        for lang in &self.languages {
            tokens = lang.wrap_fn(tokens, config);
        }
        tokens
    }
}

pub fn language() -> ScriptingLangDelegate {
    ScriptingLangDelegate::default()
}
