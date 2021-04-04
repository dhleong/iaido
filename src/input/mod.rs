pub mod commands;
pub mod completion;
pub mod keys;
pub mod maps;
pub mod source;

pub use source::KeySource;

use std::io;
use std::time::Duration;

use crate::delegate_keysource;
use delegate::delegate;

use self::maps::KeyHandler;

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

    pub fn write_str(&self, dest: &mut String) {
        let opened = if self.modifiers != KeyModifiers::NONE {
            dest.push('<');
            true
        } else {
            false
        };

        if self.modifiers.contains(KeyModifiers::CONTROL) {
            dest.push_str("c-");
        }
        if self.modifiers.contains(KeyModifiers::ALT) {
            dest.push_str("m-");
        }

        match self.code {
            KeyCode::Char(ch) => dest.push(ch),
            code => dest.push_str(format!("{:?}", code).as_str()),
        };

        if opened {
            dest.push('>');
        }
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
    NotPermitted(String),
    ReadOnlyBuffer,
    Interrupted,
    InvalidInput(String),
    NoSuchCommand(String),
}

impl From<io::Error> for KeyError {
    fn from(error: io::Error) -> Self {
        KeyError::IO(error)
    }
}

impl From<url::ParseError> for KeyError {
    fn from(error: url::ParseError) -> Self {
        KeyError::InvalidInput(error.to_string())
    }
}

pub trait KeymapContext: KeySource {
    fn state(&self) -> &crate::app::State;
    fn state_mut(&mut self) -> &mut crate::app::State;
}

impl KeymapContext for Box<&mut dyn KeymapContext> {
    delegate! {
        to (**self) {
            fn state(&self) -> &crate::app::State;
            fn state_mut(&mut self) -> &mut crate::app::State;
        }
    }
}

impl KeySource for Box<&mut dyn KeymapContext> {
    delegate! {
        to (**self) {
            fn poll_key(&mut self, timeout: Duration) -> Result<bool, KeyError>;
            fn next_key(&mut self) -> Result<Option<Key>, KeyError>;
        }
    }
}

pub struct KeymapContextWithKeys<'a, K: KeySource> {
    base: Box<&'a mut dyn KeymapContext>,
    keys: K,
}

impl<'a, K: KeySource> KeymapContextWithKeys<'a, K> {
    pub fn new(base: Box<&'a mut dyn KeymapContext>, keys: K) -> Self {
        Self { base, keys }
    }
}

impl<'a, K: KeySource> KeymapContext for KeymapContextWithKeys<'a, K> {
    delegate! {
        to self.base {
            fn state(&self) -> &crate::app::State;
            fn state_mut(&mut self) -> &mut crate::app::State;
        }
    }
}

impl<'a, K: KeySource> KeySource for KeymapContextWithKeys<'a, K> {
    delegate_keysource!(keys);
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum RemapMode {
    VimNormal,
    VimInsert,
    User(String),
}

pub trait BoxableKeymap {
    fn remap_keys(&mut self, mode: RemapMode, from: Vec<Key>, to: Vec<Key>);
}

pub fn remap_keys_to_fn<K: Keymap, R: Remappable<K>>(
    keymap: &mut R,
    mode: RemapMode,
    from: Vec<Key>,
    to: Vec<Key>,
) {
    keymap.remap_keys_fn(
        mode,
        from,
        Box::new(move |ctx| {
            ctx.feed_keys(to.clone())?;
            Ok(())
        }),
    );
}

pub trait Remappable<T: Keymap>: BoxableKeymap {
    fn remap_keys_fn(&mut self, mode: RemapMode, keys: Vec<Key>, handler: Box<KeyHandler<T>>);
}

impl BoxableKeymap for Box<&mut dyn BoxableKeymap> {
    delegate! {
        to (**self) {
            fn remap_keys(&mut self, mode: RemapMode, from: Vec<Key>, to: Vec<Key>);
        }
    }
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
