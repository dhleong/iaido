use std::{
    collections::HashMap,
    io,
    sync::{Arc, Mutex},
};

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
    Connection, ConnectionFactories,
};

pub struct ConnectionRecord {
    pub id: Id,
    #[allow(unused)]
    stop_read_signal: StopSignal,
    outgoing: mpsc::UnboundedSender<String>,
    connection: Arc<Mutex<GameConnection>>,
}

impl ConnectionRecord {
    pub fn send(&mut self, message: String) -> io::Result<()> {
        match self.outgoing.send(message) {
            Ok(_) => Ok(()),
            Err(_) => Err(io::ErrorKind::NotConnected.into()),
        }
    }

    pub fn with_engine<R, F: FnOnce(&GameEngine) -> R>(&self, f: F) -> R {
        let conn = self.connection.lock().unwrap();
        f(&conn.game)
    }

    pub fn with_engine_mut<R, F: FnOnce(&mut GameEngine) -> R>(&mut self, f: F) -> R {
        let mut conn = self.connection.lock().unwrap();
        f(&mut conn.game)
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
    factories: ConnectionFactories,

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
            let connection = Mutex::new(factory.create(id, uri)?);
            let transport_context = Mutex::new(ctx.clone());

            ctx.run(move |state| {
                state.connections.as_mut().unwrap().add_transport(
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
        connection: Arc<Mutex<GameConnection>>,
    ) -> ConnectionRecord {
        let stop_read_signal = TransportReader::spawn(ctx, buffer_id, connection.clone());

        let (tx, mut rx) = mpsc::unbounded_channel::<String>();
        let writable = connection.clone();
        tokio::spawn(async move {
            while let Some(to_send) = rx.recv().await {
                let mut conn = writable.lock().unwrap();
                conn.send(&to_send); // TODO send result... somewhere
            }
        });

        ConnectionRecord {
            id,
            stop_read_signal,
            outgoing: tx,
            connection,
        }
    }

    /// Returns the associated buffer ID
    pub fn disconnect(&mut self, connection_id: Id) -> io::Result<Id> {
        if self.by_id.remove(&connection_id).is_some() {
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

        let record =
            Connections::launch(id, ctx.clone(), buffer_id, Arc::new(Mutex::new(transport)));

        self.add_record(id, buffer_id, input_buffer_id, record);
    }

    fn add_record(&mut self, id: Id, buffer_id: Id, input_buffer_id: Id, record: ConnectionRecord) {
        self.connection_to_buffer.insert(id, buffer_id);
        self.buffer_to_connection.insert(buffer_id, id);
        self.buffer_to_connection.insert(input_buffer_id, id);
        self.by_id.insert(id, record);
    }

    #[cfg(test)]
    pub fn add_for_test(&mut self, _buffer_id: Id, _connection: Box<dyn Connection>) {
        // TODO
    }
}
