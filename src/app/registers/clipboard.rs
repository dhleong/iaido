use clipboard::{ClipboardContext, ClipboardProvider};

use crate::log;

use super::{memory::InMemoryRegister, Register};

pub struct ClipboardRegister {
    clipboard: ClipboardContext,
    last_value: Option<String>,
}

impl ClipboardRegister {
    pub fn new() -> Box<dyn Register> {
        match ClipboardProvider::new() {
            Ok(clipboard) => Box::new(Self {
                clipboard,
                last_value: None,
            }),

            Err(e) => {
                log!(log::LogLevel::Warn, "Unable to init clipboard: {:?}", e);

                Box::new(InMemoryRegister::new())
            }
        }
    }
}

impl Register for ClipboardRegister {
    fn read(&mut self) -> Option<&str> {
        if let Ok(value) = self.clipboard.get_contents() {
            self.last_value = Some(value);
            self.last_value.as_ref().and_then(|v| Some(v.as_str()))
        } else {
            self.last_value = None;
            None
        }
    }

    fn write(&mut self, value: String) {
        if let Err(e) = self.clipboard.set_contents(value.clone()) {
            log!(log::LogLevel::Error, "Error writing to clipboard: {:?}", e);
        } else {
            log!(log::LogLevel::Info, "clipboard <- `{}`", value);
        }
    }
}
