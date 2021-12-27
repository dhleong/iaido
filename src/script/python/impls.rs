use rustpython_vm::{
    function::{FuncArgs, IntoFuncArgs},
    pyobject::{IntoPyObject, ItemProtocol},
};

use crate::script::args::FnArgs;

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
