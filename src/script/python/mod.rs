use std::{io, path::PathBuf, sync::Arc};

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{exceptions::PyBaseExceptionRef, pyobject::PyResult};

use crate::{
    editing::Id,
    input::{maps::KeyResult, KeyError},
};

use self::wrapper::unwrap_error;

use super::{
    api::{
        core::{IaidoApi, ScriptingFnRef},
        ApiManagerDelegate,
    },
    bindings::ScriptFile,
    ScriptingRuntime, ScriptingRuntimeFactory,
};

mod wrapper;

use wrapper::create_iaido_module;

pub struct PythonScriptingRuntime {
    vm: Option<vm::Interpreter>,
}

impl PythonScriptingRuntime {
    fn new(id: Id, api: ApiManagerDelegate) -> Self {
        let settings = vm::PySettings::default();
        let iaido = Arc::new(IaidoApi::new(api));
        let mut runtime = PythonScriptingRuntime { vm: None };

        let vm = vm::Interpreter::new_with_init(settings, move |vm| {
            vm.add_native_module(
                "iaido".to_string(),
                Box::new(move |vm| {
                    create_iaido_module(vm, id, iaido.clone())
                        .expect("Unable to initialize iaido module")
                }),
            );

            vm::InitParameter::External
        });

        runtime.vm = Some(vm);

        runtime
    }

    fn with_vm<R>(&self, f: impl FnOnce(&vm::VirtualMachine) -> R) -> R {
        self.vm
            .as_ref()
            .expect("No VM somehow; this should not happen")
            .enter(f)
    }

    fn format_exception(&mut self, e: PyBaseExceptionRef) -> String {
        let mut output: Vec<u8> = Vec::new();
        self.with_vm(|vm| vm::exceptions::write_exception(&mut output, vm, &e))
            .unwrap();

        String::from_utf8_lossy(&output).to_string()
    }
}

impl ScriptingRuntime for PythonScriptingRuntime {
    fn load(&mut self, path: PathBuf) -> std::io::Result<()> {
        let script = ScriptFile::read_from(path)?;

        let result: PyResult<()> = self.with_vm(move |runtime| {
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

    fn invoke(&mut self, f: ScriptingFnRef) -> KeyResult {
        self.with_vm(move |vm| {
            let fns = vm
                .current_globals()
                .get_item_option("__iaido_fns__", vm)
                .expect("Could not find fns")
                .unwrap()
                .dict()
                .unwrap();
            if let Some(f) = unwrap_error(vm, fns.get_item_option(f.id.to_string(), vm))? {
                unwrap_error(vm, vm.invoke(&f, vec![]))?;
                Ok(())
            } else {
                Err(KeyError::InvalidInput("Invalid Fn ref".to_string()))
            }
        })
    }
}

pub struct PythonScriptingRuntimeFactory;
impl ScriptingRuntimeFactory for PythonScriptingRuntimeFactory {
    fn create(&self, id: Id, app: ApiManagerDelegate) -> Box<dyn ScriptingRuntime + Send> {
        Box::new(PythonScriptingRuntime::new(id, app))
    }

    fn handles_file(&self, path: &std::path::PathBuf) -> bool {
        if let Some(ext) = path.extension() {
            return ext == "py";
        }

        return false;
    }
}
