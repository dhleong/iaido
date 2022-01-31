use std::collections::HashMap;

use telnet::TelnetOption;

use crate::editing::Size;

use super::{handler::TelnetOptionHandler, naws, ttype};

pub struct TelnetHandlers {
    handlers: HashMap<u8, TelnetOptionHandler>,
}

impl TelnetHandlers {
    pub fn empty() -> Self {
        Self {
            handlers: Default::default(),
        }
    }

    pub fn with_size(size: Size) -> Self {
        let mut handlers = Self::empty();

        handlers.register(naws::create(size));
        handlers.register(ttype::create());

        handlers
    }
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
