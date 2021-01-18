use crossterm::event::{EventStream, Event};
use futures::{StreamExt, FutureExt};

#[derive(Debug, Clone, Copy)]
pub enum TuiEvent {
    Resize,
}

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

impl TuiEvents {
    pub async fn next(&mut self) -> Option<TuiEvent> {
        loop {
            let event = self.events.next().fuse().await;
            match event {
                Some(Ok(Event::Resize(_, _))) => return Some(TuiEvent::Resize),
                _ => {}
            }
        }
    }
}
