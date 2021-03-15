use crate::{input::KeyError, script::api};
use std::{io, path::PathBuf};

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{
    builtins::PyStrRef,
    pyobject::{ItemProtocol, PyResult},
};

use super::{api::ApiManagerDelegate, ScriptingRuntime, ScriptingRuntimeFactory};

pub struct PythonScriptingRuntime {
    vm: vm::Interpreter,
}

impl PythonScriptingRuntime {
    fn new() -> Self {
        Self {
            vm: vm::Interpreter::default(),
        }
    }
}

impl ScriptingRuntime for PythonScriptingRuntime {
    fn load(&mut self, app: ApiManagerDelegate, path: PathBuf) -> std::io::Result<()> {
        let echo = move |message: PyStrRef, vm: &vm::VirtualMachine| {
            wrap_error(vm, api::core::echo(&app, message.to_string()))
        };

        let result: PyResult<()> = self.vm.enter(|runtime| {
            let scope = runtime.new_scope_with_builtins();

            let module = runtime.ctx.new_dict();
            module.set_item("echo", runtime.ctx.new_function("echo", echo), runtime)?;

            scope
                .globals
                .set_item("iaido", runtime.new_module("iaido", module), runtime)?;

            let code_obj = runtime
                .compile(
                    "iaido.echo('hello from python!')",
                    vm::compile::Mode::Exec,
                    path.to_string_lossy().to_string(),
                )
                .map_err(|e| runtime.new_syntax_error(&e))?;

            runtime.run_code_obj(code_obj, scope)?;

            Ok(())
        });

        // TODO: return the py exception properly
        match result {
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{:?}", e))),
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
    fn create(&self) -> Box<dyn ScriptingRuntime + Send> {
        Box::new(PythonScriptingRuntime::new())
    }

    fn handles_file(&self, path: &std::path::PathBuf) -> bool {
        if let Some(ext) = path.extension() {
            return ext == "py";
        }

        return false;
    }
}
