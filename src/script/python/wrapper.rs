/*! API wrapper */

use std::sync::Arc;

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{
    builtins::PyStrRef,
    pyobject::{ItemProtocol, PyObjectRef, PyResult},
};

use crate::{
    editing::Id,
    script::api::{core::IaidoApi, ApiManagerDelegate},
};
use crate::{input::KeyError, script::api::core::ScriptingFnRef};

pub fn create_iaido_module(
    vm: &vm::VirtualMachine,
    runtime_id: Id,
    api: Arc<IaidoApi<ApiManagerDelegate>>,
) -> PyResult<PyObjectRef> {
    let dict = vm.ctx.new_dict();
    let fns = vm.ctx.new_dict();

    vm.current_globals()
        .set_item("__iaido_fns__", fns.as_object().clone(), vm)?;

    let api_echo = api.clone();
    dict.set_item(
        "echo",
        vm.ctx
            .new_function("echo", move |message: PyStrRef, vm: &vm::VirtualMachine| {
                wrap_error(vm, api_echo.echo(message.to_string()))
            }),
        vm,
    )?;

    let api_set_keymap = api.clone();
    dict.set_item(
        "set_keymap",
        vm.ctx.new_function(
            "set_keymap",
            move |modes: PyStrRef, from_keys: PyStrRef, f: PyObjectRef, vm: &vm::VirtualMachine| {
                fns.set_item("0", f, vm)?;
                wrap_error(
                    vm,
                    api_set_keymap.set_keymap(
                        modes.to_string(),
                        from_keys.to_string(),
                        ScriptingFnRef::new(runtime_id, 0),
                    ),
                )
            },
        ),
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

pub fn unwrap_error<T>(_vm: &vm::VirtualMachine, result: PyResult<T>) -> Result<T, KeyError> {
    // TODO: format exception better
    match result {
        Ok(v) => Ok(v),
        Err(e) => Err(KeyError::InvalidInput(format!("{:?}", e))),
    }
}
