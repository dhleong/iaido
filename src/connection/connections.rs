use std::{collections::HashMap, io};

use url::Url;

use crate::{
    app,
    editing::{ids::Ids, text::TextLines, Id},
};

use super::{Connection, ConnectionFactories};

pub struct Connections {
    ids: Ids,
    all: Vec<Box<dyn Connection>>,
    connection_to_buffer: HashMap<Id, Id>,
    factories: ConnectionFactories,
}

impl Default for Connections {
    fn default() -> Self {
        Self {
            ids: Ids::new(),
            all: Vec::new(),
            connection_to_buffer: HashMap::default(),
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

    /// Create a new connection attached to the given buffer_id
    pub fn create(&mut self, buffer_id: Id, uri: Url) -> io::Result<Id> {
        let id = self.ids.next();
        let result = self.factories.create(id, uri)?;
        self.connection_to_buffer.insert(id, buffer_id);
        self.all.push(result);
        Ok(id)
    }

    pub fn process(&mut self, app: &mut app::State) -> bool {
        let to_buffer = &mut self.connection_to_buffer;
        let mut any_updated = false;
        retain(&mut self.all, |conn| {
            let buffer_id = to_buffer[&conn.id()];
            let mut winsbuf = app
                .winsbuf_by_id(buffer_id)
                .expect("Could not find buffer for connection");

            match conn.read() {
                Ok(None) => {} // nop
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
            };

            // keep the conn, by default
            return RetainAction::Keep;
        });
        any_updated
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
