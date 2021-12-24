#![cfg(feature = "python")]

use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use rustpython_vm as vm;
use vm::{
    exceptions::PyBaseExceptionRef,
    pyobject::{ItemProtocol, PyResult},
};

use crate::{
    app::jobs::{JobError, JobResult},
    editing::Id,
};

use self::{modules::ModuleContained, util::unwrap_error};

use super::{
    api::{core::IaidoCore, ApiManagerDelegate},
    bindings::ScriptFile,
    fns::{FnManager, NativeFn, ScriptingFnRef},
    ScriptingRuntime, ScriptingRuntimeFactory,
};

mod compat;
mod modules;
pub mod util;

use compat::apply_compat;

pub struct PythonScriptingRuntime {
    fns: Arc<Mutex<FnManager>>,
    vm: Option<vm::Interpreter>,
}

fn declare_module(vm: &vm::VirtualMachine, name: &str, module: vm::pyobject::PyObjectRef) {
    vm.get_attribute(vm.sys_module.clone(), "modules")
        .expect("Failed to get sys modules")
        .set_item(name, module, &vm)
        .expect("failed to insert iaido module");
}

impl PythonScriptingRuntime {
    fn new(id: Id, api: ApiManagerDelegate) -> Self {
        let settings = vm::PySettings::default();
        let fns = Arc::new(Mutex::new(FnManager::new(id)));
        let runtime = PythonScriptingRuntime {
            fns: fns.clone(),
            vm: Some(vm::Interpreter::new(settings, vm::InitParameter::External)),
        };

        let iaido = IaidoCore::new(api.clone(), fns.clone());
        runtime.with_vm(move |vm| {
            let module = match iaido.clone().to_py_module(vm) {
                Ok(module) => module,
                Err(e) => panic!(
                    "Unable to initialize iaido module: {:?}",
                    Self::format_exception_vm(vm, e)
                ),
            };
            declare_module(vm, "iaido", module);

            apply_compat(iaido, vm);
        });

        runtime
    }

    fn with_vm<R>(&self, f: impl FnOnce(&vm::VirtualMachine) -> R) -> R {
        self.vm
            .as_ref()
            .expect("No VM somehow; this should not happen")
            .enter(f)
    }

    fn format_exception_vm(vm: &vm::VirtualMachine, e: PyBaseExceptionRef) -> String {
        let mut output: Vec<u8> = Vec::new();
        vm::exceptions::write_exception(&mut output, vm, &e).unwrap();
        String::from_utf8_lossy(&output).to_string()
    }

    fn format_exception(&mut self, e: PyBaseExceptionRef) -> String {
        self.with_vm(|vm| PythonScriptingRuntime::format_exception_vm(vm, e))
    }
}

impl ScriptingRuntime for PythonScriptingRuntime {
    fn load(&mut self, path: PathBuf) -> JobResult {
        let script = ScriptFile::read_from(path.clone())?;

        let result: PyResult<()> = self.with_vm(move |runtime| {
            let scope = runtime.new_scope_with_builtins();

            let package = script.package_name();
            if let Some(package) = package {
                script.ensure_module(runtime)?;

                scope
                    .globals
                    .set_item("__package__", runtime.ctx.new_str(package), runtime)?;
            }

            scope.globals.set_item(
                "__file__",
                runtime.ctx.new_str(script.path.clone()),
                runtime,
            )?;
            let code_obj = runtime
                .compile(&script.code, vm::compile::Mode::Exec, script.path)
                .map_err(|e| runtime.new_syntax_error(&e))?;

            runtime.run_code_obj(code_obj, scope)?;

            Ok(())
        });

        match result {
            Err(e) => Err(JobError::Script(self.format_exception(e))),

            _ => Ok(()),
        }
    }

    fn invoke(&mut self, fn_ref: ScriptingFnRef) -> JobResult {
        let fns = self.fns.clone();
        self.with_vm(move |vm| {
            let lock = fns.lock().unwrap();
            let native = lock.get(&fn_ref);
            let f = match native {
                NativeFn::Py(ref f) => f,
                _ => panic!("Received non-py Fn ref"),
            };

            unwrap_error(vm, vm.invoke(&f, vec![]))?;
            Ok(())
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
