use crate::{
    editing::Id,
    input::{maps::KeyResult, KeyError},
};

use super::{ApiDelegate, ApiRequest, ApiResponse, IdType};

pub struct IaidoApi<A: ApiDelegate> {
    api: A,
}

#[derive(Clone, Copy)]
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

    pub fn buffer_name(&self, buf_id: Id) -> KeyResult<String> {
        match self.api.perform(ApiRequest::BufferName(buf_id))? {
            Some(ApiResponse::String(name)) => Ok(name),
            _ => Err(KeyError::Interrupted),
        }
    }

    pub fn connection_close(&self, conn_id: Id) -> KeyResult {
        match self.api.perform(ApiRequest::ConnectionClose(conn_id))? {
            Some(_) => Ok(()),
            _ => Err(KeyError::Interrupted),
        }
    }

    pub fn current_buffer(&self) -> KeyResult<Id> {
        match self.api.perform(ApiRequest::CurrentId(IdType::Buffer))? {
            Some(ApiResponse::Id(id)) => Ok(id),
            _ => Err(KeyError::Interrupted),
        }
    }

    pub fn set_current_buffer(&self, id: Id) -> KeyResult {
        match self
            .api
            .perform(ApiRequest::SetCurrentId(IdType::Buffer, id))?
        {
            Some(_) => Ok(()),
            _ => Err(KeyError::Interrupted),
        }
    }

    pub fn current_connection(&self) -> KeyResult<Option<Id>> {
        match self
            .api
            .perform(ApiRequest::CurrentId(IdType::Connection))?
        {
            Some(ApiResponse::Id(id)) => Ok(Some(id)),
            None => Ok(None),
            _ => Err(KeyError::Interrupted),
        }
    }

    pub fn echo(&self, message: String) -> KeyResult {
        self.api.perform(ApiRequest::Echo(message))?;
        Ok(())
    }

    pub fn set_keymap(&self, modes: String, from_keys: String, to: ScriptingFnRef) -> KeyResult {
        self.api.perform(ApiRequest::SetKeymapFn(
            modes.to_string(),
            from_keys.to_string(),
            to,
        ))?;
        Ok(())
    }
}
