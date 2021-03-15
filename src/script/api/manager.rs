use std::sync::{mpsc, Arc, Mutex};

use crate::{
    app,
    input::{maps::KeyResult, KeyError},
};

use super::{ApiDelegate, ApiRequest, ApiResult};

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

    pub fn process(&mut self, app: &mut app::State) -> Result<bool, KeyError> {
        let mut dirty = false;
        for _ in 0..MAX_TASKS_PER_TICK {
            match self.from_script.try_recv() {
                Ok(msg) => {
                    self.process_one(app, msg)?;
                    dirty = true;
                }
                Err(mpsc::TryRecvError::Empty) => return Ok(dirty),
                Err(e) => panic!(e),
            }
        }

        Ok(dirty)
    }

    fn process_one(&self, app: &mut app::State, msg: ApiMessage<ApiRequest>) -> KeyResult {
        match msg.payload {
            ApiRequest::Echo(msg) => {
                app.echo(msg.into());
            }
        }

        match msg.response.send(Ok(())) {
            Err(e) => panic!(e),
            Ok(_) => {}
        }

        Ok(())
    }
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
