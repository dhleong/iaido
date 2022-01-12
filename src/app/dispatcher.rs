use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use crate::app;
use crate::input::{maps::KeyResult, Key, KeyError, KeySource};

pub struct DispatchRecord<R> {
    from_main: Receiver<R>,
}

impl<R> DispatchRecord<R> {
    pub fn join(&self) -> KeyResult<R> {
        Ok(self.from_main.recv().unwrap())
    }

    /// Wait for this Job to finish, returning a KeyResult representing
    /// the result of the Job. This fn acts like it's blocking input,
    /// but still allows the UI to redraw and also accepts <ctrl-c> input
    /// from the user, which triggers a cancellation of this Job, returning
    /// [`KeyError:Interrupted`] to the caller.
    pub fn join_interruptably<K: KeySource>(&self, keys: &mut K) -> KeyResult<R> {
        loop {
            match self.from_main.recv_timeout(Duration::from_millis(50)) {
                Ok(result) => return Ok(result),
                Err(_) => {} // Receive timeout
            }

            if keys.poll_key(Duration::from_millis(50))? {
                match keys.next_key()? {
                    Some(key) if key == Key::from("<c-c>") => {
                        return Err(KeyError::Interrupted);
                    }
                    _ => {}
                }
            }
        }
    }
}

struct PendingDispatch<R: Send, F: FnOnce(&mut app::State) -> R + Send> {
    f: Option<F>,
    to_caller: Sender<R>,
}

impl<R: Send, F: FnOnce(&mut app::State) -> R + Send> PendingDispatch<R, F> {
    #[allow(unused)]
    pub fn execute(&mut self, state: &mut app::State) {
        if let Some(f) = self.f.take() {
            self.to_caller.send(f(state)).unwrap();
        }
    }
}

type BoxedPendingDispatch = Box<dyn FnMut(&mut app::State)>;

/// Provides access to the main thread
pub struct Dispatcher {
    to_main: Sender<BoxedPendingDispatch>,
    rx: Receiver<BoxedPendingDispatch>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Dispatcher { to_main: tx, rx }
    }
}

impl Dispatcher {
    pub fn process(state: &mut app::State) -> io::Result<bool> {
        if let Some(mut action) = state.dispatcher.next_action()? {
            action(state);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn next_action(&mut self) -> io::Result<Option<BoxedPendingDispatch>> {
        // TODO Probably, do a hard recv or recv with a long timeout
        match self.rx.try_recv() {
            Ok(action) => Ok(Some(action)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    pub fn spawn<R, F>(&mut self, f: F) -> DispatchRecord<R>
    where
        R: Send + 'static,
        F: FnOnce(&mut app::State) -> R + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let mut pending = PendingDispatch {
            f: Some(f),
            to_caller: tx,
        };

        let b: BoxedPendingDispatch = Box::new(move |state| pending.execute(state));

        self.to_main.send(b).unwrap();

        DispatchRecord { from_main: rx }
    }
}
