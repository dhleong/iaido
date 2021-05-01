use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

use super::{buffer::BufferApiObject, window::WindowApiObject, Api};

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

    #[rpc]
    fn window_id(context: &mut CommandHandlerContext) -> Id {
        context.state().current_window().id
    }

    #[property]
    pub fn window(&self) -> WindowApiObject {
        WindowApiObject::new(self.api.clone(), self.window_id())
    }
}

impl fmt::Debug for CurrentObjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<CurrentObjects>")
    }
}
