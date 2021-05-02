use std::fmt;

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, maps::KeyResult, KeymapContext},
};

use super::Api;

#[apigen::ns]
pub struct ConnectionApiObject {
    api: Api,
    pub id: Id,
}

impl fmt::Debug for ConnectionApiObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Connection #{}>", self.id)
    }
}

#[apigen::ns_impl]
impl ConnectionApiObject {
    pub fn new(api: Api, id: Id) -> Self {
        Self { api, id }
    }

    #[rpc(passing(self.id))]
    pub fn close(context: &mut CommandHandlerContext, id: Id) -> KeyResult {
        if let Some(ref mut conns) = context.state_mut().connections {
            conns.disconnect(id)?;
        }
        Ok(())
    }
}
