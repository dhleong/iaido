use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::game::engine::GameEngine;

use super::{transport::Transport, ReadValue};

#[derive(Clone)]
pub struct GameConnection {
    conn: Arc<Mutex<Box<dyn Transport + Send>>>,
    pub game: Arc<Mutex<GameEngine>>,
}

impl GameConnection {
    pub fn with_engine(conn: Box<dyn Transport + Send>, game: GameEngine) -> Self {
        Self {
            conn: Arc::new(Mutex::new(conn)),
            game: Arc::new(Mutex::new(game)),
        }
    }
}

impl Transport for GameConnection {
    fn read_timeout(&mut self, duration: Duration) -> io::Result<Option<ReadValue>> {
        if let Some(value) = self.conn.lock().unwrap().read_timeout(duration)? {
            let mut game = self.game.lock().unwrap();
            Ok(game.process_received(value))
        } else {
            Ok(None)
        }
    }

    fn send(&mut self, text: &str) -> io::Result<()> {
        if let Some(processed) = self
            .game
            .lock()
            .unwrap()
            .process_to_send(text.to_string())?
        {
            self.conn.lock().unwrap().send(&processed)
        } else {
            Ok(())
        }
    }
}
