/*! API wrapper */

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use rustpython_vm as vm;
use vm::pyobject::{PyObjectRef, PyResult};

use crate::script::api::{core::ScriptingFnRef, core2::IaidoCore};
use crate::{
    editing::{ids::Ids, Id},
    script::api::manager::Api,
};

use super::objects::init_objects;

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
    api: Api,
    fns: Arc<Mutex<FnManager>>,
) -> PyResult<PyObjectRef> {
    panic!("deprecated");

    // init_objects(vm);
    //
    // let core = IaidoCore::new(api #<{(| , fns |)}>#);
    //
    // // TODO: restore this
    // // let api_set_keymap = api.clone();
    // // dict.set_item(
    // //     "set_keymap",
    // //     vm.ctx.new_function(
    // //         "set_keymap",
    // //         move |modes: PyStrRef, from_keys: PyStrRef, f: PyObjectRef, vm: &vm::VirtualMachine| {
    // //             let fns = fns.clone();
    // //             let mut lock = fns.lock().unwrap();
    // //             let fn_ref = lock.create_ref(f);
    // //             api_set_keymap
    // //                 .set_keymap(modes.to_string(), from_keys.to_string(), fn_ref)
    // //                 .wrap_err(vm)
    // //         },
    // //     ),
    // //     vm,
    // // )?;
    //
    // return core.to_py_module(vm);
}
