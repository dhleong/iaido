use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::oneshot::{self, error::TryRecvError, Sender};

use crate::{app::jobs::JobContext, editing::Id};

use super::transport::Transport;

pub struct StopSignal {
    tx: Option<Sender<()>>,
}

impl StopSignal {
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

pub struct TransportReader {
    ctx: JobContext,
    buffer_id: Id,
    transport: Arc<Mutex<Box<dyn Transport + Send>>>,
}

impl TransportReader {
    pub fn spawn(
        ctx: JobContext,
        buffer_id: Id,
        transport: Arc<Mutex<Box<dyn Transport + Send>>>,
    ) -> StopSignal {
        let (tx, rx) = oneshot::channel();

        tokio::task::spawn_blocking(move || {
            let mut reader = TransportReader {
                ctx,
                buffer_id,
                transport,
            };
            reader.loop_until(rx);
        });

        StopSignal { tx: Some(tx) }
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
        }
    }

    fn read_once(&mut self) -> bool {
        let mut conn = self.transport.lock().unwrap();
        let read = conn.read_timeout(Duration::from_millis(250));
        if let Ok(None) = read {
            // Nothing read
            return true;
        }

        let buffer_id = self.buffer_id;
        self.ctx
            .spawn(move |state| {
                let mut buffer = state
                    .winsbuf_by_id(buffer_id)
                    .expect("Could not find buffer for connection");
                match read {
                    Ok(Some(value)) => buffer.append_value(value),
                    Ok(None) => {} // nop
                    Err(e) => {
                        buffer.append(format!("Disconnected: {}", e).into());
                        return false;
                    }
                }
                true
            })
            .join()
            .expect("Spawn error")
    }
}
