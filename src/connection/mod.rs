use std::io;

use url::Url;

use crate::editing::{text::TextLine, Id};

use self::telnet::TelnetConnectionFactory;

mod ansi;
pub mod connections;
mod telnet;

#[derive(Debug, PartialEq)]
pub enum ReadValue {
    Newline,
    Text(TextLine),
}

pub trait Connection {
    fn id(&self) -> Id;
    fn read(&mut self) -> io::Result<Option<ReadValue>>;
}

pub trait ConnectionFactory {
    fn create(&self, id: Id, uri: &Url) -> Option<io::Result<Box<dyn Connection>>>;
}

pub struct ConnectionFactories {
    factories: Vec<Box<dyn ConnectionFactory>>,
}

impl Default for ConnectionFactories {
    fn default() -> Self {
        ConnectionFactories {
            factories: vec![Box::new(TelnetConnectionFactory)],
        }
    }
}

impl ConnectionFactories {
    pub fn create(&self, id: Id, uri: Url) -> io::Result<Box<dyn Connection>> {
        for f in &self.factories {
            match f.create(id, &uri) {
                None => {} // unsupported
                Some(Ok(conn)) => return Ok(conn),
                Some(Err(e)) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{}: Unsupported uri", uri),
        ))
    }
}
