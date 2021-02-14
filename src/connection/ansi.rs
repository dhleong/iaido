use vte::{Parser, Perform};

pub struct AnsiPipeline {
    parser: Parser,
    performer: AnsiPerformer,
}

impl Default for AnsiPipeline {
    fn default() -> Self {
        Self {
            parser: Parser::new(),
            performer: AnsiPerformer {},
        }
    }
}

impl AnsiPipeline {
    pub fn feed(&mut self, buf: &[u8], n: usize) {
        for byte in &buf[..n] {
            self.parser.advance(&mut self.performer, *byte);
        }
    }
}

struct AnsiPerformer {}

impl Perform for AnsiPerformer {
    fn print(&mut self, _c: char) {}

    fn execute(&mut self, _byte: u8) {}

    fn hook(&mut self, _params: &vte::Params, _intermediates: &[u8], _ignore: bool, _action: char) {
    }

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        _params: &vte::Params,
        _intermediates: &[u8],
        _ignore: bool,
        _action: char,
    ) {
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}
