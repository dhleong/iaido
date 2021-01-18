use async_trait::async_trait;

pub type Key = crossterm::event::KeyEvent;
pub type KeyCode = crossterm::event::KeyCode;

#[async_trait]
pub trait KeySource {
    async fn key(&mut self) -> Option<Key>;
}
