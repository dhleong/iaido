#![allow(unused_imports)]
use crate::{
    methods::{MethodConfig, RpcConfig},
    ns::Ns,
    types::{result_type, SimpleType, SynResult},
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Ident, Item, ItemFn, PatType, Signature, Visibility};

use super::{ConfiguredNsImpl, IaidoScriptingLang};

mod types;

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
                    vm: &rustpython_vm::VirtualMachine
                ) -> &rustpython_vm::builtins::PyTypeRef {
                    use rustpython_vm::pyobject::StaticType;
                    use rustpython_vm::pyobject::PyClassImpl;
                    Self::make_class(&vm.ctx);
                    Self::static_type()
                }
            }
        }
    }

    fn wrap_ns_impl(&self, tokens: TokenStream, ns: &ConfiguredNsImpl) -> SynResult<TokenStream> {
        let to_module = if ns.config.is_module {
            let name = &ns.ns.name;
            let to_module_impl = self.generate_to_py_module(ns)?;
            quote! {
                impl #name {
                    pub fn to_py_module(
                        &self,
                        vm: &rustpython_vm::VirtualMachine
                    ) -> rustpython_vm::pyobject::PyResult<
                        rustpython_vm::pyobject::PyObjectRef
                    > {
                        #to_module_impl
                    }
                }
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            #[rustpython_vm::pyimpl(flags(BASETYPE, HAS_DICT))]
            #tokens

            #to_module
        })
    }

    fn wrap_fn(
        &self,
        f: TokenStream,
        item: &ItemFn,
        config: &MethodConfig,
    ) -> SynResult<TokenStream> {
        if config.is_property && !config.is_property_setter {
            return Ok(quote! {
                #[pyproperty]
                #f
            });
        } else if config.is_method || (is_public(item) && config.is_rpc) || config.is_property {
            let ItemFn { sig, .. } = item;
            let name = &sig.ident;
            let output = &sig.output;
            let py_name = py_wrapper_fn_name(name);
            let name_string = name.to_string();
            let result_type = result_type(&sig);

            let mut args = convert_args_to_python(sig, &config.rpc_config)?;
            let converted = self.arg_conversions_from_python(sig, &config.rpc_config)?;

            let invocation = quote! { self.#name(#(#converted),*) };

            let wrapped_output = if let Some(result) = &result_type {
                quote! { -> rustpython_vm::pyobject::PyResult<#result> }
            } else {
                quote! { #output }
            };

            let invocation = if result_type.is_some() {
                args.push(quote! { vm: &rustpython_vm::VirtualMachine });
                quote! {
                    use crate::script::python::util::KeyResultConvertible;
                    #invocation.wrap_err(vm)
                }
            } else {
                invocation
            };

            let annotation = if config.is_property {
                let getter_string = name_string.replace("set_", "");
                quote! { #[pyproperty(name = #getter_string, setter)] }
            } else {
                quote! { #[pymethod(name = #name_string)] }
            };

            return Ok(quote! {
                #f

                #annotation
                pub fn #py_name(&self, #(#args),*) #wrapped_output {
                    #invocation
                }
            });
        }

        Ok(f)
    }
}

#[cfg(feature = "python")]
impl PythonScriptingLang {
    fn arg_conversions_from_python(
        &self,
        sig: &Signature,
        rpc_config: &Option<RpcConfig>,
    ) -> SynResult<Vec<TokenStream>> {
        map_args(sig, rpc_config, types::python_conversion)
    }

    fn generate_to_py_module(&self, ns: &ConfiguredNsImpl) -> SynResult<TokenStream> {
        let ConfiguredNsImpl { ns, config } = ns;
        let module_name = &config.module_name;
        let mut definitions = vec![];

        for method in &ns.direct_fns {
            if method.config.is_property {
                definitions.push(generate_module_property(&method.item.sig));
            } else if method.config.is_method {
                definitions.push(generate_module_function(&method.item)?);
            }
        }

        for method in &ns.rpc_fns {
            if method.config.is_property {
                definitions.push(generate_module_property(&method.item.sig));
            } else {
                definitions.push(generate_module_function(&method.item)?);
            }
        }

        Ok(quote! {
            use rustpython_vm::pyobject::ItemProtocol;

            let dict = vm.ctx.new_dict();

            #(#definitions)*

            Ok(vm.new_module(#module_name, dict))
        })
    }
}

#[cfg(feature = "python")]
fn map_args<F>(sig: &Signature, rpc_config: &Option<RpcConfig>, f: F) -> SynResult<Vec<TokenStream>>
where
    F: Fn(&FnArg) -> SynResult<Option<TokenStream>>,
{
    let mut args = vec![];
    let skippable_count = if let Some(config) = rpc_config {
        config.rpc_args.len()
    } else {
        0
    };

    let mut seen = 0;
    for arg in &sig.inputs {
        if is_command_context(arg) {
            continue;
        }

        seen += 1;
        if seen <= skippable_count {
            continue;
        }

        if let Some(replacement) = f(arg)? {
            args.push(replacement);
        }
    }
    Ok(args)
}

#[cfg(feature = "python")]
fn is_public(item: &ItemFn) -> bool {
    if let Visibility::Public(_) = item.vis {
        true
    } else {
        false
    }
}

/// Given the Ident of a fn name, generate the name of a wrapper
/// fn that accepts Python-specific type arguments and converts
/// calls through to the original fn, converting the arguments
#[cfg(feature = "python")]
fn py_wrapper_fn_name(name: &Ident) -> Ident {
    Ident::new(format!("{}_py", name).as_str(), name.span())
}

#[cfg(feature = "python")]
fn generate_module_property(sig: &Signature) -> TokenStream {
    // TODO in general, for any module-level properties this
    // *should* be fine; if not, we may be able to define the
    // property accessor on the module, instead of in the dict?
    let name = &sig.ident;
    let name_str = name.to_string();
    quote! {
        {
            use rustpython_vm::pyobject::IntoPyObject;
            dict.set_item(#name_str, self.#name().into_pyobject(vm), vm)?;
        }
    }
}

#[cfg(feature = "python")]
fn convert_args_to_python(
    sig: &Signature,
    rpc_config: &Option<RpcConfig>,
) -> SynResult<Vec<TokenStream>> {
    map_args(sig, rpc_config, types::python_arg_from)
}

#[cfg(feature = "python")]
fn generate_module_function(item: &ItemFn) -> SynResult<TokenStream> {
    let name = &item.sig.ident;
    let py_name = py_wrapper_fn_name(name);
    let name_str = name.to_string();
    let result_type = result_type(&item.sig);

    let arg_decls = convert_args_to_python(&item.sig, &None)?;
    let arg_names = map_args(&item.sig, &None, types::python_arg_name)?;

    let (vm_arg, vm_usage) = if result_type.is_some() {
        (quote! {vm: &rustpython_vm::VirtualMachine}, quote! { vm })
    } else {
        (quote!(), quote!())
    };

    Ok(quote! {
        {
            let zelf = self.clone();
            dict.set_item(
                #name_str,
                vm.ctx.new_function(#name_str, move |#(#arg_decls),*, #vm_arg| {
                    zelf.#py_name(#(#arg_names),*, #vm_usage)
                }),
                vm
            )?;
        }
    })
}

#[cfg(feature = "python")]
fn is_command_context(arg: &FnArg) -> bool {
    let PatType { ty, .. } = match arg {
        FnArg::Typed(typed) => typed,
        _ => return false,
    };

    crate::types::is_command_context(ty)
}
