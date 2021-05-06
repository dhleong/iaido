use std::collections::HashMap;

use crate::editing::buffer::CopiedRange;

mod clipboard;
mod memory;

use memory::InMemoryRegister;

use self::clipboard::ClipboardRegister;

const UNNAMED_REGISTER: char = '"';

pub trait Register {
    fn read(&mut self) -> Option<&str>;
    fn write(&mut self, value: String);
}

pub struct RegisterManager {
    registers: HashMap<char, Box<dyn Register>>,
}

impl RegisterManager {
    pub fn new() -> Self {
        let mut registers = HashMap::new();

        // TODO Some registers are special
        registers.insert('*', ClipboardRegister::new());

        Self { registers }
    }

    pub fn handle_yanked(&mut self, selected_register: Option<char>, range: CopiedRange) {
        self.by_name(selected_register.unwrap_or(UNNAMED_REGISTER))
            .write(range.get_contents());

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
