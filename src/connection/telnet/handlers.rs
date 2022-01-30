use std::collections::HashMap;

use telnet::TelnetOption;

use super::handler::TelnetOptionHandler;

#[derive(Default)]
pub struct TelnetHandlers {
    handlers: HashMap<u8, TelnetOptionHandler>,
}

impl TelnetHandlers {
    pub fn get_mut(&mut self, option: &TelnetOption) -> Option<&mut TelnetOptionHandler> {
        self.handlers.get_mut(&option.as_byte())
    }

    pub fn register(&mut self, handler: TelnetOptionHandler) {
        self.handlers
            .insert(handler.interactor.option.as_byte(), handler);
    }
}
