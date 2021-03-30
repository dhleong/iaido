use crate::{editing::Id, input::maps::KeyResult};

use super::{ApiDelegate, ApiRequest};

pub struct IaidoApi<A: ApiDelegate> {
    api: A,
}

pub struct ScriptingFnRef {
    pub runtime: Id,
    pub id: Id,
}

impl ScriptingFnRef {
    pub fn new(runtime: Id, id: Id) -> Self {
        Self { runtime, id }
    }
}

impl<A: ApiDelegate> IaidoApi<A> {
    pub fn new(api: A) -> Self {
        Self { api }
    }

    pub fn echo(&self, message: String) -> KeyResult {
        self.api.perform(ApiRequest::Echo(message))
    }

    pub fn set_keymap(&self, modes: String, from_keys: String, to: ScriptingFnRef) -> KeyResult {
        self.api.perform(ApiRequest::Echo(from_keys.to_string()))
    }
}
