use delegate::delegate;
use std::{io, time::Duration};

use crate::game::engine::GameEngine;

use super::{transport::Transport, ReadValue};

pub struct GameConnection {
    conn: Box<dyn Transport + Send>,
    pub game: GameEngine,
}

impl GameConnection {
    pub fn with_engine(conn: Box<dyn Transport + Send>, game: GameEngine) -> Self {
        Self { conn, game }
    }
}

impl From<Box<dyn Transport + Send>> for GameConnection {
    fn from(conn: Box<dyn Transport + Send>) -> Self {
        Self {
            conn,
            game: GameEngine::default(),
        }
    }
}

impl Transport for GameConnection {
    delegate! {
        to (self.conn) {
            fn read_timeout(&mut self, duration: Duration) -> io::Result<Option<ReadValue>>;
        }
    }

    fn send(&mut self, text: &str) -> io::Result<()> {
        if let Some(processed) = self.game.process_to_send(text.to_string())? {
            self.conn.send(&processed)
        } else {
            Ok(())
        }
    }
}
