pub mod commands;
pub mod completion;
pub mod history;
pub mod keys;
pub mod maps;
pub mod source;

pub use source::KeySource;

use std::any::Any;
use std::io;
use std::time::Duration;

use crate::editing::Id;
use crate::{app::jobs::JobError, delegate_keysource};
use delegate::delegate;

use self::maps::prompt::PromptConfig;
use self::maps::{KeyHandler, KeyResult, UserKeyHandler};
use self::source::memory::MemoryKeySource;

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

    pub fn to_digit(&self) -> Option<u32> {
        if let KeyCode::Char(ch) = self.code {
            ch.to_digit(10)
        } else {
            None
        }
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
    Job(JobError),
    NotPermitted(String),
    ReadOnlyBuffer,
    Interrupted,
    InvalidInput(String),
    NoSuchCommand(String),
    PatternNotFound(String),
}

impl From<KeyError> for io::Error {
    fn from(error: KeyError) -> Self {
        io::Error::new(io::ErrorKind::Other, format!("{:?}", error))
    }
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

#[derive(PartialEq, Clone, Copy)]
pub struct KeymapConfig {
    pub allow_remap: bool,
}

impl Default for KeymapConfig {
    fn default() -> Self {
        Self { allow_remap: true }
    }
}

pub trait KeymapContext: KeySource {
    fn config(&self) -> KeymapConfig {
        KeymapConfig::default()
    }
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
            fn poll_key_with_map(&mut self, timeout: Duration, keymap: Option<Box<&mut dyn BoxableKeymap>>) -> Result<bool, KeyError>;
            fn next_key_with_map(&mut self, keymap: Option<Box<&mut dyn BoxableKeymap>>) -> Result<Option<Key>, KeyError>;
        }
    }
}

pub struct KeymapContextWithKeys<'a, K: KeySource> {
    base: Box<&'a mut dyn KeymapContext>,
    pub config: KeymapConfig,
    keys: K,
}

impl<'a, K: KeySource> KeymapContextWithKeys<'a, K> {
    pub fn new(base: Box<&'a mut dyn KeymapContext>, keys: K, config: KeymapConfig) -> Self {
        Self { base, keys, config }
    }
}

impl<'a, K: KeySource> KeymapContext for KeymapContextWithKeys<'a, K> {
    delegate! {
        to self.base {
            fn state(&self) -> &crate::app::State;
            fn state_mut(&mut self) -> &mut crate::app::State;
        }
    }
    fn config(&self) -> KeymapConfig {
        self.config
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

pub fn remap_keys_to_fn<K: Keymap + BoxableKeymap, R: Remappable<K>>(
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

pub trait Remappable<T: Keymap + BoxableKeymap>: BoxableKeymap {
    fn remap_keys_fn(&mut self, mode: RemapMode, keys: Vec<Key>, handler: Box<KeyHandler<T>>);
    fn buf_remap_keys_fn(
        &mut self,
        buf_id: Id,
        mode: RemapMode,
        keys: Vec<Key>,
        handler: Box<KeyHandler<T>>,
    );
}

pub trait BoxableKeymap {
    fn remap_keys(&mut self, mode: RemapMode, from: Vec<Key>, to: Vec<Key>);
    fn remap_keys_user_fn(&mut self, mode: RemapMode, keys: Vec<Key>, handler: Box<UserKeyHandler>);
    fn buf_remap_keys_user_fn(
        &mut self,
        buf_id: Id,
        mode: RemapMode,
        keys: Vec<Key>,
        handler: Box<UserKeyHandler>,
    );
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn process_keys(&mut self, context: &mut KeymapContextWithKeys<MemoryKeySource>) -> KeyResult;

    fn prompt(&mut self, config: PromptConfig);
}

impl BoxableKeymap for Box<&mut dyn BoxableKeymap> {
    delegate! {
        to (**self) {
            fn remap_keys(&mut self, mode: RemapMode, from: Vec<Key>, to: Vec<Key>);
            fn remap_keys_user_fn(&mut self, mode: RemapMode, keys: Vec<Key>, handler: Box<UserKeyHandler>);
            fn buf_remap_keys_user_fn(&mut self, buf_id: Id, mode: RemapMode, keys: Vec<Key>, handler: Box<UserKeyHandler>);
            fn as_any_mut(&mut self) -> &mut dyn Any;
            fn process_keys(&mut self, context: &mut KeymapContextWithKeys<MemoryKeySource>) -> KeyResult;
            fn prompt(&mut self, config: PromptConfig);
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
    fn process<K: KeymapContext>(&mut self, context: &mut K) -> KeyResult;
}
