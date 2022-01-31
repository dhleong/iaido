use telnet::TelnetOption;

use crate::editing::Size;

use super::handler::{TelnetHandler, TelnetOptionHandler, TelnetOptionInteractor};

pub struct NawsHandler {
    pub size: Size,
}

impl NawsHandler {
    fn send_size(&mut self, telnet: &mut telnet::Telnet) -> Result<(), telnet::TelnetError> {
        let w = self.size.w.to_be_bytes();
        let h = self.size.h.to_be_bytes();

        crate::info!("## TELNET > IAC SB NAWS {} {}", self.size.w, self.size.h);

        let message = [w, h].concat();
        telnet.subnegotiate(TelnetOption::NAWS, &message)
    }
}

impl TelnetHandler for NawsHandler {
    fn on_remote_do(&mut self, telnet: &mut telnet::Telnet) -> Result<(), telnet::TelnetError> {
        self.send_size(telnet)
    }

    fn on_resize(
        &mut self,
        telnet: &mut telnet::Telnet,
        size: Size,
    ) -> Result<(), telnet::TelnetError> {
        self.size = size;
        self.send_size(telnet)
    }
}

pub fn create(size: Size) -> TelnetOptionHandler {
    TelnetOptionHandler {
        interactor: TelnetOptionInteractor::accept_do(TelnetOption::NAWS),
        handler: Box::new(NawsHandler { size }),
    }
}
