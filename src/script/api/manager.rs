use std::{
    io,
    sync::{mpsc, Arc, Mutex},
};

use crate::{
    app,
    input::{
        keys::KeysParsable,
        maps::{KeyResult, UserKeyHandler},
        BoxableKeymap, KeyError, KeymapContext, RemapMode,
    },
};

use super::{core::ScriptingFnRef, ApiDelegate, ApiRequest, ApiResult};

const MAX_TASKS_PER_TICK: u16 = 10;

struct ApiMessage<T: Send + Sync> {
    payload: T,
    response: mpsc::Sender<ApiResult>,
}

pub struct ApiManager {
    to_app: Arc<Mutex<mpsc::Sender<ApiMessage<ApiRequest>>>>,
    from_script: mpsc::Receiver<ApiMessage<ApiRequest>>,
}

impl Default for ApiManager {
    fn default() -> Self {
        let (to_app, from_script) = mpsc::channel::<ApiMessage<ApiRequest>>();
        Self {
            to_app: Arc::new(Mutex::new(to_app)),
            from_script,
        }
    }
}

impl ApiManager {
    pub fn delegate(&self) -> ApiManagerDelegate {
        ApiManagerDelegate {
            to_app: self.to_app.clone(),
        }
    }

    pub fn process<K: BoxableKeymap>(
        &mut self,
        app: &mut app::State,
        keymap: &mut K,
    ) -> Result<bool, KeyError> {
        let mut dirty = false;
        for _ in 0..MAX_TASKS_PER_TICK {
            match self.from_script.try_recv() {
                Ok(msg) => {
                    self.process_one(app, keymap, msg)?;
                    dirty = true;
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(dirty),
                Err(e) => panic!(e),
            }
        }

        Ok(dirty)
    }

    fn process_one<K: BoxableKeymap>(
        &self,
        app: &mut app::State,
        keymap: &mut K,
        msg: ApiMessage<ApiRequest>,
    ) -> KeyResult {
        match msg.payload {
            ApiRequest::Echo(text) => {
                app.echo(text.into());
            }

            ApiRequest::SetKeymapFn(mode, keys, f) => {
                // TODO store this... somewhere
                let mode = match mode.as_str() {
                    "n" => RemapMode::VimNormal,
                    "i" => RemapMode::VimInsert,
                    _ => RemapMode::User(mode),
                };
                keymap.remap_keys_user_fn(mode, keys.into_keys(), create_user_keyhandler(f));
            }
        }

        match msg.response.send(Ok(())) {
            Err(e) => panic!(e),
            Ok(_) => {}
        }

        Ok(())
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

pub struct ApiManagerDelegate {
    to_app: Arc<Mutex<mpsc::Sender<ApiMessage<ApiRequest>>>>,
}

impl ApiDelegate for ApiManagerDelegate {
    fn perform(&self, request: ApiRequest) -> ApiResult {
        let (tx, rx) = mpsc::channel();
        let message = ApiMessage {
            payload: request,
            response: tx,
        };

        if let Ok(tx) = self.to_app.lock() {
            if let Err(_) = tx.send(message) {
                return Err(KeyError::Interrupted);
            }
        } else {
            panic!();
        }

        match rx.recv() {
            Ok(response) => response,
            Err(_) => {
                return Err(KeyError::Interrupted);
            }
        }
    }
}
