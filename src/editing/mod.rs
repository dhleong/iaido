pub mod buffer;
pub mod buffers;
pub mod ids;
pub mod layout;
pub mod motion;
pub mod tabpage;
pub mod tabpages;
pub mod text;
pub mod window;

pub use buffer::Buffer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CursorPosition {
    /// Line index within a buffer
    pub line: usize,
    /// Column within a line (NOT the visual column)
    pub col: u16,
}

impl CursorPosition {
    pub fn start_of_line(&self) -> CursorPosition {
        CursorPosition {
            line: self.line,
            col: 0,
        }
    }

    pub fn with_col<T: Into<u16>>(&self, col: T) -> CursorPosition {
        CursorPosition {
            line: self.line,
            col: col.into(),
        }
    }

    pub fn end_of_line(&self, buffer: &Box<dyn Buffer>) -> CursorPosition {
        let line_width = buffer.get(self.line).width();
        CursorPosition {
            line: self.line,
            col: (line_width - 1) as u16,
        }
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self { line: 0, col: 0 }
    }
}

impl From<(usize, u16)> for CursorPosition {
    fn from((line, col): (usize, u16)) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    None,
    Block(u16, u16),
    Line(u16, u16),
}

pub type Id = usize;

pub trait HasId {
    fn id(&self) -> Id;
}

pub trait Resizable {
    fn resize(&mut self, new_size: Size);
}
