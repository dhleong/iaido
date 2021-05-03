use std::collections::HashMap;

use crate::editing::buffer::CopiedRange;

const UNNAMED_REGISTER: char = '"';

pub trait Register {
    fn read(&self) -> Option<&str>;
    fn write(&mut self, value: String);
}

pub struct RegisterManager {
    registers: HashMap<char, Box<dyn Register>>,
}

impl RegisterManager {
    pub fn new() -> Self {
        // TODO some registers are special
        let registers = HashMap::new();
        Self { registers }
    }

    pub fn handle_yanked(&mut self, selected_register: Option<char>, range: CopiedRange) {
        self.by_name(UNNAMED_REGISTER).write(range.get_contents());
        if let Some(name) = selected_register {
            self.by_name(name).write(range.get_contents());
        }

        // TODO numbered registers, etc.
    }

    pub fn by_optional_name(&mut self, name: Option<char>) -> &mut Box<dyn Register> {
        if let Some(name) = name {
            self.by_name(name)
        } else {
            // TODO default register setting
            self.by_name(UNNAMED_REGISTER)
        }
    }

    pub fn by_name(&mut self, name: char) -> &mut Box<dyn Register> {
        self.registers
            .entry(name)
            .or_insert_with(|| Box::new(InMemoryRegister::new()))
    }
}

struct InMemoryRegister {
    value: Option<String>,
}

impl InMemoryRegister {
    pub fn new() -> Self {
        Self { value: None }
    }
}

impl Register for InMemoryRegister {
    fn read(&self) -> Option<&str> {
        self.value.as_ref().and_then(|v| Some(v.as_str()))
    }

    fn write(&mut self, value: String) {
        self.value = Some(value)
    }
}
