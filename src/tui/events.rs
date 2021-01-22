use std::{io, time::Duration};

use crossterm::{event::Event, ErrorKind};

use crate::{
    input::{Key, KeyError, KeySource},
    ui::{UiEvent, UiEvents},
};

pub struct TuiEvents {
    pending_event: Option<UiEvent>,
}

impl Default for TuiEvents {
    fn default() -> Self {
        Self {
            pending_event: None,
        }
    }
}

fn wrap_as_io(e: ErrorKind) -> io::Error {
    match e {
        ErrorKind::IoError(source) => source,
        _ => io::Error::new(io::ErrorKind::Other, e),
    }
}

impl UiEvents for TuiEvents {
    fn poll_event(&mut self, timeout: Duration) -> io::Result<Option<UiEvent>> {
        match crossterm::event::poll(timeout) {
            Ok(found) if found => {
                if let Some(pending) = self.pending_event {
                    // unconsumed pending event; return unchanged:
                    Ok(Some(pending))
                } else {
                    let next = self.next_event()?;
                    self.pending_event = Some(next);
                    Ok(Some(next))
                }
            }
            Ok(_) => Ok(None),
            Err(e) => Err(wrap_as_io(e)),
        }
    }

    fn next_event(&mut self) -> Result<UiEvent, io::Error> {
        if let Some(pending) = self.pending_event {
            self.pending_event = None;
            return Ok(pending);
        }

        loop {
            match crossterm::event::read() {
                Ok(Event::Resize(_, _)) => return Ok(UiEvent::Redraw),
                Ok(Event::Key(key)) => return Ok(UiEvent::Key(key)),
                Err(e) => return Err(wrap_as_io(e)),
                _ => {}
            }
        }
    }
}

impl KeySource for TuiEvents {
    fn poll_key(&mut self, duration: Duration) -> Result<bool, KeyError> {
        match self.poll_event(duration) {
            Ok(Some(UiEvent::Key(_))) => Ok(true),
            Ok(_) => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        loop {
            match self.next_event() {
                Ok(UiEvent::Key(key)) => return Ok(Some(key)),
                Err(e) => return Err(e.into()),
                _ => {}
            }
        }
    }
}
