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

pub trait Connection: Send {
    fn id(&self) -> Id;
    fn read(&mut self) -> io::Result<Option<ReadValue>>;
    fn write(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn send(&mut self, text: String) -> io::Result<()> {
        self.write(text.as_bytes())?;
        self.write(&vec!['\n' as u8])
    }
}

pub trait ConnectionFactory: Send + Sync {
    fn clone_boxed(&self) -> Box<dyn ConnectionFactory>;
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
    pub fn clone(&self) -> Self {
        Self {
            factories: self.factories.iter().map(|f| f.clone_boxed()).collect(),
        }
    }

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
