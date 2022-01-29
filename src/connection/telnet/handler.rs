use telnet::{Telnet, TelnetError};

pub trait TelnetHandler {
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
