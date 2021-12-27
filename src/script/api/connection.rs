use std::fmt;

use crate::{
    editing::Id,
    input::{
        commands::{connection::on_disconnect, CommandHandlerContext},
        maps::{actions::connection::send_string_to_buffer, KeyResult},
        KeyError, KeymapContext,
    },
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
            let buffer_id = conns.disconnect(id)?;

            on_disconnect(context, buffer_id);
        }
        Ok(())
    }

    #[rpc(passing(self.id))]
    pub fn send(context: &mut CommandHandlerContext, id: Id, text: String) -> KeyResult {
        if let Some(conn_buffer_id) = context
            .state_mut()
            .connections
            .as_ref()
            .and_then(|conns| conns.id_to_buffer(id))
        {
            send_string_to_buffer(context, conn_buffer_id, text)
        } else {
            Err(KeyError::IO(std::io::ErrorKind::NotConnected.into()))
        }
    }
}
