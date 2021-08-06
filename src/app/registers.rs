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
        self.write_or_append(selected_register, range);

        // TODO numbered registers, etc.
    }

    pub fn handle_yanked(&mut self, selected_register: Option<char>, range: CopiedRange) {
        if selected_register.is_none() {
            self.by_name(YANK_REGISTER).write(range.get_contents());
        }

        self.write_or_append(selected_register, range);
    }

    pub fn by_optional_name(&mut self, name: Option<char>) -> &mut Box<dyn Register> {
        if let Some(name) = name {
            self.by_name(name.to_ascii_lowercase())
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

    fn write_or_append(&mut self, name: Option<char>, range: CopiedRange) {
        let register = self.by_optional_name(name);

        let ch = name.unwrap_or(UNNAMED_REGISTER);
        if ch.to_ascii_lowercase() == ch {
            register.write(range.get_contents());
        } else {
            if let Some(existing) = register.read() {
                let mut s = existing.to_owned();
                s.push_str(&range.get_contents());
                register.write(s);
            } else {
                register.write(range.get_contents());
            }
        }
    }
}
