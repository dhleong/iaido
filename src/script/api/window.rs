use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
};

use super::{buffer::BufferApiObject, Api, Fns};

#[apigen::ns]
pub struct WindowApiObject {
    api: Api,
    fns: Fns,
    id: Id,
}

#[apigen::ns_impl]
impl WindowApiObject {
    pub fn new(api: Api, fns: Fns, id: Id) -> Self {
        Self { api, fns, id }
    }

    #[property]
    pub fn buffer(&self) -> Option<BufferApiObject> {
        if let Some(id) = self.buffer_id() {
            Some(BufferApiObject::new(self.api.clone(), self.fns.clone(), id))
        } else {
            None
        }
    }

    #[rpc(passing(self.id))]
    fn buffer_id(context: &mut CommandHandlerContext, win_id: Id) -> Option<Id> {
        if let Some(win) = context.state_mut().bufwin_by_id(win_id) {
            Some(win.buffer.id())
        } else {
            None
        }
    }

    #[rpc(passing(self.id))]
    pub fn close(context: &mut CommandHandlerContext, id: Id) {
        context.state_mut().close_window(id);
    }
}

impl fmt::Debug for WindowApiObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Window #{}>", self.id)
    }
}
