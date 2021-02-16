use std::io;

use url::Url;

use crate::editing::text::TextLine;

use self::telnet::TelnetConnectionFactory;

mod ansi;
mod telnet;

#[derive(Debug, PartialEq)]
pub enum ReadValue {
    Newline,
    Text(TextLine),
}

pub trait Connection {
    fn read(&mut self) -> io::Result<Option<ReadValue>>;
}

pub trait ConnectionFactory<T: Connection> {
    fn create(&self, uri: &Url) -> Option<io::Result<T>>;
}

pub struct ConnectionFactories;

impl ConnectionFactories {
    pub fn create(&self, uri: Url) -> io::Result<impl Connection> {
        for f in vec![TelnetConnectionFactory] {
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
