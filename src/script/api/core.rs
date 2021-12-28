use std::{collections::HashMap, fmt, io};

use crate::{
    editing::Id,
    input::{
        commands::{connection, CommandHandlerContext},
        keys::KeysParsable,
        maps::{user_key_handler, KeyResult, UserKeyHandler},
        KeymapConfig, KeymapContext, RemapMode,
    },
    script::{args::FnArgs, fns::ScriptingFnRef, poly::Either},
};

use super::{current::CurrentObjects, ui::ScriptUi, Api, Fns};

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
        CurrentObjects::new(self.api.clone(), self.fns.clone())
    }

    #[property]
    pub fn ui(&self) -> ScriptUi {
        ScriptUi::new(self.api.clone(), self.fns.clone())
    }

    #[rpc]
    pub fn connect(context: &mut CommandHandlerContext, url: String) -> KeyResult {
        connection::connect(context, url)
    }

    #[rpc]
    pub fn echo(context: &mut CommandHandlerContext, text: String) {
        context.state_mut().echom(text);
    }

    #[rpc]
    pub fn feedkeys(context: &mut CommandHandlerContext, keys: String, mode: String) -> KeyResult {
        let keys = keys.into_keys();
        let allow_remap = mode.find("n").is_none();
        context.feed_keys(keys, KeymapConfig { allow_remap })
    }

    #[rpc]
    pub fn buf_set_keymap(
        context: &mut CommandHandlerContext,
        buffer_id: Id,
        mode: String,
        keys: String,
        mapping: Either<String, ScriptingFnRef>,
        opts: Option<HashMap<String, FnArgs>>,
    ) {
        context.keymap.buf_remap_keys_user_fn(
            buffer_id,
            parse_mode(mode),
            keys.into_keys(),
            keyhandler(mapping, parse_keymap_config(opts)),
        )
    }

    #[rpc]
    pub fn set_keymap(
        context: &mut CommandHandlerContext,
        mode: String,
        keys: String,
        mapping: Either<String, ScriptingFnRef>,
        opts: Option<HashMap<String, FnArgs>>,
    ) {
        context.keymap.remap_keys_user_fn(
            parse_mode(mode),
            keys.into_keys(),
            keyhandler(mapping, parse_keymap_config(opts)),
        )
    }
}

fn keyhandler(
    mapping: Either<String, ScriptingFnRef>,
    config: KeymapConfig,
) -> Box<UserKeyHandler> {
    match mapping {
        Either::A(to_keys) => user_key_handler(to_keys.into_keys(), config),
        Either::B(f) => create_user_keyhandler(f),
    }
}

fn parse_keymap_config(config: Option<HashMap<String, FnArgs>>) -> KeymapConfig {
    if let Some(config) = config {
        let allow_remap = if let Some(FnArgs::Bool(noremap)) = config.get("noremap") {
            !noremap
        } else {
            true
        };
        KeymapConfig { allow_remap }
    } else {
        KeymapConfig::default()
    }
}

fn parse_mode(mode: String) -> RemapMode {
    match mode.as_str() {
        "n" => RemapMode::VimNormal,
        "i" => RemapMode::VimInsert,
        _ => RemapMode::User(mode),
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
                        scripting.invoke(f, FnArgs::None)?;
                        Ok(())
                    }
                    Err(_) => Err(io::ErrorKind::WouldBlock.into()),
                }
            })
            .join_interruptably(&mut ctx)
    })
}
