use proc_macro2::TokenStream;

mod python;

use python::PythonScriptingLang;

pub trait IaidoScriptingLang {
    fn wrap_ns(&self, ns: TokenStream) -> TokenStream {
        ns
    }
    fn wrap_ns_impl(&self, ns_impl: TokenStream) -> TokenStream {
        ns_impl
    }
}

struct ScriptingLangDelegate {
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
}

pub fn language() -> Box<dyn IaidoScriptingLang> {
    Box::new(ScriptingLangDelegate::default())
}
