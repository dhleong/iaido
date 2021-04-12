use std::path::PathBuf;

use rustpython_vm as vm;
use vm::pyobject::{ItemProtocol, PyResult};

use crate::script::{bindings::ScriptFile, ScriptingManager};

const INIT_MODULE: &str = "_init";

pub trait ModuleContained {
    fn ensure_module(&self, vm: &vm::VirtualMachine) -> PyResult<()>;
    fn package_name(&self) -> Option<String>;
}

impl ModuleContained for ScriptFile {
    fn ensure_module(&self, vm: &vm::VirtualMachine) -> PyResult<()> {
        let package = if let Some(package) = self.package_name() {
            package
        } else {
            return Ok(());
        };

        let modules = vm.get_attribute(vm.sys_module.clone(), "modules").unwrap();
        if modules.get_item(&package, vm).is_ok() {
            return Ok(());
        }

        let parent_path = PathBuf::from(&self.path)
            .parent()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let user_module = vm.ctx.new_dict();
        user_module.set_item("__package__", vm.ctx.new_str(package.clone()), vm)?;
        user_module.set_item(
            "__path__",
            vm.ctx.new_list(vec![vm.ctx.new_str(parent_path)]),
            vm,
        )?;
        modules.set_item(package.clone(), vm.new_module(&package, user_module), vm)?;

        Ok(())
    }

    fn package_name(&self) -> Option<String> {
        if let Some(dir) = ScriptingManager::config_dir() {
            let config_path = dir.to_string_lossy().to_string();
            if self.path.find(&config_path).is_some() {
                return Some(INIT_MODULE.to_string());
            }
        }

        if let Some(parent) = PathBuf::from(&self.path).parent() {
            if let Some(name) = parent.file_name() {
                return Some(name.to_string_lossy().to_string());
            }
        }

        return None;
    }
}
