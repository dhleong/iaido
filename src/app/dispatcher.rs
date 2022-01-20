use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

use crate::input::{maps::KeyResult, Key, KeyError, KeySource, KeymapContext};
use crate::{app, input::commands::CommandHandlerContext};

const MAX_PER_FRAME_DURATION: Duration = Duration::from_millis(50);

#[must_use = "Use background to ignore the launch and any result"]
pub struct DispatchRecord<R> {
    from_main: Receiver<R>,
}

impl<R> DispatchRecord<R> {
    pub fn background(&self) {
        // nop
    }

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

struct PendingDispatch<R: Send, F: FnOnce(&mut CommandHandlerContext) -> R + Send> {
    f: Option<F>,
    to_caller: Sender<R>,
}

impl<R: Send, F: FnOnce(&mut CommandHandlerContext) -> R + Send> PendingDispatch<R, F> {
    #[allow(unused)]
    pub fn execute(&mut self, ctx: &mut CommandHandlerContext) {
        if let Some(f) = self.f.take() {
            let result = f(ctx);

            // NOTE: A SendError means that the receiver has been deallocated,
            // so we can just ignore it:
            self.to_caller.send(result).ok();
        }
    }
}

type BoxedPendingDispatch = Box<dyn FnMut(&mut CommandHandlerContext) + Send>;

/// Provides access to the main thread. The sender side may be trivially
/// cloned
pub struct Dispatcher {
    pub sender: DispatchSender,
    rx: Receiver<BoxedPendingDispatch>,
}

#[derive(Clone)]
pub struct DispatchSender {
    to_main: Sender<BoxedPendingDispatch>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let sender = DispatchSender { to_main: tx };
        Dispatcher { sender, rx }
    }
}

impl Dispatcher {
    pub fn process(ctx: &mut CommandHandlerContext) -> io::Result<bool> {
        let mut any_tasks = false;
        let start = Instant::now();
        loop {
            if let Some(mut action) = ctx.state_mut().dispatcher.next_action()? {
                action(ctx);
                any_tasks = true;
            } else {
                break;
            }

            if start.elapsed() >= MAX_PER_FRAME_DURATION {
                break;
            }
        }
        Ok(any_tasks)
    }

    fn next_action(&mut self) -> io::Result<Option<BoxedPendingDispatch>> {
        // TODO Probably, do a hard recv or recv with a long timeout
        match self.rx.try_recv() {
            Ok(action) => Ok(Some(action)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}

impl DispatchSender {
    pub fn spawn<R, F>(&self, f: F) -> DispatchRecord<R>
    where
        R: Send + 'static,
        F: FnOnce(&mut app::State) -> R + Send + 'static,
    {
        self.spawn_command(move |ctx| f(ctx.state_mut()))
    }

    pub fn spawn_command<R, F>(&self, f: F) -> DispatchRecord<R>
    where
        R: Send + 'static,
        F: FnOnce(&mut CommandHandlerContext) -> R + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();
        let mut pending = PendingDispatch {
            f: Some(f),
            to_caller: tx,
        };

        let b: BoxedPendingDispatch = Box::new(move |ctx| pending.execute(ctx));

        self.to_main.send(b).unwrap();

        DispatchRecord { from_main: rx }
    }
}
