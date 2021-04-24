use std::{fmt, sync::Arc};

use crate::{editing::Id, input::maps::KeyResult};

use super::{core::IaidoApi, ApiManagerDelegate};

type Api = Arc<IaidoApi<ApiManagerDelegate>>;

pub struct CurrentObjects {
    api: Api,
}

impl CurrentObjects {
    pub fn new(api: Api) -> Self {
        Self { api }
    }

    pub fn buffer(&self) -> KeyResult<BufferApiObject> {
        Ok(BufferApiObject::new(
            self.api.clone(),
            self.api.current_buffer()?,
        ))
    }

    pub fn set_buffer(&self, buffer: &BufferApiObject) -> KeyResult {
        self.api.set_current_buffer(buffer.id)
    }

    pub fn connection(&self) -> KeyResult<Option<ConnectionApiObject>> {
        if let Some(id) = self.api.current_connection()? {
            Ok(Some(ConnectionApiObject::new(self.api.clone(), id)))
        } else {
            Ok(None)
        }
    }
}

pub struct BufferApiObject {
    api: Api,
    pub id: Id,
}

impl fmt::Debug for BufferApiObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Buffer #{}>", self.id)
    }
}

impl BufferApiObject {
    pub fn new(api: Api, id: Id) -> Self {
        Self { api, id }
    }

    pub fn name(&self) -> KeyResult<String> {
        self.api.buffer_name(self.id)
    }
}

pub struct ConnectionApiObject {
    api: Api,
    pub id: Id,
}

impl fmt::Debug for ConnectionApiObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Connection #{}>", self.id)
    }
}

impl ConnectionApiObject {
    pub fn new(api: Api, id: Id) -> Self {
        Self { api, id }
    }

    pub fn close(&self) -> KeyResult {
        self.api.connection_close(self.id)
    }
}
