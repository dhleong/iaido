use delegate::delegate;
use std::io;

use crate::editing::Id;
use crate::game::engine::GameEngine;

use super::{Connection, ReadValue};

pub struct GameConnection {
    conn: Box<dyn Connection>,
    pub game: GameEngine,
}

impl GameConnection {
    pub fn with_engine(conn: Box<dyn Connection>, game: GameEngine) -> Self {
        Self { conn, game }
    }
}

impl From<Box<dyn Connection>> for GameConnection {
    fn from(conn: Box<dyn Connection>) -> Self {
        Self {
            conn,
            game: GameEngine::default(),
        }
    }
}

impl Connection for GameConnection {
    delegate! {
        to (self.conn) {
            fn id(&self) -> Id;
            fn read(&mut self) -> io::Result<Option<ReadValue>>;
            fn write(&mut self, bytes: &[u8]) -> io::Result<()>;
        }
    }

    fn send(&mut self, text: String) -> io::Result<()> {
        if let Some(processed) = self.game.process_to_send(text)? {
            self.write(processed.as_bytes())?;
            self.write(&vec!['\n' as u8])
        } else {
            Ok(())
        }
    }
}
