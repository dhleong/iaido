use async_trait::async_trait;

use crossterm::event::{EventStream, Event};
use futures::{StreamExt, FutureExt};

use crate::{ui::{UiEvent, UiEvents}, input::{Key, KeySource}};

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
    async fn next_event(&mut self) -> Option<UiEvent> {
        loop {
            let event = self.events.next().fuse().await;
            match event {
                Some(Ok(Event::Resize(_, _))) => return Some(UiEvent::Redraw),
                Some(Ok(Event::Key(key))) => return Some(UiEvent::Key(key)),
                Some(Err(e)) => {
                    // TODO log error to disk, maybe?
                    println!("ERROR reading key: {}", e);
                },
                _ => {}
            }
        }
    }
}

#[async_trait]
impl KeySource for TuiEvents {
    async fn next_key(&mut self) -> Option<Key> {
        loop {
            match self.next_event().await {
                Some(UiEvent::Key(key)) => return Some(key),
                _ => {}
            }
        }
    }
}
