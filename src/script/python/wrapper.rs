/*! API wrapper */

use std::sync::Arc;

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{
    builtins::PyStrRef,
    pyobject::{ItemProtocol, PyObjectRef, PyResult},
};

use crate::input::KeyError;
use crate::script::api::{core::IaidoApi, ApiManagerDelegate};

pub fn create_iaido_module(
    vm: &vm::VirtualMachine,
    api: Arc<IaidoApi<ApiManagerDelegate>>,
) -> PyResult<PyObjectRef> {
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

fn wrap_error<T>(vm: &vm::VirtualMachine, result: Result<T, KeyError>) -> PyResult<T> {
    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(vm.new_runtime_error(format!("{:?}", e))),
    }
}
