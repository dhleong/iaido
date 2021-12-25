use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, maps::KeyResult, KeymapContext},
};

use super::Api;

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

    #[rpc(passing(self.id))]
    pub fn alias(
        context: &mut CommandHandlerContext,
        id: Id,
        pattern: String,
        replacement: String,
    ) -> KeyResult {
        if let Some(ref mut conns) = context.state_mut().connections {
            conns.with_buffer_engine(id, |engine| {
                engine.aliases.insert_text(pattern, replacement)
            })?;
        }
        Ok(())
    }
}
