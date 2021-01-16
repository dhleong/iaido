use std::io::{self, Write};

use crate::editing;

pub struct CursorRenderer {
    stdout: io::Stdout,
    supports_line: bool,
}

impl CursorRenderer {
    pub fn nop() -> Self {
        Self {
            stdout: io::stdout(),
            supports_line: false,
        }
    }

    /// Call before quitting
    pub fn reset(&mut self) -> Result<(), io::Error> {
        if !self.supports_line {
            // nothing to do:
            return Ok(());
        }

        self.render(editing::Cursor::Block(0, 0))
    }

    pub fn render(&mut self, cursor: editing::Cursor) -> Result<(), io::Error> {
        if !self.supports_line {
            // nothing to do:
            return Ok(());
        }

        let csi = match cursor {
            editing::Cursor::Line(_, _) => b"\x1B[6 q",
            _ => b"\x1B[2 q",
        };

        self.stdout.write_all(csi)?;
        self.stdout.flush()
    }
}

impl Default for CursorRenderer {
    fn default() -> Self {
        Self {
            stdout: io::stdout(),
            supports_line: true, // ?
        }
    }
}
