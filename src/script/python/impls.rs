use std::{cmp::Ordering, collections::HashMap};

use rustpython_common::borrow::BorrowValue;
use rustpython_vm::{
    builtins::{PyDict, PyInt, PyNone, PyStr},
    function::{FuncArgs, IntoFuncArgs},
    pyobject::{IdProtocol, IntoPyObject, ItemProtocol, PyObjectRef, TryFromObject},
};

use crate::script::args::{FnArgs, FnReturnable};

impl IntoPyObject for FnArgs {
    fn into_pyobject(self, vm: &rustpython_vm::VirtualMachine) -> PyObjectRef {
        match self {
            FnArgs::None => vm.ctx.none(),
            FnArgs::Bool(b) => vm.ctx.new_bool(b),
            FnArgs::String(s) => vm.ctx.new_str(s),
            FnArgs::Map(m) => {
                let dict = vm.ctx.new_dict();
                for (k, v) in m {
                    dict.set_item(vm.ctx.new_str(k), v.into_pyobject(vm), vm)
                        .expect("Unable to store entry in dict");
                }
                dict.into_pyobject(vm)
            }
        }
    }
}

impl IntoFuncArgs for FnArgs {
    fn into_args(self, vm: &rustpython_vm::VirtualMachine) -> FuncArgs {
        match self {
            FnArgs::None => ().into(),
            other => vec![other.into_pyobject(vm)].into(),
        }
    }
}

fn into_string(obj: &PyObjectRef) -> Option<String> {
    if let Some(s) = obj.downcast_ref::<PyStr>().map(|s| s.to_string()) {
        return Some(s);
    } else {
        None
    }
}

impl TryFromObject for FnArgs {
    fn try_from_object(
        vm: &rustpython_vm::VirtualMachine,
        obj: PyObjectRef,
    ) -> rustpython_vm::pyobject::PyResult<Self> {
        if obj.payload_is::<PyNone>() {
            return Ok(Self::None);
        } else if obj.payload_is::<PyStr>() {
            if let Some(s) = into_string(&obj) {
                return Ok(Self::String(s));
            }
        } else if obj.payload_is::<PyDict>() {
            if let Some(dict) = obj.downcast_ref::<PyDict>() {
                let mut map: HashMap<String, FnArgs> = Default::default();
                for (k, v) in dict.into_iter() {
                    if let Some(k) = into_string(&k) {
                        let v = Self::try_from_object(vm, v)?;
                        map.insert(k, v);
                    }
                }

                return Ok(Self::Map(map));
            }
        }

        // Fallback:
        if obj.is(&vm.ctx.true_value) {
            return Ok(Self::Bool(true));
        } else if obj.is(&vm.ctx.false_value) {
            return Ok(Self::Bool(false));
        }
        Ok(Self::None)
    }
}

pub struct PyFnReturnable(pub PyObjectRef);

impl FnReturnable for PyFnReturnable {
    fn is_string(&self) -> bool {
        self.0.payload_is::<PyStr>()
    }

    fn is_truthy(&self) -> bool {
        if self.0.payload_is::<PyInt>() {
            if let Some(as_int) = self.0.downcast_ref::<PyInt>() {
                return as_int.borrow_value().cmp(&0u32.into()) != Ordering::Equal;
            }
        }

        false
    }

    fn to_string(&self) -> Option<String> {
        self.0.downcast_ref::<PyStr>().map(|s| s.to_string())
    }
}
