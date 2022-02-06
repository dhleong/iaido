use std::{io, time::Duration};

use crate::editing::Size;

use super::ReadValue;

pub trait Transport {
    fn read_timeout(&mut self, duration: Duration) -> io::Result<Option<ReadValue>>;
    fn send(&mut self, text: &str) -> io::Result<()>;

    // Optional:

    fn resize(&mut self, _new_size: Size) -> io::Result<()> {
        Ok(())
    }
}
