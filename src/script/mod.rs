mod api;
mod bindings;
mod python;

use std::{cell::RefCell, collections::HashMap, io, path::PathBuf};

use crate::{
    app,
    editing::Id,
    input::{
        commands::CommandHandlerContext, maps::KeyResult, BoxableKeymap, KeyError, KeymapContext,
    },
};
pub use api::ApiManager;

use self::api::{core::ScriptingFnRef, ApiManagerDelegate};

pub trait ScriptingRuntime {
    fn load(&mut self, path: PathBuf) -> io::Result<()>;
    fn invoke(&mut self, f: ScriptingFnRef) -> KeyResult;
}

pub trait ScriptingRuntimeFactory {
    fn handles_file(&self, path: &PathBuf) -> bool;

    fn create(&self, id: Id, app: ApiManagerDelegate) -> Box<dyn ScriptingRuntime + Send>;
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
    pub fn init<K: KeymapContext, KM: BoxableKeymap>(context: &mut K, map: &mut KM) {
        let scripting = context.state().scripting.clone();
        let delegate = context.state().api.as_ref().unwrap().delegate();
        let jobs = &mut context.state_mut().jobs;

        let result = jobs
            .start(move |_| async move {
                let lock = scripting.lock().unwrap();
                lock.load(delegate, "/Users/dhleong/.config/iaido/init.py".to_string())?;
                Ok(())
            })
            .join_interruptably(&mut CommandHandlerContext::new(
                context,
                map,
                "".to_string(),
            ));

        if let Err(e) = result {
            let error = format!("INIT ERR: {:?}", e);
            for line in error.split("\n") {
                context.state_mut().echom(line.to_string());
            }
        }
    }

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
            let runtime = self.runtime_factories[id].create(id, api);
            runtimes.insert(id, runtime);
            runtimes.get_mut(&id).unwrap()
        };

        runtime.load(path_buf)?;

        Ok(id)
    }

    pub fn invoke(&self, f: ScriptingFnRef) -> KeyResult {
        let mut runtimes = self.runtimes.borrow_mut();
        if let Some(runtime) = runtimes.get_mut(&f.runtime) {
            runtime.invoke(f)
        } else {
            // maybe panic?
            Err(KeyError::InvalidInput("No such runtime".to_string()))
        }
    }

    pub fn process<K: BoxableKeymap>(
        mut state: &mut app::State,
        keymap: &mut K,
    ) -> io::Result<bool> {
        if let Some(mut api) = state.api.take() {
            let dirty = api.process(&mut state, keymap)?;
            state.api = Some(api);
            Ok(dirty)
        } else {
            panic!("Re-entrant script API access");
        }
    }
}
