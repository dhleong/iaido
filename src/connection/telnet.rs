use native_tls::TlsConnector;
use std::io;
use std::net::TcpStream;

mod echo;
mod handler;
mod handlers;
mod naws;
mod ttype;

use telnet::{Telnet, TelnetOption};

use self::handler::{TelnetHandler, TelnetOptionInteractor};
use self::handlers::TelnetHandlers;

use super::ConnectParams;
use super::{ansi::AnsiPipeline, tls, transport::Transport, ReadValue, TransportFactory};

const BUFFER_SIZE: usize = 2048;

struct TelnetWrapper(Telnet);

pub struct TelnetConnection {
    telnet: TelnetWrapper,
    handlers: TelnetHandlers,
    pipeline: AnsiPipeline,
}

/// NOTE: this `unsafe` is probably a terrible idea, but *should* be
/// fine. We use this *only once* to move the Connection's thread after a
/// successful connection and before any reads or writes. The Telnet
/// lib does not appear to use any thread local state, and TcpStream
/// has try_clone so... *should be* fine.
unsafe impl Send for TelnetWrapper {}

impl TelnetConnection {
    fn process_event(&mut self, event: telnet::Event) -> io::Result<Option<ReadValue>> {
        match event {
            telnet::Event::Data(data) => {
                self.pipeline.feed(&data, data.len());
            }
            telnet::Event::UnknownIAC(_) => {}

            telnet::Event::Negotiation(action, option) => {
                crate::info!("## TELNET < {:?} {:?}", &action, &option);
                if let Some(handler) = self.handlers.get_mut(&option) {
                    handler.negotiate(&action, &mut self.telnet.0)
                } else {
                    TelnetOptionInteractor::default(option).negotiate(&action, &mut self.telnet.0)
                }?;
            }

            telnet::Event::Subnegotiation(option, bytes) => {
                crate::info!(
                    "## TELNET < IAC SB {:?} (... {} bytes)",
                    &option,
                    bytes.len()
                );

                if let Some(handler) = self.handlers.get_mut(&option) {
                    let result = handler.on_subnegotiate(&mut self.telnet.0, &bytes);

                    if let Err(e) = result {
                        return Err(io::Error::new(io::ErrorKind::Other, e));
                    }
                }
            }

            telnet::Event::TimedOut => {}
            telnet::Event::NoData => {}
            telnet::Event::Error(e) => {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
        // always attempt to pull ReadValues out of the pipeline
        return Ok(self.pipeline.next());
    }
}

impl Transport for TelnetConnection {
    fn resize(&mut self, new_size: crate::editing::Size) -> io::Result<()> {
        if let Some(naws) = self.handlers.get_mut(&TelnetOption::NAWS) {
            if let Err(e) = naws.on_resize(&mut self.telnet.0, new_size) {
                return Err(io::Error::new(io::ErrorKind::Other, e));
            }
        }
        Ok(())
    }

    fn read_timeout(&mut self, duration: std::time::Duration) -> io::Result<Option<ReadValue>> {
        if let Some(pending) = self.pipeline.next() {
            return Ok(Some(pending));
        }

        let event = self.telnet.0.read_timeout(duration)?;
        self.process_event(event)
    }

    fn send(&mut self, text: &str) -> io::Result<()> {
        self.telnet.0.write(text.as_bytes())?;
        self.telnet.0.write(b"\r\n")?;
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
impl TransportFactory for TelnetConnectionFactory {
    fn clone_boxed(&self) -> Box<dyn TransportFactory> {
        Box::new(TelnetConnectionFactory)
    }

    fn create(&self, params: &ConnectParams) -> Option<std::io::Result<Box<dyn Transport + Send>>> {
        let secure = match params.uri.scheme() {
            "telnet" => false,
            "ssl" => true,
            _ => return None,
        };

        match (params.uri.host_str(), params.uri.port()) {
            (Some(host), Some(port)) => match connect(host, port, secure, BUFFER_SIZE) {
                Ok(conn) => Some(Ok(Box::new(TelnetConnection {
                    telnet: TelnetWrapper(conn),
                    handlers: TelnetHandlers::with_params(params),
                    pipeline: AnsiPipeline::new(),
                }))),
                Err(e) => Some(Err(e)),
            },

            _ => Some(Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{}: invalid telnet uri", params.uri),
            ))),
        }
    }
}
