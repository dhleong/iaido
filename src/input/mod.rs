use async_trait::async_trait;

pub type Key = crossterm::event::KeyEvent;

#[async_trait]
pub trait KeySource {
    async fn key(&self) -> Key;
}
