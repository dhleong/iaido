use std::io;

use telnet::Telnet;
use url::Url;

use super::{Connection, ConnectionFactory};

const BUFFER_SIZE: usize = 2048;

pub struct TelnetConnection {
    telnet: Telnet,
}

impl Connection for TelnetConnection {
    fn read(&mut self) -> std::io::Result<super::ReadValue> {
        let event = self.telnet.read_nonblocking()?;
        todo!("handle {:?}", event);
    }
}

pub struct TelnetConnectionFactory;
impl ConnectionFactory<TelnetConnection> for TelnetConnectionFactory {
    fn create(&self, uri: &Url) -> Option<std::io::Result<TelnetConnection>> {
        let secure = match uri.scheme() {
            "telnet" => false,
            "ssl" => true,
            _ => return None,
        };
        if secure {
            todo!("tls");
        }

        if let Some(host) = uri.host_str() {
            if let Some(port) = uri.port() {
                match Telnet::connect((host, port), BUFFER_SIZE) {
                    Ok(conn) => {
                        todo!("connected")
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
        }

        Some(Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{}: invalid telnet uri", uri),
        )))
    }
}
