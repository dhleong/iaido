use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

use super::{buffer::BufferApiObject, Api};

#[apigen::ns]
pub struct CurrentObjects {
    api: Api,
}

#[apigen::ns_impl]
impl CurrentObjects {
    pub fn new(api: Api) -> Self {
        Self { api }
    }

    #[rpc]
    fn buffer_id(context: &mut CommandHandlerContext) -> Id {
        context.state().current_buffer().id()
    }

    #[property]
    pub fn buffer(&self) -> BufferApiObject {
        BufferApiObject::new(self.api.clone(), self.buffer_id())
    }
}

impl fmt::Debug for CurrentObjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<CurrentObjects>")
    }
}
