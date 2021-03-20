use std::{io, path::PathBuf, sync::Arc};

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{exceptions::PyBaseExceptionRef, pyobject::PyResult};

use super::{
    api::{core::IaidoApi, ApiManagerDelegate},
    bindings::ScriptFile,
    ScriptingRuntime, ScriptingRuntimeFactory,
};

mod wrapper;

use wrapper::create_iaido_module;

pub struct PythonScriptingRuntime {
    vm: vm::Interpreter,
}

impl PythonScriptingRuntime {
    fn new(api: ApiManagerDelegate) -> Self {
        let settings = vm::PySettings::default();
        let iaido = Arc::new(IaidoApi::new(api));
        Self {
            vm: vm::Interpreter::new_with_init(settings, move |vm| {
                vm.add_native_module(
                    "iaido".to_string(),
                    Box::new(move |vm| {
                        create_iaido_module(vm, iaido.clone())
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
        let script = ScriptFile::read_from(path)?;

        let result: PyResult<()> = self.vm.enter(move |runtime| {
            let scope = runtime.new_scope_with_builtins();
            let code_obj = runtime
                .compile(&script.code, vm::compile::Mode::Exec, script.path)
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
