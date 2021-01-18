use async_trait::async_trait;

use crossterm::event::{EventStream, Event};
use futures::{StreamExt, FutureExt};

use crate::ui::{UiEvent, UiEvents};

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
    async fn next(&mut self) -> Option<UiEvent> {
        loop {
            let event = self.events.next().fuse().await;
            match event {
                Some(Ok(Event::Resize(_, _))) => return Some(UiEvent::Redraw),
                _ => {}
            }
        }
    }
}
