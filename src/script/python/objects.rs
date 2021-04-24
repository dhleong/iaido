use std::fmt;

use rustpython_vm as vm;
use vm::{
    builtins::PyTypeRef,
    pyobject::{PyClassImpl, PyRef, PyResult, PyValue, StaticType},
};

use crate::{
    editing::Id,
    script::api::objects::{BufferApiObject, ConnectionApiObject, CurrentObjects},
};

use super::util::KeyResultConvertible;

#[vm::pyclass(module = "iaido", name = "CurrentObjects")]
pub struct CurrentPyObjects {
    api: CurrentObjects,
}

impl CurrentPyObjects {
    pub fn new(api: CurrentObjects) -> Self {
        Self { api }
    }
}

impl fmt::Debug for CurrentPyObjects {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "CurrentObjects")
    }
}

impl PyValue for CurrentPyObjects {
    fn class(_vm: &vm::VirtualMachine) -> &PyTypeRef {
        Self::static_type()
    }
}

#[vm::pyimpl]
impl CurrentPyObjects {
    #[pyproperty]
    pub fn buffer(&self, vm: &vm::VirtualMachine) -> PyResult<BufferPyObject> {
        let api = self.api.buffer().wrap_err(vm)?;
        Ok(BufferPyObject { api })
    }

    #[pyproperty(setter)]
    pub fn set_buffer(
        &self,
        buffer: PyRef<BufferPyObject>,
        vm: &vm::VirtualMachine,
    ) -> PyResult<()> {
        self.api.set_buffer(&buffer.api).wrap_err(vm)
    }

    #[pyproperty]
    pub fn connection(&self, vm: &vm::VirtualMachine) -> PyResult<Option<ConnectionPyObject>> {
        if let Some(api) = self.api.connection().wrap_err(vm)? {
            Ok(Some(ConnectionPyObject { api }))
        } else {
            Ok(None)
        }
    }
}

#[vm::pyclass(module = "iaido", name = "Buffer")]
pub struct BufferPyObject {
    pub api: BufferApiObject,
}

impl fmt::Debug for BufferPyObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.api.fmt(f)
    }
}

impl PyValue for BufferPyObject {
    fn class(_vm: &vm::VirtualMachine) -> &PyTypeRef {
        Self::static_type()
    }
}

#[vm::pyimpl]
impl BufferPyObject {
    #[pyproperty]
    pub fn id(&self) -> Id {
        self.api.id
    }

    #[pyproperty]
    pub fn name(&self, vm: &vm::VirtualMachine) -> PyResult<String> {
        self.api.name().wrap_err(vm)
    }

    #[pymethod(magic)]
    fn repr(zelf: PyRef<Self>) -> PyResult<String> {
        Ok(format!("{:?}", zelf.api))
    }
}

#[vm::pyclass(module = "iaido", name = "Connection")]
pub struct ConnectionPyObject {
    pub api: ConnectionApiObject,
}

impl fmt::Debug for ConnectionPyObject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.api.fmt(f)
    }
}

impl PyValue for ConnectionPyObject {
    fn class(_vm: &vm::VirtualMachine) -> &PyTypeRef {
        Self::static_type()
    }
}

#[vm::pyimpl]
impl ConnectionPyObject {
    #[pyproperty]
    pub fn id(&self) -> Id {
        self.api.id
    }

    #[pymethod(magic)]
    fn repr(zelf: PyRef<Self>) -> PyResult<String> {
        Ok(format!("{:?}", zelf.api))
    }

    #[pymethod]
    fn close(&self, vm: &vm::VirtualMachine) -> PyResult<()> {
        self.api.close().wrap_err(vm)
    }
}

pub fn init_objects(vm: &vm::VirtualMachine) {
    ConnectionPyObject::make_class(&vm.ctx);
    CurrentPyObjects::make_class(&vm.ctx);
    BufferPyObject::make_class(&vm.ctx);
}
