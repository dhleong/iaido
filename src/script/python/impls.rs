use rustpython_vm::{
    builtins::PyStr,
    function::{FuncArgs, IntoFuncArgs},
    pyobject::{IntoPyObject, ItemProtocol, PyObjectRef},
};

use crate::script::args::{FnArgs, FnReturnable};

impl IntoFuncArgs for FnArgs {
    fn into_args(self, vm: &rustpython_vm::VirtualMachine) -> FuncArgs {
        match self {
            FnArgs::None => ().into(),
            FnArgs::Map(m) => {
                let dict = vm.ctx.new_dict();
                for (k, v) in m {
                    dict.set_item(vm.ctx.new_str(k), vm.ctx.new_str(v), vm)
                        .expect("Unable to store entry in dict");
                }
                let obj = dict.into_pyobject(vm);
                vec![obj].into()
            }
        }
    }
}

pub struct PyFnReturnable(pub PyObjectRef);

impl FnReturnable for PyFnReturnable {
    fn is_string(&self) -> bool {
        self.0.payload_is::<PyStr>()
    }

    fn to_string(&self) -> Option<String> {
        self.0.downcast_ref::<PyStr>().map(|s| s.to_string())
    }
}
