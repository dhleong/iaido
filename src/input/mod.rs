pub mod maps;

use async_trait::async_trait;

pub type Key = crossterm::event::KeyEvent;
pub type KeyCode = crossterm::event::KeyCode;

#[async_trait]
pub trait KeySource {
    async fn next_key(&mut self) -> Option<Key>;
}

pub trait KeymapContext : KeySource {
    fn state_mut(&mut self) -> &mut crate::app::State;
}

#[async_trait]
pub trait Keymap {
    async fn process<K: KeymapContext + Send + Sync>(&self, context: &mut K) -> Option<()>;
}
