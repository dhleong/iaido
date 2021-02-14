use std::io;

use telnet::Telnet;
use url::Url;

use super::{ansi::AnsiPipeline, Connection, ConnectionFactory, ReadValue};

const BUFFER_SIZE: usize = 2048;

pub struct TelnetConnection {
    telnet: Telnet,
    pipeline: AnsiPipeline,
}

impl Connection for TelnetConnection {
    fn read(&mut self) -> std::io::Result<super::ReadValue> {
        match self.telnet.read_nonblocking()? {
            telnet::TelnetEvent::Data(data) => {
                self.pipeline.feed(&data, data.len());
                // TODO pull ReadValues out of the pipeline
            }
            telnet::TelnetEvent::UnknownIAC(_) => {}
            telnet::TelnetEvent::Negotiation(_, _) => {}
            telnet::TelnetEvent::Subnegotiation(_, _) => {}
            telnet::TelnetEvent::TimedOut => {
                return Ok(ReadValue::None);
            }
            telnet::TelnetEvent::NoData => {
                return Ok(ReadValue::None);
            }
            telnet::TelnetEvent::Error(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
        return Ok(ReadValue::None);
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
                return match Telnet::connect((host, port), BUFFER_SIZE) {
                    Ok(conn) => Some(Ok(TelnetConnection {
                        telnet: conn,
                        pipeline: AnsiPipeline::default(),
                    })),
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
