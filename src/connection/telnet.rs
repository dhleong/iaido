use std::io;

use telnet::Telnet;
use url::Url;

use crate::editing::Id;

use super::{ansi::AnsiPipeline, Connection, ConnectionFactory, ReadValue};

const BUFFER_SIZE: usize = 2048;

pub struct TelnetConnection {
    id: Id,
    telnet: Telnet,
    pipeline: AnsiPipeline,
}

impl Connection for TelnetConnection {
    fn id(&self) -> Id {
        self.id
    }
    fn read(&mut self) -> std::io::Result<Option<ReadValue>> {
        match self.telnet.read_nonblocking()? {
            telnet::TelnetEvent::Data(data) => {
                self.pipeline.feed(&data, data.len());
            }
            telnet::TelnetEvent::UnknownIAC(_) => {}
            telnet::TelnetEvent::Negotiation(_, _) => {}
            telnet::TelnetEvent::Subnegotiation(_, _) => {}
            telnet::TelnetEvent::TimedOut => {}
            telnet::TelnetEvent::NoData => {}
            telnet::TelnetEvent::Error(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
        // always attempt to pull ReadValues out of the pipeline
        return Ok(self.pipeline.next());
    }
}

pub struct TelnetConnectionFactory;
impl ConnectionFactory for TelnetConnectionFactory {
    fn create(&self, id: Id, uri: &Url) -> Option<std::io::Result<Box<dyn Connection>>> {
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
                return match Telnet::connect((host, port), BUFFER_SIZE) {
                    Ok(conn) => Some(Ok(Box::new(TelnetConnection {
                        id,
                        telnet: conn,
                        pipeline: AnsiPipeline::new(),
                    }))),
                    Err(e) => Some(Err(e)),
                };
            }
        }

        Some(Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{}: invalid telnet uri", uri),
        )))
    }
}
