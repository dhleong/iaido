use native_tls::TlsConnector;
use std::io;
use std::net::TcpStream;

use telnet::Telnet;
use url::Url;

use crate::editing::Id;

use super::{ansi::AnsiPipeline, tls, Connection, ConnectionFactory, ReadValue};

const BUFFER_SIZE: usize = 2048;

pub struct TelnetConnection {
    id: Id,
    telnet: Telnet,
    pipeline: AnsiPipeline,
}

/// NOTE: this `unsafe` is probably a terrible idea, but *should* be
/// fine. We use this *only once* to move the Connection's thread after a
/// successful connection and before any reads or writes. The Telnet
/// lib does not appear to use any thread local state, and TcpStream
/// has try_clone so... *should be* fine.
unsafe impl Send for TelnetConnection {}

impl Connection for TelnetConnection {
    fn id(&self) -> Id {
        self.id
    }

    fn read(&mut self) -> io::Result<Option<ReadValue>> {
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

    fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.telnet.write(bytes)?;
        Ok(())
    }
}

fn connect(host: &str, port: u16, secure: bool, buffer_size: usize) -> io::Result<Telnet> {
    let tcp = TcpStream::connect((host, port))?;
    if !secure {
        Ok(Telnet::from_stream(Box::new(tcp), buffer_size))
    } else {
        let connector = TlsConnector::new().expect("Failed to initialize TLS");
        match connector.connect(host, tcp) {
            Ok(raw_tls) => Ok(tls::create_telnet(raw_tls, buffer_size)),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}

pub struct TelnetConnectionFactory;
impl ConnectionFactory for TelnetConnectionFactory {
    fn clone_boxed(&self) -> Box<dyn ConnectionFactory> {
        Box::new(TelnetConnectionFactory)
    }

    fn create(&self, id: Id, uri: &Url) -> Option<std::io::Result<Box<dyn Connection>>> {
        let secure = match uri.scheme() {
            "telnet" => false,
            "ssl" => true,
            _ => return None,
        };

        match (uri.host_str(), uri.port()) {
            (Some(host), Some(port)) => match connect(host, port, secure, BUFFER_SIZE) {
                Ok(conn) => Some(Ok(Box::new(TelnetConnection {
                    id,
                    telnet: conn,
                    pipeline: AnsiPipeline::new(),
                }))),
                Err(e) => Some(Err(e)),
            },

            _ => Some(Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{}: invalid telnet uri", uri),
            ))),
        }
    }
}
