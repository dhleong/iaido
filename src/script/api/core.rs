use std::{fmt, io};

use crate::{
    editing::Id,
    input::{
        commands::CommandHandlerContext, keys::KeysParsable, maps::UserKeyHandler, KeymapContext,
        RemapMode,
    },
    script::fns::ScriptingFnRef,
};

use super::{Api, Fns};

#[apigen::ns]
#[derive(Clone)]
pub struct IaidoCore {
    api: Api,
    fns: Fns,
}

#[apigen::ns_impl(module)]
impl IaidoCore {
    pub fn new(api: Api, fns: Fns) -> Self {
        Self { api, fns }
    }

    #[property]
    pub fn current(&self) -> CurrentObjects {
        CurrentObjects::new(self.api.clone())
    }

    #[rpc]
    pub fn echo(context: &mut CommandHandlerContext, text: String) {
        context.state_mut().echom(text);
    }

    #[rpc]
    pub fn set_keymap(
        context: &mut CommandHandlerContext,
        mode: String,
        keys: String,
        f: ScriptingFnRef,
    ) {
        let mode = match mode.as_str() {
            "n" => RemapMode::VimNormal,
            "i" => RemapMode::VimInsert,
            _ => RemapMode::User(mode),
        };
        context
            .keymap
            .remap_keys_user_fn(mode, keys.into_keys(), create_user_keyhandler(f));
    }
}

impl fmt::Debug for IaidoCore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<iaido>")
    }
}

fn create_user_keyhandler(f: ScriptingFnRef) -> Box<UserKeyHandler> {
    Box::new(move |mut ctx| {
        let scripting = ctx.state().scripting.clone();
        ctx.state_mut()
            .jobs
            .start(move |_| async move {
                match scripting.try_lock() {
                    Ok(scripting) => {
                        scripting.invoke(f)?;
                        Ok(())
                    }
                    Err(_) => Err(io::ErrorKind::WouldBlock.into()),
                }
            })
            .join_interruptably(&mut ctx)
    })
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
