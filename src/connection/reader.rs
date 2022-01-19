use std::{thread::yield_now, time::Duration};

use tokio::sync::oneshot::{self, error::TryRecvError, Sender};

use crate::{app::jobs::JobContext, editing::Id};

use super::transport::Transport;

pub struct StopSignal {
    tx: Option<Sender<()>>,
}

impl StopSignal {
    pub fn new() -> (Self, oneshot::Receiver<()>) {
        let (tx, rx) = oneshot::channel();
        (StopSignal { tx: Some(tx) }, rx)
    }

    pub fn stop(&mut self) {
        if let Some(tx) = self.tx.take() {
            // If it failed, then we probably already stopped reading
            tx.send(()).ok();
        }
    }
}

impl Drop for StopSignal {
    fn drop(&mut self) {
        self.stop();
    }
}

pub struct TransportReader<T: Transport> {
    ctx: JobContext,
    buffer_id: Id,
    transport: T,
}

impl<T: Transport + Send + 'static> TransportReader<T> {
    pub fn spawn(ctx: JobContext, id: Id, buffer_id: Id, transport: T) -> StopSignal {
        let (signal, rx) = StopSignal::new();

        tokio::task::spawn_blocking(move || {
            let mut reader = TransportReader {
                ctx,
                buffer_id,
                transport,
            };
            reader.loop_until(rx);
            reader
                .ctx
                .spawn(move |ctx| {
                    ctx.connections.as_mut().unwrap().disconnect(id).ok();
                })
                .join()
                .ok();
        });

        signal
    }

    pub fn loop_until(&mut self, mut signal: oneshot::Receiver<()>) {
        loop {
            match signal.try_recv() {
                Err(TryRecvError::Empty) => {} // Nop
                _ => break,                    // Any other message, we should drop the connection
            }

            if !self.read_once() {
                break;
            }

            yield_now();
        }
    }

    fn read_once(&mut self) -> bool {
        let read = self.transport.read_timeout(Duration::from_millis(250));
        let result = match read {
            Ok(None) => {
                // Nothing read
                return true;
            }
            Ok(_) => true,
            Err(_) => false,
        };

        let buffer_id = self.buffer_id;
        self.ctx.run(move |state| {
            let mut buffer = state
                .winsbuf_by_id(buffer_id)
                .expect("Could not find buffer for connection");
            match read {
                Ok(Some(value)) => buffer.append_value(value),
                Ok(None) => (), // nop
                Err(e) => buffer.append(format!("Disconnected: {}", e).into()),
            };
        });

        return result;
    }
}
