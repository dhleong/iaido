use std::io;

use url::Url;

use crate::editing::text::TextLine;

use self::{telnet::TelnetConnectionFactory, transport::Transport};

mod ansi;
pub mod connections;
pub mod game;
mod reader;
mod telnet;
mod tls;
pub mod transport;

#[derive(Debug, PartialEq)]
pub enum ReadValue {
    Newline,
    Text(TextLine),
}

pub trait TransportFactory: Send + Sync {
    fn clone_boxed(&self) -> Box<dyn TransportFactory>;
    fn create(&self, uri: &Url) -> Option<io::Result<Box<dyn Transport + Send>>>;
}

pub struct TransportFactories {
    factories: Vec<Box<dyn TransportFactory>>,
}

impl Default for TransportFactories {
    fn default() -> Self {
        TransportFactories {
            factories: vec![Box::new(TelnetConnectionFactory)],
        }
    }
}

impl TransportFactories {
    pub fn clone(&self) -> Self {
        Self {
            factories: self.factories.iter().map(|f| f.clone_boxed()).collect(),
        }
    }

    pub fn create(&self, uri: Url) -> io::Result<Box<dyn Transport + Send>> {
        for f in &self.factories {
            match f.create(&uri) {
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
