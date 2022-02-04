use std::io;

use url::Url;

use crate::editing::{text::TextLine, Size};

use self::{telnet::TelnetConnectionFactory, transport::Transport};

mod ansi;
pub mod connections;
mod flags;
pub mod game;
mod reader;
mod telnet;
mod tls;
pub mod transport;

pub use flags::Flags;

#[derive(Debug, PartialEq)]
pub enum ReadValue {
    Newline,
    Text(TextLine),
}

pub struct ConnectParams {
    uri: Url,
    size: Size,
    flags: Flags,
}

impl ConnectParams {
    pub fn with_uri_and_size(uri: Url, size: Size) -> Self {
        Self {
            uri,
            size,
            flags: Flags::default(),
        }
    }
}

pub trait TransportFactory: Send + Sync {
    fn clone_boxed(&self) -> Box<dyn TransportFactory>;
    fn create(&self, params: &ConnectParams) -> Option<io::Result<Box<dyn Transport + Send>>>;
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

    pub fn create(&self, params: ConnectParams) -> io::Result<Box<dyn Transport + Send>> {
        for f in &self.factories {
            match f.create(&params) {
                None => {} // unsupported
                Some(Ok(conn)) => return Ok(conn),
                Some(Err(e)) => return Err(e),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{}: Unsupported uri", params.uri),
        ))
    }
}
