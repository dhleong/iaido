use rustpython_vm as vm;
use vm::{
    builtins::PyTypeRef,
    pyobject::{PyValue, StaticType},
};

#[vm::pyclass(module = "iaido", name = "CurrentObjects")]
#[derive(Debug)]
pub struct CurrentObjects {}

impl PyValue for CurrentObjects {
    fn class(_vm: &vm::VirtualMachine) -> &PyTypeRef {
        Self::static_type()
    }
}

#[vm::pyimpl]
impl CurrentObjects {
    #[pyproperty(name = "buffer")]
    pub fn buffer(&self) -> BufferPyObject {
        BufferPyObject {}
    }
}

#[vm::pyclass(module = false, name = "Buffer")]
#[derive(Debug)]
pub struct BufferPyObject {}

impl PyValue for BufferPyObject {
    fn class(_vm: &vm::VirtualMachine) -> &PyTypeRef {
        Self::static_type()
    }
}
