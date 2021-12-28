use std::{collections::HashMap, fmt::Debug, io};

use crate::{
    input::{
        commands::{CommandHandler, CommandHandlerContext},
        maps::{prompt::PromptConfig, KeyResult},
        KeyError, KeymapContext,
    },
    script::{args::FnArgs, fns::ScriptingFnRef},
};

use super::{Api, Fns};

#[apigen::ns]
#[derive(Clone)]
pub struct ScriptUi {
    api: Api,
    fns: Fns,
}

impl Debug for ScriptUi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<iaido.ui>")
    }
}

#[apigen::ns_impl(module)]
impl ScriptUi {
    pub fn new(api: Api, fns: Fns) -> Self {
        Self { api, fns }
    }

    #[rpc]
    pub fn input(
        context: &mut CommandHandlerContext,
        config: HashMap<String, FnArgs>,
        on_confirm: ScriptingFnRef,
    ) -> KeyResult {
        let handler: Box<CommandHandler> = Box::new(move |ctx| {
            let s = ctx.input.to_string();
            match ctx.state().scripting.try_lock() {
                Ok(scripting) => {
                    scripting.invoke(on_confirm, FnArgs::String(s))?;
                    Ok(())
                }
                Err(_) => Err(KeyError::IO(io::ErrorKind::WouldBlock.into())),
            }
        });

        let config = PromptConfig {
            prompt: match config.get("prompt") {
                Some(FnArgs::String(s)) => s.to_string(),
                _ => ">".to_string(),
            },
            history_key: "@".to_string(),
            handler,
            completer: None,
        };

        context.state_mut().clear_echo();
        context
            .state_mut()
            .prompt
            .activate(config.prompt.clone().into());
        context.keymap.prompt(config);
        Ok(())
    }
}
