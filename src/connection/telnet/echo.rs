use telnet::TelnetOption;

use crate::connection::{flags::Flag, Flags};

use super::handler::{TelnetHandler, TelnetOptionHandler, TelnetOptionInteractor};

pub struct EchoHandler {
    flags: Flags,
}

impl TelnetHandler for EchoHandler {
    fn on_remote_do(&mut self, _telnet: &mut telnet::Telnet) -> Result<(), telnet::TelnetError> {
        self.flags.remove(Flag::NoEcho);
        Ok(())
    }

    fn on_remote_dont(&mut self, _telnet: &mut telnet::Telnet) -> Result<(), telnet::TelnetError> {
        self.flags.add(Flag::NoEcho);
        Ok(())
    }
}

pub fn create(flags: Flags) -> TelnetOptionHandler {
    TelnetOptionHandler {
        interactor: TelnetOptionInteractor::accept_do(TelnetOption::Echo),
        handler: Box::new(EchoHandler { flags }),
    }
}
