use std::sync::{mpsc, Arc, Mutex};

mod buffer;
pub mod core;
mod current;

use crate::input::{commands::CommandHandlerContext, maps::KeyResult, KeyError};

use super::fns::FnManager;

const MAX_TASKS_PER_TICK: u16 = 10;

pub trait ApiHandler<Payload: Clone + Send + Sync, Response: Clone + Send + Sync> {
    fn handle(&self, context: &mut CommandHandlerContext, p: Payload) -> KeyResult<Response>;
}

trait ApiRpcCall: Send {
    fn handle(&self, context: &mut CommandHandlerContext);
}

struct ApiMessage<Payload, Response, Handler>
where
    Payload: Clone + Send + Sync,
    Response: Clone + Send + Sync,
    Handler: ApiHandler<Payload, Response> + Send + Sync,
{
    handler: Handler,
    payload: Payload,
    response: mpsc::Sender<KeyResult<Response>>,
}

// NOTE: the goal here is that each Module can declare its messages and
// handler of the messages in isolation, so we're essentially passing
// a closure that operates on the CommandHandlerContext
impl<Payload, Response, Handler> ApiRpcCall for ApiMessage<Payload, Response, Handler>
where
    Payload: Clone + Send + Sync,
    Response: 'static + Clone + Send + Sync,
    Handler: ApiHandler<Payload, Response> + Send + Sync,
{
    fn handle(&self, context: &mut CommandHandlerContext) {
        // TODO: can we just pass a reference instead of cloning?
        let result = self.handler.handle(context, self.payload.clone());
        if let Err(e) = self.response.send(result) {
            std::panic::panic_any(e);
        }
    }
}

pub struct ApiManagerRpc {
    to_app: Arc<Mutex<mpsc::Sender<Box<dyn ApiRpcCall>>>>,
    from_script: mpsc::Receiver<Box<dyn ApiRpcCall>>,
}

impl Default for ApiManagerRpc {
    fn default() -> Self {
        let (to_app, from_script) = mpsc::channel();
        Self {
            to_app: Arc::new(Mutex::new(to_app)),
            from_script,
        }
    }
}

impl ApiManagerRpc {
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
                    msg.handle(context);
                    dirty = true;
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(dirty),
                Err(e) => std::panic::panic_any(e),
            }
        }

        Ok(dirty)
    }
}

#[derive(Clone)]
pub struct ApiManagerDelegate {
    to_app: Arc<Mutex<mpsc::Sender<Box<dyn ApiRpcCall>>>>,
}

impl ApiManagerDelegate {
    pub fn perform<Payload, Response, Handler>(
        &self,
        handler: Handler,
        payload: Payload,
    ) -> KeyResult<Response>
    where
        Payload: 'static + Clone + Send + Sync,
        Response: 'static + Clone + Send + Sync,
        Handler: 'static + ApiHandler<Payload, Response> + Send + Sync,
    {
        let (tx, rx) = mpsc::channel();
        let message = Box::new(ApiMessage {
            handler,
            payload,
            response: tx,
        });

        if let Ok(tx) = self.to_app.lock() {
            if let Err(_) = tx.send(message) {
                return Err(KeyError::Interrupted);
            }
        } else {
            panic!("Failed to lock to_app RPC sender");
        }

        match rx.recv() {
            Ok(response) => response,
            Err(_) => {
                return Err(KeyError::Interrupted);
            }
        }
    }
}

pub type Api = ApiManagerDelegate;
pub type Fns = Arc<Mutex<FnManager>>;
