use std::{
    io,
    sync::mpsc::{self, Receiver, Sender},
    time::{Duration, Instant},
};

use crate::input::{maps::KeyResult, Key, KeyError, KeySource, KeymapContext};
use crate::{app, input::commands::CommandHandlerContext};

const MAX_PER_FRAME_DURATION: Duration = Duration::from_millis(11);

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

pub enum Dispatchable {
    Executable(BoxedPendingDispatch),
    PendingKey,
}

pub struct Processed {
    executables: usize,
    pub has_key: bool,
}

impl Processed {
    fn key_only() -> Self {
        Self {
            executables: 0,
            has_key: true,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.executables > 0
    }

    fn adding_executables(mut self, count: usize) -> Processed {
        self.executables += count;
        self
    }
}

/// Provides access to the main thread. The sender side may be trivially cloned
pub struct Dispatcher {
    pub sender: DispatchSender,
    rx: Receiver<Dispatchable>,
}

#[derive(Clone)]
pub struct DispatchSender {
    to_main: Sender<Dispatchable>,
}

impl Default for Dispatcher {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        let sender = DispatchSender { to_main: tx };
        Dispatcher { sender, rx }
    }
}

impl Dispatcher {
    /// Process any pending events (IE: does not block waiting for events), stopping early if
    /// MAX_PER_FRAME_DURATION has elapsed
    pub fn process_pending(ctx: &mut CommandHandlerContext) -> io::Result<Processed> {
        let mut executables = 0;
        let mut has_key = false;

        let start = Instant::now();
        loop {
            if let Some(dispatchable) = ctx.state_mut().dispatcher.next_dispatchable()? {
                match dispatchable {
                    Dispatchable::Executable(mut action) => {
                        executables += 1;
                        action(ctx);
                    }
                    Dispatchable::PendingKey => {
                        has_key = true;
                        break;
                    }
                }
            } else {
                break;
            }

            if start.elapsed() >= MAX_PER_FRAME_DURATION {
                break;
            }
        }

        Ok(Processed {
            executables,
            has_key,
        })
    }

    /// Process a "chunk" of events; waits until *some* event is received, then continues to
    /// process any events that come in within the subsequent "frame" duration
    pub fn process_chunk(ctx: &mut CommandHandlerContext) -> io::Result<Processed> {
        match ctx.state_mut().dispatcher.rx.recv() {
            Ok(dispatchable) => {
                match dispatchable {
                    Dispatchable::PendingKey => return Ok(Processed::key_only()),
                    Dispatchable::Executable(mut action) => action(ctx),
                }

                // If we processed *any*, continue processing any pending
                let pending_processed = Dispatcher::process_pending(ctx)?;

                Ok(pending_processed.adding_executables(1))
            }
            Err(_) => Err(io::ErrorKind::UnexpectedEof.into()),
        }
    }

    fn next_dispatchable(&mut self) -> io::Result<Option<Dispatchable>> {
        match self.rx.try_recv() {
            Ok(dispatchable) => Ok(Some(dispatchable)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::UnexpectedEof, e)),
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

        self.dispatch(Dispatchable::Executable(b));

        DispatchRecord { from_main: rx }
    }

    pub fn dispatch(&self, dispatchable: Dispatchable) {
        self.to_main.send(dispatchable).unwrap();
    }
}
