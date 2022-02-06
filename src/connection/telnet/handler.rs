use std::io;

use telnet::{self, Telnet, TelnetError, TelnetOption};

use crate::editing::Size;

pub trait TelnetHandler {
    fn negotiate(&mut self, action: &telnet::Action, telnet: &mut Telnet) -> io::Result<()> {
        let result = match action {
            telnet::Action::Do => self.on_remote_do(telnet),
            telnet::Action::Dont => self.on_remote_dont(telnet),
            telnet::Action::Will => self.on_remote_will(telnet),
            telnet::Action::Wont => self.on_remote_wont(telnet),
        };

        if let Err(e) = result {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }

        Ok(())
    }

    fn on_resize(&mut self, _telnet: &mut Telnet, _size: Size) -> Result<(), TelnetError> {
        Ok(())
    }

    fn on_remote_will(&mut self, _telnet: &mut Telnet) -> Result<(), TelnetError> {
        Ok(())
    }
    fn on_remote_wont(&mut self, _telnet: &mut Telnet) -> Result<(), TelnetError> {
        Ok(())
    }
    fn on_remote_do(&mut self, _telnet: &mut Telnet) -> Result<(), TelnetError> {
        Ok(())
    }
    fn on_remote_dont(&mut self, _telnet: &mut Telnet) -> Result<(), TelnetError> {
        Ok(())
    }

    fn on_subnegotiate(&mut self, _telnet: &mut Telnet, _bytes: &[u8]) -> Result<(), TelnetError> {
        Ok(())
    }
}

pub struct TelnetOptionInteractor {
    pub option: TelnetOption,

    accept_will: bool,
    accept_do: bool,

    acked_will: bool,
    acked_do: bool,
}

impl TelnetOptionInteractor {
    pub fn default(option: TelnetOption) -> Self {
        Self::new(option, false, false)
    }

    pub fn accept_do(option: TelnetOption) -> Self {
        Self::new(option, false, true)
    }

    fn new(option: TelnetOption, accept_will: bool, accept_do: bool) -> Self {
        Self {
            option,
            accept_will,
            accept_do,
            acked_will: false,
            acked_do: false,
        }
    }
}

impl TelnetHandler for TelnetOptionInteractor {
    fn on_remote_will(&mut self, telnet: &mut Telnet) -> Result<(), TelnetError> {
        match (self.accept_will, self.acked_will) {
            (true, false) => {
                crate::info!("## TELNET > Do {:?}", self.option);
                self.acked_will = true;
                telnet.negotiate(&telnet::Action::Do, self.option)
            }
            (false, false) => {
                crate::info!("## TELNET > Dont {:?}", self.option);
                self.acked_will = true;
                telnet.negotiate(&telnet::Action::Dont, self.option)
            }
            _ => Ok(()),
        }
    }

    fn on_remote_do(&mut self, telnet: &mut Telnet) -> Result<(), TelnetError> {
        match (self.accept_do, self.acked_do) {
            (true, false) => {
                crate::info!("## TELNET > Will {:?}", self.option);
                self.acked_do = true;
                telnet.negotiate(&telnet::Action::Will, self.option)
            }
            (false, false) => {
                crate::info!("## TELNET > Wont {:?}", self.option);
                self.acked_do = true;
                telnet.negotiate(&telnet::Action::Wont, self.option)
            }
            _ => Ok(()),
        }
    }
}

pub struct TelnetOptionHandler {
    pub interactor: TelnetOptionInteractor,
    pub handler: Box<dyn TelnetHandler + Send>,
}

impl TelnetHandler for TelnetOptionHandler {
    fn negotiate(&mut self, action: &telnet::Action, telnet: &mut Telnet) -> io::Result<()> {
        self.interactor.negotiate(&action, telnet)?;
        self.handler.negotiate(&action, telnet)
    }

    fn on_subnegotiate(&mut self, telnet: &mut Telnet, bytes: &[u8]) -> Result<(), TelnetError> {
        if self.interactor.acked_will || self.interactor.acked_do {
            self.handler.on_subnegotiate(telnet, bytes)
        } else {
            Ok(())
        }
    }
}
