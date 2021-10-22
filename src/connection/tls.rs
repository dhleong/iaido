use delegate::delegate;
use native_tls::TlsStream;
use std::io::{self, Read, Write};
use std::net::TcpStream;
use std::time::Duration;
use telnet::Telnet;

pub struct TlsTelnetStream(TlsStream<TcpStream>);

impl Read for TlsTelnetStream {
    delegate! {
        to self.0 {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize>;
        }
    }
}
impl Write for TlsTelnetStream {
    delegate! {
        to self.0 {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize>;
            fn flush(&mut self) -> io::Result<()>;
        }
    }
}

impl telnet::Stream for TlsTelnetStream {
    delegate! {
        to self.0.get_ref() {
            fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()>;
            fn set_read_timeout(&self, dur: Option<Duration>) -> io::Result<()>;
        }
    }
}

pub fn create_telnet(raw_tls: TlsStream<TcpStream>, buffer_size: usize) -> Telnet {
    Telnet::from_stream(Box::new(TlsTelnetStream(raw_tls)), buffer_size)
}
