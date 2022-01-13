use std::{io, time::Duration};

use super::ReadValue;

pub trait Transport {
    fn read_timeout(&mut self, duration: Duration) -> io::Result<Option<ReadValue>>;
    fn send(&mut self, text: &str) -> io::Result<()>;
}
