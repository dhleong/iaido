use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::sync::{
    mpsc,
    oneshot::{self, error::TryRecvError},
};
use url::Url;

use crate::{
    app::{
        self,
        jobs::{JobContext, JobRecord, Jobs},
    },
    editing::{ids::Ids, text::TextLines, Id},
    game::engine::GameEngine,
};

use super::{game::GameConnection, transport::Transport, Connection, ConnectionFactories};

const DEFAULT_LINES_PER_REDRAW: u16 = 10;

struct ConnectionRecord {
    stop_read_signal: Option<oneshot::Sender<()>>,
    outgoing: mpsc::UnboundedSender<String>,
}

impl Drop for ConnectionRecord {
    fn drop(&mut self) {
        if let Some(signal) = self.stop_read_signal.take() {
            signal.send(()).ok();
        }
    }
}

impl ConnectionRecord {
    pub fn send(&mut self, message: String) -> io::Result<()> {
        match self.outgoing.send(message) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::ErrorKind::NotConnected.into()),
        }
    }
}

#[derive(Default)]
pub struct Connections {
    ids: Ids,
    all: Vec<GameConnection>,
    connection_to_buffer: HashMap<Id, Id>,

    // NOTE: More than one Buffer may be associated with a Connection
    // here (for example, the input buffer) but a Connection will
    // be associated with ONLY ONE Buffer above (IE: the buffer it
    // writes to)
    buffer_to_connection: HashMap<Id, Id>,
    factories: ConnectionFactories,

    buffer_engines: HashMap<Id, GameEngine>,
}

impl Connections {
    pub fn by_id(&self, id: Id) -> Option<&GameConnection> {
        self.all.iter().find(|conn| conn.id() == id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut GameConnection> {
        self.all.iter_mut().find(|conn| conn.id() == id)
    }

    pub fn buffer_to_id(&self, buffer_id: Id) -> Option<Id> {
        self.buffer_to_connection.get(&buffer_id).cloned()
    }

    pub fn id_to_buffer(&self, id: Id) -> Option<Id> {
        self.connection_to_buffer.get(&id).cloned()
    }

    pub fn id_to_buffers(&self, id: Id) -> Vec<Id> {
        self.buffer_to_connection
            .iter()
            .filter_map(|(buff_id, conn_id)| {
                if *conn_id == id {
                    Some(buff_id.to_owned())
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn by_buffer_id(&mut self, buffer_id: Id) -> Option<&mut GameConnection> {
        if let Some(conn_id) = self.buffer_to_id(buffer_id) {
            self.by_id_mut(conn_id)
        } else {
            None
        }
    }

    pub fn with_buffer_engine<R>(
        &mut self,
        buffer_id: Id,
        callback: impl FnOnce(&mut GameEngine) -> R,
    ) -> R {
        if let Some(conn) = self.by_buffer_id(buffer_id) {
            callback(&mut conn.game)
        } else {
            let game = self.buffer_engines.entry(buffer_id).or_default();
            callback(game)
        }
    }

    /// Asynchronously create a new connection attached to the given buffer_id (and
    /// input_buffer_id). Returns a JobRecord for joining on the request
    pub fn create_async(
        &mut self,
        jobs: &mut Jobs,
        buffer_id: Id,
        input_buffer_id: Id,
        uri: Url,
    ) -> JobRecord {
        let id = self.ids.next();
        let factory = self.factories.clone();
        jobs.start(move |ctx| async move {
            let connection = Mutex::new(factory.create(id, uri)?);

            ctx.run(move |state| {
                state.connections.as_mut().unwrap().add(
                    buffer_id,
                    input_buffer_id,
                    connection.into_inner().unwrap(),
                );
                Ok(())
            })
        })
    }

    fn launch(
        ctx: JobContext,
        connection: Arc<Mutex<Box<dyn Transport + Send>>>,
    ) -> ConnectionRecord {
        let buffer_id = 0usize; // TODO
        let readable = connection.clone();
        let (tx_signal, mut rx_signal) = oneshot::channel::<()>();
        tokio::task::spawn_blocking(move || {
            loop {
                match rx_signal.try_recv() {
                    Err(TryRecvError::Empty) => {} // Nop
                    _ => break, // Any other message, we should drop the connection
                }

                let mut conn = readable.lock().unwrap();
                let read = conn.read_timeout(Duration::from_millis(250));
                let record = ctx.spawn(move |state| {
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
                });

                if !record.join().expect("spawn error") {
                    break; // Disconnected
                }
            }
        });

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let writable = connection.clone();
        tokio::spawn(async move {
            while let Some(to_send) = rx.recv().await {
                let mut conn = writable.lock().unwrap();
                conn.send(&to_send); // TODO
            }
        });

        ConnectionRecord {
            stop_read_signal: Some(tx_signal),
            outgoing: tx,
        }
    }

    /// Returns the associated buffer ID
    pub fn disconnect(&mut self, connection_id: Id) -> io::Result<Id> {
        if let Some(index) = self.all.iter().position(|conn| conn.id() == connection_id) {
            self.all.swap_remove(index);
            return Ok(self
                .connection_to_buffer
                .remove(&connection_id)
                .expect("No buffer associated with connection"));
        }

        Err(io::Error::new(
            io::ErrorKind::NotConnected,
            format!("c{}: Connection not found", connection_id),
        ))
    }

    /// As for `disconnect`, but accepts a Buffer Id instead of a Connection Id
    pub fn disconnect_buffer(&mut self, buffer_id: Id) -> io::Result<Id> {
        if let Some(connection_id) = self.buffer_to_connection.remove(&buffer_id) {
            return self.disconnect(connection_id);
        }

        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("#{}: No connection for buffer", buffer_id),
        ))
    }

    pub fn process(&mut self, app: &mut app::State) -> bool {
        let to_buffer = &mut self.connection_to_buffer;
        let mut any_updated = false;
        retain(&mut self.all, |conn| {
            let buffer_id = to_buffer[&conn.id()];
            let mut winsbuf = app
                .winsbuf_by_id(buffer_id)
                .expect("Could not find buffer for connection");
            let lines_per_redraw = winsbuf
                .windows
                .iter()
                .map(|win| win.size.h)
                .max()
                .unwrap_or(DEFAULT_LINES_PER_REDRAW);

            for _ in 0..lines_per_redraw {
                match conn.read() {
                    Ok(None) => break, // nop
                    Ok(Some(value)) => {
                        any_updated = true;

                        if let Some(processed) = conn.game.process_received(value) {
                            winsbuf.append_value(processed);
                        }
                    }
                    Err(e) => {
                        any_updated = true;
                        winsbuf.append(TextLines::from(format!("Disconnected: {}", e)));
                        to_buffer.remove(&conn.id());
                        return RetainAction::Remove;
                    }
                }
            }

            // keep the conn, by default
            return RetainAction::Keep;
        });
        any_updated
    }

    fn add(&mut self, buffer_id: Id, input_buffer_id: Id, connection: Box<dyn Connection>) {
        self.connection_to_buffer.insert(connection.id(), buffer_id);
        self.buffer_to_connection.insert(buffer_id, connection.id());
        self.buffer_to_connection
            .insert(input_buffer_id, connection.id());

        let with_game = if let Some(engine) = self.buffer_engines.remove(&buffer_id) {
            GameConnection::with_engine(connection, engine)
        } else {
            connection.into()
        };
        self.all.push(with_game);
    }

    #[cfg(test)]
    pub fn add_for_test(&mut self, buffer_id: Id, connection: Box<dyn Connection>) {
        self.add(buffer_id, buffer_id, connection);
    }
}

#[derive(PartialEq)]
enum RetainAction {
    Remove,
    Keep,
}

// mutating, partitioning iteration. order is not preserved
fn retain<T, F>(v: &mut Vec<T>, mut pred: F)
where
    F: FnMut(&mut T) -> RetainAction,
{
    if v.is_empty() {
        // nop
        return;
    }

    let mut i = 0;
    let mut end = v.len();
    loop {
        // invariants:
        // items v[0..end] will be kept
        // items v[j..i] will be removed

        if pred(&mut v[i]) == RetainAction::Remove {
            v.swap(i, end - 1);
            end -= 1;
        } else {
            i += 1;
        }

        if i >= end {
            break;
        }
    }
    v.truncate(end);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod retain {
        use super::*;

        #[test]
        fn touches_all_items() {
            let mut v = vec![1, 2, 3, 4];
            let mut sum = 0;
            retain(&mut v, |val| {
                sum += *val;
                RetainAction::Keep
            });
            assert_eq!(sum, 10);
        }

        #[test]
        fn visits_all_after_remove() {
            let mut v = vec![1, 2, 3, 4];
            let mut sum = 0;
            retain(&mut v, |val| {
                sum += val.to_owned();
                if *val == 2 {
                    RetainAction::Remove
                } else {
                    RetainAction::Keep
                }
            });
            assert_eq!(sum, 10);
            assert_eq!(v.len(), 3);
        }

        #[test]
        fn remove_all_safely() {
            let mut v = vec![1, 2, 3, 4];
            let mut sum = 0;
            retain(&mut v, |val| {
                sum += val.to_owned();
                RetainAction::Remove
            });
            assert_eq!(sum, 10);
            assert_eq!(v.len(), 0);
        }
    }
}
