pub mod maps;

use std::io;

use async_trait::async_trait;

pub type Key = crossterm::event::KeyEvent;
pub type KeyCode = crossterm::event::KeyCode;

#[derive(Debug)]
pub enum KeyError {
    IO(io::Error),
}

#[async_trait]
pub trait KeySource {
    async fn next_key(&mut self) -> Result<Option<Key>, KeyError>;
}

pub trait KeymapContext : KeySource {
    fn state_mut(&mut self) -> &mut crate::app::State;
}

#[async_trait]
pub trait Keymap {
    /// A Keymap processes some number of keys and updates app state in response to user input.
    /// When the Keymap feels it has finished a unit of work (for example, it has submitted a
    /// command or performed a mapping, etc.) it should relinquish control back to the main loop
    /// (or to a parent Keymap) by returning `Ok(())`
    /// Errors received by context.next_key() may simply be propagated upward, where they will be
    /// printed into the active buffer by the main loop
    async fn process<K: KeymapContext + Send + Sync>(&self, context: &mut K) -> Result<(), KeyError>;
}
