use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

use super::{
    buffer::BufferApiObject, connection::ConnectionApiObject, window::WindowApiObject, Api,
};

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
    fn connection_id(context: &mut CommandHandlerContext) -> Option<Id> {
        context
            .state()
            .connections
            .as_ref()
            .and_then(|conns| conns.buffer_to_id(context.state().current_buffer().id()))
    }

    #[property]
    pub fn connection(&self) -> Option<ConnectionApiObject> {
        if let Some(id) = self.connection_id() {
            Some(ConnectionApiObject::new(self.api.clone(), id))
        } else {
            None
        }
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
