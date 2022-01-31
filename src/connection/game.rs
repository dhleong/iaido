use std::{
    io,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{editing::Size, game::engine::GameEngine};

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
    fn resize(&mut self, new_size: Size) -> io::Result<()> {
        self.conn.lock().unwrap().resize(new_size)
    }

    fn read_timeout(&mut self, duration: Duration) -> io::Result<Option<ReadValue>> {
        // NOTE: The explicit scoping here and in send() are to ensure this
        // lock gets released before we attempt to access the next lock
        let read = { self.conn.lock().unwrap().read_timeout(duration) };
        if let Some(value) = read? {
            Ok(self.game.lock().unwrap().process_received(value))
        } else {
            Ok(None)
        }
    }

    fn send(&mut self, text: &str) -> io::Result<()> {
        let processed = {
            self.game
                .lock()
                .unwrap()
                .process_to_send(text.to_string())?
        };
        if let Some(processed) = processed {
            self.conn.lock().unwrap().send(&processed)
        } else {
            Ok(())
        }
    }
}
