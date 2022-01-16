use std::{
    collections::HashMap,
    fmt,
    sync::{Arc, Mutex},
};

use crate::{
    editing::Id,
    input::{commands::CommandHandlerContext, maps::KeyResult, KeyError, KeymapContext},
    script::{args::FnArgs, fns::ScriptingFnRef, poly::Either, ScriptingManager},
};

use super::{connection::ConnectionApiObject, Api, Fns};

#[apigen::ns]
pub struct BufferApiObject {
    api: Api,
    fns: Fns,
    pub id: Id,
}

impl fmt::Debug for BufferApiObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<Buffer #{}>", self.id)
    }
}

#[apigen::ns_impl]
impl BufferApiObject {
    pub fn new(api: Api, fns: Fns, id: Id) -> Self {
        Self { api, fns, id }
    }

    #[rpc(passing(self.id))]
    fn connection_id(context: &mut CommandHandlerContext, buffer_id: Id) -> Option<Id> {
        context
            .state()
            .connections
            .as_ref()
            .and_then(|conns| conns.buffer_to_id(buffer_id))
    }

    #[property]
    pub fn connection(&self) -> Option<ConnectionApiObject> {
        if let Some(id) = self.connection_id() {
            Some(ConnectionApiObject::new(self.api.clone(), id))
        } else {
            None
        }
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
        replacement: Either<String, ScriptingFnRef>,
    ) -> KeyResult {
        let scripting = context.state().scripting.clone();
        if let Some(ref mut conns) = context.state_mut().connections {
            conns.with_buffer_engine(id, move |engine| match replacement {
                Either::A(text) => engine.aliases.insert_text(pattern, text),
                Either::B(f) => engine
                    .aliases
                    .insert_fn(pattern, create_user_processor(scripting, f)),
            })?;
        }
        Ok(())
    }
}

fn create_user_processor(
    scripting: Arc<Mutex<ScriptingManager>>,
    f: ScriptingFnRef,
) -> Box<dyn Fn(HashMap<String, String>) -> KeyResult<Option<String>> + Send> {
    Box::new(move |groups| match scripting.try_lock() {
        Ok(scripting) => match scripting.invoke(f, groups.into())? {
            FnArgs::None => Ok(None),
            FnArgs::String(s) => Ok(Some(s)),
            _ => Err(KeyError::InvalidInput(
                "Returned an unexpected value".to_string(),
            )),
        },

        Err(_) => Err(KeyError::IO(std::io::ErrorKind::WouldBlock.into())),
    })
}
