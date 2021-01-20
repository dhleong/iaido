use std::io;

use async_trait::async_trait;

use crossterm::{event::{EventStream, Event}, ErrorKind};
use futures::{StreamExt, FutureExt};

use crate::{ui::{UiEvent, UiEvents}, input::{Key, KeyError, KeySource}};

pub struct TuiEvents {
    events: EventStream,
}

impl Default for TuiEvents {
    fn default() -> Self {
        Self {
            events: EventStream::new(),
        }
    }
}

#[async_trait]
impl UiEvents for TuiEvents {
    async fn next_event(&mut self) -> Result<UiEvent, io::Error> {
        loop {
            let event = self.events.next().fuse().await;
            match event {
                Some(Ok(Event::Resize(_, _))) => return Ok(UiEvent::Redraw),
                Some(Ok(Event::Key(key))) => return Ok(UiEvent::Key(key)),
                Some(Err(e)) => match e {
                    ErrorKind::IoError(source) => return Err(source),
                    _ => return Err(io::Error::new(io::ErrorKind::Other, e)),
                }
                _ => {}
            }
        }
    }
}

#[async_trait]
impl KeySource for TuiEvents {
    async fn next_key(&mut self) -> Result<Option<Key>, KeyError> {
        loop {
            match self.next_event().await {
                Ok(UiEvent::Key(key)) => return Ok(Some(key)),
                Err(e) => return Err(KeyError::IO(e)),
                _ => {}
            }
        }
    }
}
