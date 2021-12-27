use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

use super::{
    buffer::BufferApiObject, connection::ConnectionApiObject, window::WindowApiObject, Api, Fns,
};

#[apigen::ns]
pub struct CurrentObjects {
    api: Api,
    fns: Fns,
}

#[apigen::ns_impl]
impl CurrentObjects {
    pub fn new(api: Api, fns: Fns) -> Self {
        Self { api, fns }
    }

    #[rpc]
    fn buffer_id(context: &mut CommandHandlerContext) -> Id {
        context.state().current_buffer().id()
    }

    #[property]
    pub fn buffer(&self) -> BufferApiObject {
        BufferApiObject::new(self.api.clone(), self.fns.clone(), self.buffer_id())
    }

    #[rpc]
    fn set_buffer_id(context: &mut CommandHandlerContext, buffer_id: Id) {
        context.state_mut().current_window_mut().buffer = buffer_id;
    }

    #[property(setter)]
    pub fn set_buffer(&self, buffer: &BufferApiObject) {
        self.set_buffer_id(buffer.id)
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
        WindowApiObject::new(self.api.clone(), self.fns.clone(), self.window_id())
    }
}

impl fmt::Debug for CurrentObjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<CurrentObjects>")
    }
}
