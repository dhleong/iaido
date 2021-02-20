use std::{future::Future, io};

use std::sync::mpsc;

pub enum MainThreadAction {
    Echo(String),
    Err(io::Error),
}

pub struct JobContext {
    to_main: mpsc::Sender<MainThreadAction>,
}

impl JobContext {
    pub fn echo(&self, message: String) -> io::Result<()> {
        self.send(MainThreadAction::Echo(message))
    }

    pub fn send(&self, action: MainThreadAction) -> io::Result<()> {
        match self.to_main.send(action) {
            Ok(_) => Ok(()),
            Err(e) => Err(io::Error::new(io::ErrorKind::BrokenPipe, e))
        }
    }
}

pub struct Jobs {
    to_main: mpsc::Sender<MainThreadAction>,
    from_job: mpsc::Receiver<MainThreadAction>,
}

impl Jobs {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<MainThreadAction>();
        Self {
            to_main: tx,
            from_job: rx,
        }
    }

    pub fn process(&mut self) -> io::Result<Option<MainThreadAction>> {
        match self.from_job.try_recv() {
            Ok(action) => Ok(Some(action)),
            Err(mpsc::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, e)),
        }
    }

    pub fn spawn<T, F>(&mut self, task: T)
    where
        T: Send + 'static + FnOnce(JobContext) -> F,
        F: Future<Output = io::Result<()>> + Send + 'static
    {
        let to_main = self.to_main.clone();
        let context = JobContext { to_main: to_main.clone() };
        tokio::spawn(async move {
            match task(context).await {
                Err(e) => {
                    let _ = to_main.send(MainThreadAction::Err(e));
                },
                _ => {} // success!
            };
        });
    }
}
