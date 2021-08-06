use std::collections::HashMap;

use crate::editing::buffer::CopiedRange;

mod blackhole;
mod clipboard;
mod memory;

use memory::InMemoryRegister;

use self::blackhole::BlackholeRegister;
use self::clipboard::ClipboardRegister;

const UNNAMED_REGISTER: char = '"';
const YANK_REGISTER: char = '0';

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

        // Some registers are special:
        registers.insert('*', ClipboardRegister::new());
        registers.insert('_', BlackholeRegister::new());

        Self { registers }
    }

    pub fn handle_deleted(&mut self, selected_register: Option<char>, range: CopiedRange) {
        self.by_optional_name(selected_register)
            .write(range.get_contents());

        // TODO replace/vs append for letter registers
        // TODO numbered registers, etc.
    }

    pub fn handle_yanked(&mut self, selected_register: Option<char>, range: CopiedRange) {
        self.by_optional_name(selected_register)
            .write(range.get_contents());

        if selected_register.is_none() {
            self.by_name(YANK_REGISTER).write(range.get_contents());
        }
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
