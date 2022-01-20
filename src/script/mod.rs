mod api;
mod args;
mod bindings;
mod fns;
mod poly;

#[cfg(feature = "python")]
mod python;

use dirs;
use std::{
    cell::RefCell,
    collections::HashMap,
    io,
    path::{Path, PathBuf},
};

use crate::{
    app::jobs::{JobError, JobResult},
    editing::Id,
    input::{commands::CommandHandlerContext, BoxableKeymap, KeymapContext},
};

use self::{api::ApiDelegate, args::FnArgs, fns::ScriptingFnRef};

pub trait ScriptingRuntime {
    fn load(&mut self, path: &Path) -> JobResult;
    fn invoke(&mut self, f: ScriptingFnRef, args: FnArgs) -> JobResult<FnArgs>;
}

pub trait ScriptingRuntimeFactory {
    fn handles_file(&self, path: &Path) -> bool;

    fn create(&self, id: Id, app: ApiDelegate) -> Box<dyn ScriptingRuntime + Send>;
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
        let init_scripts = {
            let scripting = context.state().scripting.clone();
            let lock = scripting.lock().unwrap();
            lock.find_init_scripts()
        };

        Self::load_scripts(context, map, init_scripts)
    }

    pub fn load_script<K: KeymapContext, KM: BoxableKeymap>(
        context: &mut K,
        map: &mut KM,
        script: PathBuf,
    ) {
        context
            .state_mut()
            .current_buffer_mut()
            .config_mut()
            .loaded_script = Some(script.clone());
        Self::load_scripts(context, map, vec![script])
    }

    pub fn load_scripts<K: KeymapContext, KM: BoxableKeymap>(
        context: &mut K,
        map: &mut KM,
        scripts: Vec<PathBuf>,
    ) {
        let scripting = context.state().scripting.clone();
        let delegate = ApiDelegate::from(context.state());
        let jobs = &mut context.state_mut().jobs;

        let result = jobs
            .start(move |_| async move {
                let lock = scripting.lock().unwrap();
                let count = scripts.len();
                for path in scripts {
                    crate::info!("Loading {}", path.to_string_lossy());
                    lock.load(delegate.clone(), &path)?;
                }
                crate::info!("Loaded {} scripts", count);
                Ok(())
            })
            .join_interruptably(&mut CommandHandlerContext::new_blank(context, map));

        if let Err(e) = result {
            context.state_mut().echom("INIT ERROR");
            context.state_mut().echom_error(e);
        }
    }

    pub fn load(&self, api: ApiDelegate, path: &Path) -> JobResult<Id> {
        if !path.exists() {
            return Err(io::Error::new(io::ErrorKind::NotFound, path.to_string_lossy()).into());
        }

        let mut runtime_id = None;
        for (id, factory) in self.runtime_factories.iter().enumerate() {
            if factory.handles_file(&path) {
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
                    path.file_name()
                        .and_then(|name| Some(name.to_string_lossy()))
                        .unwrap_or_else(|| path.to_string_lossy()),
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

        runtime.load(path)?;

        Ok(id)
    }

    pub fn invoke(&self, f: ScriptingFnRef, args: FnArgs) -> JobResult<FnArgs> {
        let mut runtimes = self.runtimes.borrow_mut();
        if let Some(runtime) = runtimes.get_mut(&f.runtime) {
            runtime.invoke(f, args)
        } else {
            // maybe panic?
            Err(JobError::Script("No such runtime".to_string()))
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

    fn find_init_scripts(&self) -> Vec<PathBuf> {
        if let Some(dir) = ScriptingManager::config_dir() {
            if let Ok(contents) = dir.read_dir() {
                return contents
                    .filter_map(|f| {
                        if let Ok(entry) = f {
                            if let Some(name) = entry.file_name().to_str() {
                                let path = entry.path();
                                if name.starts_with("init.") && self.supports_file(&path) {
                                    return Some(path);
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
