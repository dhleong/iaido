use rustpython_vm as vm;
use vm::pyobject::PyResult;

use crate::{
    app::jobs::{JobError, JobResult},
    input::KeyError,
};

use super::PythonScriptingRuntime;

pub trait KeyResultConvertible<T> {
    fn wrap_err(self, vm: &vm::VirtualMachine) -> PyResult<T>;
}

impl<T> KeyResultConvertible<T> for Result<T, KeyError> {
    fn wrap_err(self, vm: &vm::VirtualMachine) -> PyResult<T> {
        self.map_err(|e| vm.new_runtime_error(format!("{:?}", e)))
    }
}

pub fn unwrap_error<T>(vm: &vm::VirtualMachine, result: PyResult<T>) -> JobResult<T> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(JobError::Script(
            PythonScriptingRuntime::format_exception_vm(vm, e),
        )),
    }
}
