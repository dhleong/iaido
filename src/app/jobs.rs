use std::{collections::HashMap, future::Future, io, time::Duration};

use std::sync::mpsc;

use tokio::task::JoinHandle;

use crate::{
    app,
    editing::{ids::Ids, Id},
    input::{maps::KeyResult, Key, KeyError, KeySource},
};

const MAX_TASKS_PER_TICK: u16 = 10;

type StateAction = dyn (FnOnce(&mut app::State) -> io::Result<()>) + Send + Sync;

pub enum MainThreadAction {
    OnState(Box<StateAction>),
    Echo(String),

    JobComplete(Id),
    Err(io::Error),
}

pub struct JobContext {
    to_main: mpsc::Sender<MainThreadAction>,
}

impl JobContext {
    pub fn run<F>(&self, on_state: F) -> io::Result<()>
    where
        F: (FnOnce(&mut app::State) -> io::Result<()>) + Send + Sync + 'static,
    {
        self.send(MainThreadAction::OnState(Box::new(on_state)))
    }

    #[allow(unused)]
    pub fn echo(&self, message: String) -> io::Result<()> {
        self.send(MainThreadAction::Echo(message))
    }

    pub fn send(&self, action: MainThreadAction) -> io::Result<()> {
        match self.to_main.send(action) {
            Ok(_) => Ok(()),
            Err(e) => Err(io::Error::new(io::ErrorKind::BrokenPipe, e)),
        }
    }
}

#[must_use = "If not using with join_interruptably, prefer spawn()"]
pub struct JobRecord {
    pub id: Id,
    await_channel: mpsc::Receiver<()>,
    handle: JoinHandle<()>,
}

impl JobRecord {
    pub fn join_interruptably<K: KeySource>(&self, keys: &mut K) -> KeyResult {
        loop {
            if let Ok(_) = self.await_channel.recv_timeout(Duration::from_millis(10)) {
                return Ok(());
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
    to_main: mpsc::Sender<MainThreadAction>,
    from_job: mpsc::Receiver<MainThreadAction>,
    jobs: HashMap<Id, JobRecord>,
}

impl Jobs {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<MainThreadAction>();
        Self {
            ids: Ids::new(),
            to_main: tx,
            from_job: rx,
            jobs: HashMap::new(),
        }
    }

    pub fn process(state: &mut app::State) -> io::Result<()> {
        for _ in 0..MAX_TASKS_PER_TICK {
            match state.jobs.process_once()? {
                None => return Ok(()),

                Some(MainThreadAction::OnState(closure)) => closure(state)?,
                Some(MainThreadAction::Echo(msg)) => state.echo(msg.into()),

                Some(MainThreadAction::JobComplete(id)) => {
                    state.jobs.jobs.remove(&id);
                }
                Some(MainThreadAction::Err(e)) => return Err(e.into()),
            };
        }

        Ok(())
    }

    pub fn cancel_all(&mut self) {
        for record in self.jobs.values_mut() {
            record.handle.abort();
        }
        self.jobs.clear();
    }

    pub fn process_once(&mut self) -> io::Result<Option<MainThreadAction>> {
        match self.from_job.try_recv() {
            Ok(action) => Ok(Some(action)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    /// Start a job, returning ownership of its JobRecord so you can
    /// interact with it directly.
    pub fn start<T, F>(&mut self, task: T) -> JobRecord
    where
        T: Send + 'static + FnOnce(JobContext) -> F,
        F: Future<Output = io::Result<()>> + Send + 'static,
    {
        let id = self.ids.next();
        let to_main = self.to_main.clone();
        let context = JobContext {
            to_main: to_main.clone(),
        };

        let (to_job, await_channel) = mpsc::channel();

        let handle = tokio::spawn(async move {
            match task(context).await {
                Err(e) => {
                    let _ = to_main.send(MainThreadAction::Err(e));
                }
                _ => {} // success!
            };
            let _ = to_main.send(MainThreadAction::JobComplete(id));
            let _ = to_job.send(());
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
        F: Future<Output = io::Result<()>> + Send + 'static,
    {
        let job = self.start(task);
        let id = job.id;
        self.jobs.insert(id, job);

        return id;
    }
}
