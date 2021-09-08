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
const SMALL_DELETE_REGISTER: char = '-';

const NUMBERED_REGISTERS: [char; 9] = ['1', '2', '3', '4', '5', '6', '7', '8', '9'];

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
        if selected_register.is_none() {
            if range.is_multi_line() {
                // Rotate numbered registers (3 gets 2, 2 gets 1, 1 gets `range`, etc)
                for i in (1..NUMBERED_REGISTERS.len()).rev() {
                    if let Some(previous) = self.try_read(NUMBERED_REGISTERS[i - 1]) {
                        let s = previous.to_string();
                        self.by_name(NUMBERED_REGISTERS[i]).write(s);
                    }
                }
                self.by_name(NUMBERED_REGISTERS[0])
                    .write(range.get_contents());
            } else {
                self.by_name(SMALL_DELETE_REGISTER)
                    .write(range.get_contents());
            }
        }

        self.write_or_append(selected_register, range);
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

    fn by_name_opt(&mut self, name: char) -> Option<&mut Box<dyn Register>> {
        self.registers.get_mut(&name)
    }

    fn try_read(&mut self, name: char) -> Option<&str> {
        if let Some(register) = self.by_name_opt(name) {
            return register.read();
        }

        return None;
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

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;

    #[cfg(test)]
    mod numbered_registers {
        use super::*;
        use indoc::indoc;

        #[test]
        fn delete_into_numbered_registers() {
            let ctx = window(indoc! {"
                Take my |love
                Take my land
            "});
            let (_, mut state) = ctx.feed_vim_for_state("dddd");

            // Register 1 should have the most recent delete (IE: line 2)
            let contents = state
                .registers
                .by_name('1')
                .read()
                .expect("Register 1 should have contents set");
            assert_eq!(contents, "\nTake my land\n");

            // Register 2 should have the older delete rotated into it (IE: line 1)
            let contents = state
                .registers
                .by_name('2')
                .read()
                .expect("Register 2 should have contents set");
            assert_eq!(contents, "\nTake my love\n");

            // Register 3 should not yet exist
            let register3 = state.registers.by_name_opt('3');
            assert!(register3.is_none());
        }
    }
}
