use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, KeymapContext},
    script::api::manager::Api,
};

#[apigen::ns]
pub struct IaidoCore {
    api: Api,
}

#[apigen::ns_impl(module)]
impl IaidoCore {
    #[property]
    pub fn current(&self) -> CurrentObjects {
        CurrentObjects::new(self.api.clone())
    }

    #[rpc]
    pub fn echo(context: &mut CommandHandlerContext) -> Id {
        context.state_mut().echom("Hello from Python!");
        42
    }

    // #[rpc]
    // pub fn echo(context: &mut CommandHandlerContext, text: String) {
    //     context.state_mut().echom(text);
    // }
}

impl fmt::Debug for IaidoCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<iaido>")
    }
}

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

#[apigen::ns]
pub struct BufferApiObject {
    api: Api,
    pub id: Id,
}

impl fmt::Debug for BufferApiObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Buffer #{}>", self.id)
    }
}

#[apigen::ns_impl]
impl BufferApiObject {
    pub fn new(api: Api, id: Id) -> Self {
        Self { api, id }
    }

    #[property]
    #[rpc(passing(self.id))]
    pub fn name(context: &mut CommandHandlerContext, id: Id) -> Option<String> {
        if let Some(buf) = context.state().buffers.by_id(id) {
            Some(format!("{:?}", buf.source()))
        } else {
            None
        }
    }
}
