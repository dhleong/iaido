// Allow dead code in this module, in case all languages are disabled:
#![allow(dead_code)]

use std::sync::{Arc, Mutex};

mod buffer;
mod connection;
pub mod core;
mod current;
mod window;

use crate::{
    app::{self, dispatcher::DispatchSender},
    input::{commands::CommandHandlerContext, maps::KeyResult},
};

use super::fns::FnManager;

pub trait ApiHandler<Payload: Send + Sync, Response: Send + Sync> {
    fn handle(&self, context: &mut CommandHandlerContext, p: Payload) -> KeyResult<Response>;
}

trait ApiRpcCall: Send {
    fn handle(&mut self, context: &mut CommandHandlerContext);
}

struct ApiMessage<Payload, Response, Handler>
where
    Payload: Send + Sync,
    Response: Send + Sync,
    Handler: ApiHandler<Payload, Response> + Send + Sync,
{
    handler: Handler,

    /// The Payload is Option because it will be consumed when handled
    payload: Option<Payload>,

    /// The Response is also Option; it will be filled once handle is
    /// successfully called
    response: Option<KeyResult<Response>>,
}

// NOTE: the goal here is that each Module can declare its messages and
// handler of the messages in isolation, so we're essentially passing
// a closure that operates on the CommandHandlerContext
impl<Payload, Response, Handler> ApiRpcCall for ApiMessage<Payload, Response, Handler>
where
    Payload: Send + Sync,
    Response: 'static + Send + Sync,
    Handler: ApiHandler<Payload, Response> + Send + Sync,
{
    fn handle(&mut self, context: &mut CommandHandlerContext) {
        let result = self
            .handler
            .handle(context, self.payload.take().expect("No payload provided"));
        self.response = Some(result);
    }
}

#[derive(Clone)]
pub struct ApiDelegate {
    dispatcher: Arc<Mutex<DispatchSender>>,
}

impl From<&app::State> for ApiDelegate {
    fn from(state: &app::State) -> Self {
        Self {
            dispatcher: Arc::new(Mutex::new(state.dispatcher.sender.clone())),
        }
    }
}

impl ApiDelegate {
    pub fn perform<Payload, Response, Handler>(
        &self,
        handler: Handler,
        payload: Payload,
    ) -> KeyResult<Response>
    where
        Payload: 'static + Clone + Send + Sync,
        Response: 'static + Send + Sync,
        Handler: 'static + ApiHandler<Payload, Response> + Send + Sync,
    {
        let mut message = ApiMessage {
            handler,
            payload: Some(payload),
            response: None,
        };

        let mut message_result = self
            .dispatcher
            .lock()
            .expect("Could not lock dispatcher")
            .spawn_command(move |ctx| {
                message.handle(ctx);
                message
            })
            .join()?; // TODO join_interruptably?

        message_result
            .response
            .take()
            .expect("Did not receive response")
    }
}

pub type Api = ApiDelegate;
pub type Fns = Arc<Mutex<FnManager>>;
