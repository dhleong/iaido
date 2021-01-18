use async_trait::async_trait;
use async_std::task;
use crossterm::event::Event;

use crate::input::KeySource;

pub struct TuiKeySource {}

impl Default for TuiKeySource {
    fn default() -> Self {
        Self {}
    }
}

#[async_trait]
impl KeySource for TuiKeySource {
    async fn key(&self) -> crate::input::Key {
        let task = task::spawn(async {
            loop {
                match crossterm::event::read() {
                    Ok(Event::Key(key)) => return key,
                    _ => {
                        // TODO ?
                    }
                }
            }
        });
        task.await
    }
}
