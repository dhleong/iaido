mod api;
mod bindings;

#[cfg(feature = "python")]
mod python;

use dirs;
use std::{cell::RefCell, collections::HashMap, io, path::PathBuf};

use crate::{
    app::jobs::{JobError, JobResult},
    editing::Id,
    input::{commands::CommandHandlerContext, BoxableKeymap, KeymapContext},
};
pub use api::manager::ApiManagerRpc;

use self::api::{fns::ScriptingFnRef, manager::ApiManagerDelegate2};

pub trait ScriptingRuntime {
    fn load(&mut self, path: PathBuf) -> JobResult;
    fn invoke(&mut self, f: ScriptingFnRef) -> JobResult;
}

pub trait ScriptingRuntimeFactory {
    fn handles_file(&self, path: &PathBuf) -> bool;

    fn create(&self, id: Id, app: ApiManagerDelegate2) -> Box<dyn ScriptingRuntime + Send>;
}

pub struct ScriptingManager {
    runtime_factories: Vec<Box<dyn ScriptingRuntimeFactory + Send>>,
    runtimes: RefCell<HashMap<Id, Box<dyn ScriptingRuntime + Send>>>,
}

impl Default for ScriptingManager {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut runtime_factories: Vec<Box<dyn ScriptingRuntimeFactory + Send>> = vec![];

        #[cfg(feature = "python")]
        runtime_factories.push(Box::new(python::PythonScriptingRuntimeFactory));

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
                for path in lock.find_init_scripts() {
                    lock.load(delegate.clone(), path)?;
                }
                Ok(())
            })
            .join_interruptably(&mut CommandHandlerContext::new(
                context,
                map,
                "".to_string(),
            ));

        if let Err(e) = result {
            context.state_mut().echom("INIT ERROR");
            context.state_mut().echom_error(e);
        }
    }

    pub fn load(&self, api: ApiManagerDelegate2, path: String) -> JobResult<Id> {
        let path_buf = PathBuf::from(path.clone());
        if !path_buf.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, path).into());
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
            )
            .into());
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

    pub fn invoke(&self, f: ScriptingFnRef) -> JobResult {
        let mut runtimes = self.runtimes.borrow_mut();
        if let Some(runtime) = runtimes.get_mut(&f.runtime) {
            runtime.invoke(f)
        } else {
            // maybe panic?
            Err(JobError::Script("No such runtime".to_string()))
        }
    }

    pub fn process(context: &mut CommandHandlerContext) -> io::Result<bool> {
        if let Some(mut api) = context.state_mut().api.take() {
            let dirty = api.process(context)?;
            context.state_mut().api = Some(api);
            Ok(dirty)
        } else {
            panic!("Re-entrant script API access");
        }
    }

    pub fn config_dir() -> Option<PathBuf> {
        if let Some(mut dir) = dirs::home_dir() {
            dir.push(".config");
            dir.push("iaido");
            Some(dir)
        } else {
            None
        }
    }

    fn find_init_scripts(&self) -> Vec<String> {
        if let Some(dir) = ScriptingManager::config_dir() {
            if let Ok(contents) = dir.read_dir() {
                return contents
                    .filter_map(|f| {
                        if let Ok(entry) = f {
                            if let Some(name) = entry.file_name().to_str() {
                                if name.starts_with("init.") && self.supports_file(&entry.path()) {
                                    return Some(entry.path().to_string_lossy().to_string());
                                }
                            }
                        }
                        None
                    })
                    .collect();
            }
        }

        vec![]
    }

    fn supports_file(&self, path: &PathBuf) -> bool {
        self.runtime_factories
            .iter()
            .any(|factory| factory.handles_file(path))
    }
}
