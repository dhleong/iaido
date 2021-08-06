use crate::app::registers::Register;

pub struct BlackholeRegister;

impl BlackholeRegister {
    pub fn new() -> Box<dyn Register> {
        Box::new(BlackholeRegister)
    }
}

impl Register for BlackholeRegister {
    fn read(&mut self) -> Option<&str> {
        None
    }

    fn write(&mut self, _value: String) {
        // ignore
    }
}
