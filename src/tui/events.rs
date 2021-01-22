use std::{io, time::Duration};

use crossterm::{event::Event, ErrorKind};

use crate::{
    input::{Key, KeyError, KeySource},
    ui::{UiEvent, UiEvents},
};

pub struct TuiEvents {}

impl Default for TuiEvents {
    fn default() -> Self {
        Self {}
    }
}

fn wrap_as_io(e: ErrorKind) -> io::Error {
    match e {
        ErrorKind::IoError(source) => source,
        _ => io::Error::new(io::ErrorKind::Other, e),
    }
}

impl UiEvents for TuiEvents {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<bool> {
        match crossterm::event::poll(timeout) {
            Ok(result) => Ok(result),
            Err(e) => Err(wrap_as_io(e)),
        }
    }

    fn next_event(&mut self) -> Result<UiEvent, io::Error> {
        loop {
            let event = crossterm::event::read();
            match event {
                Ok(Event::Resize(_, _)) => return Ok(UiEvent::Redraw),
                Ok(Event::Key(key)) => return Ok(UiEvent::Key(key)),
                Err(e) => return Err(wrap_as_io(e)),
                _ => {}
            }
        }
    }
}

impl KeySource for TuiEvents {
    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        loop {
            match self.next_event() {
                Ok(UiEvent::Key(key)) => return Ok(Some(key)),
                Err(e) => return Err(KeyError::IO(e)),
                _ => {}
            }
        }
    }
}
