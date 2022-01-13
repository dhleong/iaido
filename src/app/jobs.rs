use std::{collections::HashMap, future::Future, io, time::Duration};

use std::sync::mpsc;

use tokio::task::JoinHandle;

use crate::{
    app,
    editing::{ids::Ids, Id},
    input::{maps::KeyResult, Key, KeyError, KeySource},
};

use super::dispatcher::{DispatchRecord, DispatchSender};

const MAX_TASKS_PER_TICK: u16 = 10;

#[derive(Debug)]
pub enum JobError {
    IO(io::Error),
    Script(String),
}

impl From<io::Error> for JobError {
    fn from(e: io::Error) -> Self {
        JobError::IO(e)
    }
}

impl From<io::ErrorKind> for JobError {
    fn from(e: io::ErrorKind) -> Self {
        JobError::IO(e.into())
    }
}

impl From<JobError> for KeyError {
    fn from(e: JobError) -> Self {
        KeyError::Job(e)
    }
}

pub type JobResult<T = ()> = Result<T, JobError>;

pub enum MainThreadAction {
    JobComplete(Id),
    Err(Id),
}

pub struct JobContext {
    dispatcher: DispatchSender,
}

impl JobContext {
    pub fn run<F>(&self, on_state: F) -> JobResult
    where
        F: (FnOnce(&mut app::State) -> JobResult) + Send + Sync + 'static,
    {
        self.dispatcher.spawn(on_state).background();
        Ok(())
    }

    pub fn spawn<R, F>(&self, f: F) -> DispatchRecord<R>
    where
        R: Send + 'static,
        F: FnOnce(&mut app::State) -> R + Send + 'static,
    {
        self.dispatcher.spawn(f)
    }
}

#[must_use = "If not using with join_interruptably, prefer spawn()"]
pub struct JobRecord {
    pub id: Id,
    await_channel: mpsc::Receiver<Option<JobError>>,
    handle: JoinHandle<()>,
}

impl JobRecord {
    /// Wait for this Job to finish, returning a KeyResult representing
    /// the result of the Job. This fn acts like it's blocking input,
    /// but still allows the UI to redraw and also accepts <ctrl-c> input
    /// from the user, which triggers a cancellation of this Job, returning
    /// [`KeyError:Interrupted`] to the caller.
    pub fn join_interruptably<K: KeySource>(&self, keys: &mut K) -> KeyResult {
        loop {
            match self.await_channel.recv_timeout(Duration::from_millis(10)) {
                Ok(None) => return Ok(()),
                Ok(Some(e)) => return Err(e.into()),
                Err(_) => {} // timeout
            }

            if keys.poll_key(Duration::from_millis(10))? {
                match keys.next_key()? {
                    Some(key) if key == Key::from("<c-c>") => {
                        self.handle.abort();
                        return Err(KeyError::Interrupted);
                    }
                    _ => {}
                }
            }
        }
    }
}

pub struct Jobs {
    ids: Ids,
    dispatcher: DispatchSender,
    to_main: mpsc::Sender<MainThreadAction>,
    from_job: mpsc::Receiver<MainThreadAction>,
    jobs: HashMap<Id, JobRecord>,
}

impl Jobs {
    pub fn new(dispatcher: DispatchSender) -> Self {
        let (tx, rx) = mpsc::channel::<MainThreadAction>();
        Self {
            ids: Ids::new(),
            dispatcher,
            to_main: tx,
            from_job: rx,
            jobs: HashMap::new(),
        }
    }

    /// Process messages from Jobs meant to be handled on the main thread.
    /// This should probably be called in looper.
    pub fn process(state: &mut app::State) -> JobResult<bool> {
        let mut dirty = false;
        for _ in 0..MAX_TASKS_PER_TICK {
            match state.jobs.next_action()? {
                None => return Ok(dirty),

                Some(MainThreadAction::JobComplete(id)) => {
                    state.jobs.jobs.remove(&id);
                }
                Some(MainThreadAction::Err(id)) => {
                    match state.jobs.jobs.get_mut(&id) {
                        Some(job) => {
                            match job.await_channel.recv() {
                                Err(_) => {} // already handled
                                Ok(None) => panic!("Expected an error from job, but got success"),
                                Ok(Some(e)) => return Err(e.into()),
                            };
                        }

                        _ => {} // job 404; ignore (already handled)
                    };
                }
            };

            dirty = true;
        }

        Ok(dirty)
    }

    pub fn cancel_all(&mut self) {
        for record in self.jobs.values_mut() {
            record.handle.abort();
        }
        self.jobs.clear();
    }

    /// Start a job, returning ownership of its JobRecord so you can
    /// interact with it directly.
    pub fn start<T, F>(&mut self, task: T) -> JobRecord
    where
        T: Send + 'static + FnOnce(JobContext) -> F,
        F: Future<Output = JobResult> + Send + 'static,
    {
        let id = self.ids.next();
        let to_main = self.to_main.clone();
        let context = JobContext {
            dispatcher: self.dispatcher.clone(),
        };

        let (to_job, await_channel) = mpsc::channel();

        let handle = tokio::spawn(async move {
            match task(context).await {
                Err(e) => {
                    let _ = to_main.send(MainThreadAction::Err(id));
                    let _ = to_job.send(Some(e));
                }
                _ => {
                    // success!
                    let _ = to_job.send(None);
                }
            };
            let _ = to_main.send(MainThreadAction::JobComplete(id));
        });
        JobRecord {
            id,
            handle,
            await_channel,
        }
    }

    /// Start a job in the background, returning its Id
    #[allow(unused)]
    pub fn spawn<T, F>(&mut self, task: T) -> Id
    where
        T: Send + 'static + FnOnce(JobContext) -> F,
        F: Future<Output = JobResult> + Send + 'static,
    {
        let job = self.start(task);
        let id = job.id;
        self.jobs.insert(id, job);

        return id;
    }

    fn next_action(&mut self) -> io::Result<Option<MainThreadAction>> {
        match self.from_job.try_recv() {
            Ok(action) => Ok(Some(action)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }
}
