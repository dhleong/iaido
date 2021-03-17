mod api;
mod python;

use std::{cell::RefCell, collections::HashMap, io, path::PathBuf};

use crate::{app, editing::Id};
pub use api::ApiManager;

use self::api::ApiManagerDelegate;

pub trait ScriptingRuntime {
    fn load(&mut self, path: PathBuf) -> io::Result<()>;
}

pub trait ScriptingRuntimeFactory {
    fn handles_file(&self, path: &PathBuf) -> bool;

    fn create(&self, app: ApiManagerDelegate) -> Box<dyn ScriptingRuntime + Send>;
}

pub struct ScriptingManager {
    runtime_factories: Vec<Box<dyn ScriptingRuntimeFactory + Send>>,
    runtimes: RefCell<HashMap<Id, Box<dyn ScriptingRuntime + Send>>>,
}

impl Default for ScriptingManager {
    fn default() -> Self {
        let mut runtime_factories: Vec<Box<dyn ScriptingRuntimeFactory + Send>> = vec![];

        if cfg!(feature = "python") {
            runtime_factories.push(Box::new(python::PythonScriptingRuntimeFactory));
        }

        Self {
            runtime_factories,
            runtimes: RefCell::new(HashMap::new()),
        }
    }
}

impl ScriptingManager {
    pub fn load(&self, api: ApiManagerDelegate, path: String) -> io::Result<Id> {
        let path_buf = PathBuf::from(path.clone());
        if !path_buf.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, path));
        }

        let mut runtime_id = None;
        for (id, factory) in self.runtime_factories.iter().enumerate() {
            if factory.handles_file(&path_buf) {
                runtime_id = Some(id);
                break;
            }
        }

        let id = if let Some(id) = runtime_id {
            id
        } else {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "No scripting engine available that supports {}",
                    path_buf
                        .file_name()
                        .and_then(|name| Some(name.to_string_lossy().to_string()))
                        .unwrap_or(path),
                ),
            ));
        };

        let mut runtimes = self.runtimes.borrow_mut();
        let runtime = if let Some(runtime) = runtimes.get_mut(&id) {
            runtime
        } else {
            let runtime = self.runtime_factories[id].create(api);
            runtimes.insert(id, runtime);
            runtimes.get_mut(&id).unwrap()
        };

        runtime.load(path_buf)?;

        Ok(id)
    }

    pub fn process(mut state: &mut app::State) -> io::Result<bool> {
        if let Some(mut api) = state.api.take() {
            let dirty = api.process(&mut state)?;
            state.api = Some(api);
            Ok(dirty)
        } else {
            panic!("Re-entrant script API access");
        }
    }
}
