use std::{collections::HashMap, io, sync::Mutex};

use tokio::sync::mpsc;
use url::Url;

use crate::{
    app::jobs::{JobContext, JobRecord, Jobs},
    editing::{ids::Ids, Id},
    game::engine::GameEngine,
};

use super::{
    game::GameConnection,
    reader::{StopSignal, TransportReader},
    transport::Transport,
    TransportFactories,
};

pub struct ConnectionRecord {
    pub id: Id,
    #[allow(unused)]
    stop_read_signal: StopSignal,
    outgoing: mpsc::UnboundedSender<String>,
    outgoing_results: std::sync::mpsc::Receiver<io::Result<()>>,
    connection: GameConnection,
}

impl ConnectionRecord {
    pub fn send(&mut self, message: String) -> io::Result<()> {
        match self.outgoing.send(message) {
            Ok(_) => self.outgoing_results.recv().unwrap_or(Ok(())),
            Err(_) => Err(io::ErrorKind::NotConnected.into()),
        }
    }

    pub fn with_engine<R, F: FnOnce(&GameEngine) -> R>(&self, f: F) -> R {
        let game = self.connection.game.lock().unwrap();
        f(&game)
    }

    pub fn with_engine_mut<R, F: FnOnce(&mut GameEngine) -> R>(&mut self, f: F) -> R {
        let mut game = self.connection.game.lock().unwrap();
        f(&mut game)
    }
}

#[derive(Default)]
pub struct Connections {
    ids: Ids,
    by_id: HashMap<Id, ConnectionRecord>,
    connection_to_buffer: HashMap<Id, Id>,

    // NOTE: More than one Buffer may be associated with a Connection
    // here (for example, the input buffer) but a Connection will
    // be associated with ONLY ONE Buffer above (IE: the buffer it
    // writes to)
    buffer_to_connection: HashMap<Id, Id>,
    factories: TransportFactories,

    buffer_engines: HashMap<Id, GameEngine>,
}

impl Connections {
    pub fn by_id(&self, id: Id) -> Option<&ConnectionRecord> {
        self.by_id.get(&id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut ConnectionRecord> {
        self.by_id.get_mut(&id)
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

    pub fn by_buffer_id(&mut self, buffer_id: Id) -> Option<&mut ConnectionRecord> {
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
            conn.with_engine_mut(callback)
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
            let connection = Mutex::new(factory.create(uri)?);
            let transport_context = Mutex::new(ctx.clone());

            ctx.run(move |state| {
                state.connections.add_transport(
                    transport_context.into_inner().unwrap(),
                    id,
                    buffer_id,
                    input_buffer_id,
                    connection.into_inner().unwrap(),
                );
            });

            Ok(())
        })
    }

    fn launch(
        id: Id,
        ctx: JobContext,
        buffer_id: Id,
        connection: GameConnection,
    ) -> ConnectionRecord {
        let stop_read_signal = TransportReader::spawn(ctx, id, buffer_id, connection.clone());

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let (result_tx, result_rx) = std::sync::mpsc::channel();
        let mut writable = connection.clone();
        tokio::spawn(async move {
            while let Some(to_send) = rx.recv().await {
                let result = writable.send(&to_send);
                result_tx.send(result).ok();
            }
        });

        ConnectionRecord {
            id,
            stop_read_signal,
            outgoing: tx,
            outgoing_results: result_rx,
            connection,
        }
    }

    /// Returns the associated buffer ID
    pub fn disconnect(&mut self, connection_id: Id) -> io::Result<Id> {
        if self.by_id.remove(&connection_id).is_some() {
            let buffer = self
                .connection_to_buffer
                .remove(&connection_id)
                .expect("No buffer associated with connection");

            // Clean up all buffer->conn mappings
            self.buffer_to_connection
                .retain(|_buf_id, conn_id| *conn_id != connection_id);

            return Ok(buffer);
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

    fn add_transport(
        &mut self,
        ctx: JobContext,
        id: Id,
        buffer_id: Id,
        input_buffer_id: Id,
        transport: Box<dyn Transport + Send>,
    ) {
        let engine = self
            .buffer_engines
            .remove(&buffer_id)
            .unwrap_or_else(|| GameEngine::default());

        let transport = GameConnection::with_engine(transport, engine);

        let record = Connections::launch(id, ctx.clone(), buffer_id, transport);

        self.add_record(id, buffer_id, input_buffer_id, record);
    }

    fn add_record(&mut self, id: Id, buffer_id: Id, input_buffer_id: Id, record: ConnectionRecord) {
        self.connection_to_buffer.insert(id, buffer_id);
        self.buffer_to_connection.insert(buffer_id, id);
        self.buffer_to_connection.insert(input_buffer_id, id);
        self.by_id.insert(id, record);
    }

    #[cfg(test)]
    pub fn add_for_test(&mut self, buffer_id: Id, transport: Box<dyn Transport + Send>) {
        let (stop_read_signal, _) = StopSignal::new();
        let (outgoing, _) = mpsc::unbounded_channel();
        let (_, outgoing_results) = std::sync::mpsc::channel();
        self.add_record(
            0,
            buffer_id,
            0,
            ConnectionRecord {
                id: 0,
                stop_read_signal,
                outgoing,
                outgoing_results,
                connection: GameConnection::with_engine(transport, Default::default()),
            },
        );
    }
}
