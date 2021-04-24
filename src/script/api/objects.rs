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
}
