use proc_macro2::TokenStream;

mod python;

use python::PythonScriptingLang;
use syn::ItemFn;

use crate::{
    methods::MethodConfig,
    ns::Ns,
    ns_impl::{NsImpl, NsImplConfig},
    types::SynResult,
};

pub struct ConfiguredNsImpl {
    pub ns: NsImpl,
    pub config: NsImplConfig,
}

impl ConfiguredNsImpl {
    pub fn new(ns: NsImpl, config: NsImplConfig) -> Self {
        Self { ns, config }
    }

    pub fn to_tokens<L: IaidoScriptingLang>(&self, language: &L) -> SynResult<TokenStream> {
        self.ns.to_tokens(language)
    }
}

pub trait IaidoScriptingLang {
    fn wrap_ns(&self, ns: TokenStream, _item: &Ns) -> TokenStream {
        ns
    }
    fn wrap_ns_impl(&self, tokens: TokenStream, _ns: &ConfiguredNsImpl) -> SynResult<TokenStream> {
        Ok(tokens)
    }
    fn wrap_fn(
        &self,
        f: TokenStream,
        _item: &ItemFn,
        _config: &MethodConfig,
    ) -> SynResult<TokenStream> {
        Ok(f)
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

    fn wrap_ns_impl(&self, tokens: TokenStream, ns: &ConfiguredNsImpl) -> SynResult<TokenStream> {
        let mut tokens = tokens;
        for lang in &self.languages {
            tokens = lang.wrap_ns_impl(tokens, ns)?;
        }
        Ok(tokens)
    }

    fn wrap_fn(
        &self,
        f: TokenStream,
        item: &ItemFn,
        config: &MethodConfig,
    ) -> SynResult<TokenStream> {
        let mut tokens = f;
        for lang in &self.languages {
            tokens = lang.wrap_fn(tokens, item, config)?;
        }
        Ok(tokens)
    }
}

pub fn language() -> ScriptingLangDelegate {
    ScriptingLangDelegate::default()
}
