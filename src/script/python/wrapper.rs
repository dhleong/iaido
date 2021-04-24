/*! API wrapper */

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

// NOTE: ItemProtocol needs to be in scope in order to insert things into scope.globals
use rustpython_vm as vm;
use vm::{
    builtins::PyStrRef,
    pyobject::{IntoPyObject, ItemProtocol, PyObjectRef, PyResult},
};

use crate::script::api::{core::ScriptingFnRef, objects::CurrentObjects};
use crate::{
    editing::{ids::Ids, Id},
    script::api::{core::IaidoApi, ApiManagerDelegate},
};

use super::{
    objects::{init_objects, CurrentPyObjects},
    util::KeyResultConvertible,
};

pub struct FnManager {
    runtime_id: Id,
    ids: Ids,
    fns: HashMap<Id, PyObjectRef>,
}

impl FnManager {
    pub fn new(runtime_id: Id) -> Self {
        Self {
            runtime_id,
            ids: Ids::new(),
            fns: HashMap::new(),
        }
    }

    pub fn get(&self, fn_ref: &ScriptingFnRef) -> &PyObjectRef {
        self.fns.get(&fn_ref.id).expect("Invalid fn ref")
    }

    pub fn create_ref(&mut self, f: PyObjectRef) -> ScriptingFnRef {
        let id = self.ids.next();

        self.fns.insert(id, f);

        ScriptingFnRef::new(self.runtime_id, id)
    }
}

pub fn create_iaido_module(
    vm: &vm::VirtualMachine,
    api: Arc<IaidoApi<ApiManagerDelegate>>,
    fns: Arc<Mutex<FnManager>>,
) -> PyResult<PyObjectRef> {
    init_objects(vm);

    let dict = vm.ctx.new_dict();

    let current_objects = CurrentPyObjects::new(CurrentObjects::new(api.clone()));
    dict.set_item("current", current_objects.into_pyobject(vm), vm)?;

    let api_echo = api.clone();
    dict.set_item(
        "echo",
        vm.ctx
            .new_function("echo", move |message: PyStrRef, vm: &vm::VirtualMachine| {
                api_echo.echo(message.to_string()).wrap_err(vm)
            }),
        vm,
    )?;

    let api_set_keymap = api.clone();
    dict.set_item(
        "set_keymap",
        vm.ctx.new_function(
            "set_keymap",
            move |modes: PyStrRef, from_keys: PyStrRef, f: PyObjectRef, vm: &vm::VirtualMachine| {
                let fns = fns.clone();
                let mut lock = fns.lock().unwrap();
                let fn_ref = lock.create_ref(f);
                api_set_keymap
                    .set_keymap(modes.to_string(), from_keys.to_string(), fn_ref)
                    .wrap_err(vm)
            },
        ),
        vm,
    )?;

    Ok(vm.new_module("iaido", dict))
}
