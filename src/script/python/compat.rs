use rustpython_vm as vm;
use vm::{
    builtins::{PyStr, PyType},
    function::FuncArgs,
    pyobject::ItemProtocol,
};

use crate::script::api::core2::IaidoCore;

/// By default, the warnings module writes to stderr, which messes up the tui
/// We may want to consider completely swapping out sys.stderr/sys.stdout...
fn patch_warnings_module(iaido: IaidoCore, vm: &vm::VirtualMachine) {
    let warnings = vm
        .get_attribute(vm.sys_module.clone(), "modules")
        .expect("Could not access modules")
        .get_item("_warnings", vm)
        .expect("Could not find _warnings");

    let f = vm.ctx.new_function(
        "_warn",
        move |mut args: FuncArgs, vm: &vm::VirtualMachine| {
            let message = args.take_positional().expect("No message");
            let category = if let Some(category) = args.take_positional_keyword("category") {
                let category = category.downcast_exact::<PyType>(vm).unwrap();
                category.name.clone()
            } else {
                vm.ctx.exceptions.user_warning.name.clone()
            };

            iaido.echo(format!(
                "[py][warn] {}: {}",
                category,
                message.downcast_exact::<PyStr>(vm).unwrap().to_string()
            ));
        },
    );

    vm.set_attr(&warnings, "warn", f)
        .expect("Could not stub warn");
}

pub fn apply_compat(iaido: IaidoCore, vm: &vm::VirtualMachine) {
    patch_warnings_module(iaido, vm);
}
