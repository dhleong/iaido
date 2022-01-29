use clap::crate_version;
use telnet::{self, TelnetError, TelnetOption};

use super::handler::TelnetHandler;

const MTTS_ANSI: u16 = 1;
const MTTS_UTF8: u16 = 4;
const MTTS_256COLOR: u16 = 8;
const MTTS_TRUE_COLOR: u16 = 256;

const TELNET_IS: u8 = 0;

#[derive(Clone, Copy)]
enum TTypeRequestState {
    ClientName,
    TermType,
    MttsBitVector,
}

pub struct TTypeHandler {
    state: TTypeRequestState,
}

impl Default for TTypeHandler {
    fn default() -> Self {
        Self {
            state: TTypeRequestState::ClientName,
        }
    }
}

impl TTypeHandler {
    fn build_mtts_bitvector(&self) -> u16 {
        // TODO Verify UTF8 support?
        MTTS_ANSI + MTTS_UTF8 + MTTS_256COLOR + MTTS_TRUE_COLOR
    }

    fn send_state(
        &self,
        telnet: &mut telnet::Telnet,
        state: TTypeRequestState,
    ) -> Result<(), TelnetError> {
        let name = match state {
            TTypeRequestState::ClientName => format!("IAIDO {}", crate_version!()),
            TTypeRequestState::TermType => "ANSI-TRUECOLOR".to_string(), // ?
            TTypeRequestState::MttsBitVector => format!("MTTS {}", self.build_mtts_bitvector()),
        };
        let name_bytes = name.as_bytes();

        crate::info!("## TELNET > IAC SB TTYPE IS {}", &name);

        let message = [&[TELNET_IS], name_bytes].concat();
        telnet.subnegotiate(TelnetOption::TTYPE, &message)
    }
}

impl TelnetHandler for TTypeHandler {
    fn on_remote_do(&mut self, telnet: &mut telnet::Telnet) -> Result<(), TelnetError> {
        crate::info!("## TELNET > Will TTYPE");
        telnet.negotiate(&telnet::Action::Will, TelnetOption::TTYPE)
    }

    fn on_remote_dont(&mut self, _telnet: &mut telnet::Telnet) -> Result<(), TelnetError> {
        self.state = TTypeRequestState::ClientName;
        Ok(())
    }

    fn on_subnegotiate(
        &mut self,
        telnet: &mut telnet::Telnet,
        _bytes: &[u8],
    ) -> Result<(), TelnetError> {
        let state = self.state;
        self.state = match state {
            TTypeRequestState::ClientName => TTypeRequestState::TermType,
            _ => TTypeRequestState::MttsBitVector,
        };

        self.send_state(telnet, state)
    }
}
