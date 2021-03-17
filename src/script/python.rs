use crate::{
    input::{maps::KeyResult, KeyError},
    script::api,
};
use std::{io, path::PathBuf, sync::Arc};

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{
    builtins::PyStrRef,
    exceptions::PyBaseExceptionRef,
    pyobject::{ItemProtocol, PyObjectRef, PyResult},
};

use super::{api::ApiManagerDelegate, ScriptingRuntime, ScriptingRuntimeFactory};

pub struct PythonScriptingRuntime {
    vm: vm::Interpreter,
}

struct Iaido {
    app: ApiManagerDelegate,
}

impl Iaido {
    pub fn echo(&self, message: String) -> KeyResult {
        api::core::echo(&self.app, message.to_string())
    }
}

fn create_iaido_module(vm: &vm::VirtualMachine, api: Arc<Iaido>) -> PyResult<PyObjectRef> {
    // let echo = move |message: PyStrRef, vm: &vm::VirtualMachine| {
    //     wrap_error(vm, api::core::echo(&app, message.to_string()))
    // };

    let dict = vm.ctx.new_dict();
    dict.set_item(
        "echo",
        vm.ctx
            .new_function("echo", move |message: PyStrRef, vm: &vm::VirtualMachine| {
                wrap_error(vm, api.echo(message.to_string()))
            }),
        vm,
    )?;

    Ok(vm.new_module("iaido", dict))
}

impl PythonScriptingRuntime {
    fn new(api: ApiManagerDelegate) -> Self {
        let settings = vm::PySettings::default();
        let iaido = Arc::new(Iaido { app: api });
        Self {
            vm: vm::Interpreter::new_with_init(settings, move |vm| {
                let moved_api = iaido;
                vm.add_native_module(
                    "iaido".to_string(),
                    Box::new(move |vm| {
                        let internal = moved_api.clone();
                        create_iaido_module(vm, internal)
                            .expect("Unable to initialize iaido module")
                    }),
                );

                vm::InitParameter::External
            }),
        }
    }

    fn format_exception(&mut self, e: PyBaseExceptionRef) -> String {
        let mut output: Vec<u8> = Vec::new();
        self.vm
            .enter(|vm| vm::exceptions::write_exception(&mut output, vm, &e))
            .unwrap();

        String::from_utf8_lossy(&output).to_string()
    }
}

impl ScriptingRuntime for PythonScriptingRuntime {
    fn load(&mut self, path: PathBuf) -> std::io::Result<()> {
        let result: PyResult<()> = self.vm.enter(|runtime| {
            let scope = runtime.new_scope_with_builtins();
            let code_obj = runtime
                .compile(
                    r#"
import iaido
iaido.echo('hello from python!')
                    "#
                    .trim(),
                    vm::compile::Mode::Exec,
                    path.to_string_lossy().to_string(),
                )
                .map_err(|e| runtime.new_syntax_error(&e))?;

            runtime.run_code_obj(code_obj, scope)?;

            Ok(())
        });

        match result {
            Err(e) => Err(io::Error::new(
                io::ErrorKind::Other,
                self.format_exception(e),
            )),

            _ => Ok(()),
        }
    }
}

fn wrap_error<T>(vm: &vm::VirtualMachine, result: Result<T, KeyError>) -> PyResult<T> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(vm.new_runtime_error(format!("{:?}", e))),
    }
}

pub struct PythonScriptingRuntimeFactory;
impl ScriptingRuntimeFactory for PythonScriptingRuntimeFactory {
    fn create(&self, app: ApiManagerDelegate) -> Box<dyn ScriptingRuntime + Send> {
        Box::new(PythonScriptingRuntime::new(app))
    }

    fn handles_file(&self, path: &std::path::PathBuf) -> bool {
        if let Some(ext) = path.extension() {
            return ext == "py";
        }

        return false;
    }
}
