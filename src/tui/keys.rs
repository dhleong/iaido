use async_trait::async_trait;
use crossterm::event::{Event, EventStream};
use futures::{StreamExt, FutureExt};

use crate::input::KeySource;

pub struct TuiKeySource {
    events: EventStream,
}

impl Default for TuiKeySource {
    fn default() -> Self {
        Self {
            events: EventStream::new(),
        }
    }
}

#[async_trait]
impl KeySource for TuiKeySource {
    async fn key(&mut self) -> Option<crate::input::Key> {
        loop {
            let event = self.events.next().fuse().await;
            match event {
                Some(Ok(ev)) => match ev {
                    Event::Key(key) => return Some(key),
                    Event::Mouse(_) => {}
                    Event::Resize(_, _) => {}
                },
                Some(Err(e)) => {
                    // TODO log error to disk, maybe?
                    println!("ERROR reading key: {}", e);
                },
                None => {}
            }
        }
    }
}
