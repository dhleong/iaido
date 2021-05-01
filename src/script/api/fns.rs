use std::collections::HashMap;

use crate::editing::{ids::Ids, Id};

#[derive(Clone, Copy, Debug)]
pub struct ScriptingFnRef {
    pub runtime: Id,
    pub id: Id,
}

impl ScriptingFnRef {
    pub fn new(runtime: Id, id: Id) -> Self {
        Self { runtime, id }
    }
}

pub enum NativeFn {
    #[cfg(feature = "python")]
    Py(rustpython_vm::pyobject::PyObjectRef),

    /// NOTE: This exists to avoid "irrefutable" warnings
    _Ignore,
}

pub struct FnManager {
    runtime_id: Id,
    ids: Ids,
    fns: HashMap<Id, NativeFn>,
}

impl FnManager {
    pub fn new(runtime_id: Id) -> Self {
        Self {
            runtime_id,
            ids: Ids::new(),
            fns: HashMap::new(),
        }
    }

    pub fn get(&self, fn_ref: &ScriptingFnRef) -> &NativeFn {
        self.fns.get(&fn_ref.id).expect("Invalid fn ref")
    }

    pub fn create_ref(&mut self, f: NativeFn) -> ScriptingFnRef {
        let id = self.ids.next();

        self.fns.insert(id, f);

        ScriptingFnRef::new(self.runtime_id, id)
    }
}
