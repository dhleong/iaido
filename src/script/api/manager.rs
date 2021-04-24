use std::{
    io,
    sync::{mpsc, Arc, Mutex},
};

use crate::input::{
    commands::CommandHandlerContext,
    keys::KeysParsable,
    maps::{KeyResult, UserKeyHandler},
    BoxableKeymap, KeyError, KeymapContext, RemapMode,
};

use super::{core::ScriptingFnRef, ApiDelegate, ApiRequest, ApiResponse, ApiResult, IdType};

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

    pub fn process(&mut self, context: &mut CommandHandlerContext) -> Result<bool, KeyError> {
        let mut dirty = false;
        for _ in 0..MAX_TASKS_PER_TICK {
            match self.from_script.try_recv() {
                Ok(msg) => {
                    self.process_one(context, msg)?;
                    dirty = true;
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(dirty),
                Err(e) => std::panic::panic_any(e),
            }
        }

        Ok(dirty)
    }

    fn process_one(
        &self,
        context: &mut CommandHandlerContext,
        msg: ApiMessage<ApiRequest>,
    ) -> KeyResult {
        let mut response = Ok(None);
        match msg.payload {
            ApiRequest::BufferName(id) => {
                response = Ok(if let Some(buf) = context.state().buffers.by_id(id) {
                    Some(ApiResponse::String(format!("{:?}", buf.source())))
                } else {
                    None
                })
            }

            ApiRequest::CurrentId(id_type) => {
                response = Ok(match id_type {
                    IdType::Buffer => Some(context.state().current_buffer().id()),
                    IdType::Connection => context.state().connections.as_ref().and_then(|conns| {
                        conns.buffer_to_id(context.state().current_buffer().id())
                    }),
                    IdType::Window => Some(context.state().current_window().id),
                    IdType::Tab => Some(context.state().current_tab().id),
                }
                .and_then(|id| Some(ApiResponse::Id(id))));
            }

            ApiRequest::Echo(text) => {
                context.state_mut().echom(text.to_string());
            }

            ApiRequest::SetCurrentId(id_type, id) => match id_type {
                IdType::Buffer => {
                    context.state_mut().current_window_mut().buffer = id;
                }

                // TODO implement these?
                IdType::Connection => {}
                IdType::Window => {}
                IdType::Tab => {}
            },

            ApiRequest::SetKeymapFn(mode, keys, f) => {
                let mode = match mode.as_str() {
                    "n" => RemapMode::VimNormal,
                    "i" => RemapMode::VimInsert,
                    _ => RemapMode::User(mode),
                };
                context.keymap.remap_keys_user_fn(
                    mode,
                    keys.into_keys(),
                    create_user_keyhandler(f),
                );
            }
        }

        match msg.response.send(response) {
            Err(e) => std::panic::panic_any(e),
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

#[derive(Clone)]
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
