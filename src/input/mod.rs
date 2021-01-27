pub mod keys;
pub mod maps;

use std::{io, time::Duration};

pub type KeyCode = crossterm::event::KeyCode;
pub type KeyModifiers = crossterm::event::KeyModifiers;

#[derive(Debug, PartialOrd, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}
impl Key {
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }
}

impl From<KeyCode> for Key {
    fn from(code: KeyCode) -> Self {
        Key::new(code, KeyModifiers::NONE)
    }
}

#[derive(Debug)]
pub enum KeyError {
    IO(io::Error),
}

impl Into<KeyError> for io::Error {
    fn into(self) -> KeyError {
        KeyError::IO(self)
    }
}

pub trait KeySource {
    fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError>;
    fn next_key(&mut self) -> Result<Option<Key>, KeyError>;
}

pub trait KeymapContext: KeySource {
    fn state(&self) -> &crate::app::State;
    fn state_mut(&mut self) -> &mut crate::app::State;
}

pub trait Keymap {
    /// A Keymap processes some number of keys and updates app state in response to user input.
    /// When the Keymap feels it has finished a unit of work (for example, it has submitted a
    /// command or performed a mapping, etc.) it should relinquish control back to the main loop
    /// (or to a parent Keymap) by returning `Ok(())`
    /// Errors received by context.next_key() may simply be propagated upward, where they will be
    /// printed into the active buffer by the main loop
    fn process<K: KeymapContext>(&mut self, context: &mut K) -> Result<(), KeyError>;
}
