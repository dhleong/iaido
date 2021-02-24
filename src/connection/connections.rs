use std::{collections::HashMap, io, sync::Mutex};

use url::Url;

use crate::{
    app::{
        self,
        jobs::{JobRecord, Jobs},
    },
    editing::{ids::Ids, text::TextLines, Id},
};

use super::{Connection, ConnectionFactories};

const DEFAULT_LINES_PER_REDRAW: u16 = 10;

pub struct Connections {
    ids: Ids,
    all: Vec<Box<dyn Connection>>,
    connection_to_buffer: HashMap<Id, Id>,

    // NOTE: More than one Buffer may be associated with a Connection
    // here (for example, the input buffer) but a Connection will
    // be associated with ONLY ONE Buffer above (IE: the buffer it
    // writes to)
    buffer_to_connection: HashMap<Id, Id>,
    factories: ConnectionFactories,
}

impl Default for Connections {
    fn default() -> Self {
        Self {
            ids: Ids::new(),
            all: Vec::new(),
            connection_to_buffer: HashMap::default(),
            buffer_to_connection: HashMap::default(),
            factories: ConnectionFactories::default(),
        }
    }
}

impl Connections {
    pub fn by_id(&self, id: Id) -> Option<&Box<dyn Connection>> {
        self.all.iter().find(|conn| conn.id() == id)
    }

    pub fn by_id_mut(&mut self, id: Id) -> Option<&mut Box<dyn Connection>> {
        self.all.iter_mut().find(|conn| conn.id() == id)
    }

    /// Asynchronously create a new connection attached to the given
    /// buffer_id. Returns a JobRecord for joining on the request
    pub fn create_async(&mut self, jobs: &mut Jobs, buffer_id: Id, uri: Url) -> JobRecord {
        let id = self.ids.next();
        let factory = self.factories.clone();
        jobs.start(move |ctx| async move {
            let connection = Mutex::new(factory.create(id, uri)?);

            ctx.run(move |state| {
                state
                    .connections
                    .as_mut()
                    .unwrap()
                    .add(buffer_id, connection.into_inner().unwrap());
                Ok(())
            })
        })
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
                        winsbuf.append_value(value);
                    }
                    Err(e) => {
                        any_updated = true;
                        winsbuf.append(TextLines::from(e.to_string()));
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

    fn add(&mut self, buffer_id: Id, connection: Box<dyn Connection>) {
        self.connection_to_buffer.insert(connection.id(), buffer_id);
        self.buffer_to_connection.insert(buffer_id, connection.id());
        self.all.push(connection);
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
