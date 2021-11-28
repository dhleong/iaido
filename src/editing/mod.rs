pub mod buffer;
pub mod buffers;
pub mod change;
pub mod gutter;
pub mod ids;
pub mod layout;
pub mod motion;
pub mod source;
pub mod tabpage;
pub mod tabpages;
pub mod text;
pub mod window;

use std::ops;

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
    pub col: usize,
}

impl CursorPosition {
    pub fn start_of_line(&self) -> CursorPosition {
        CursorPosition {
            line: self.line,
            col: 0,
        }
    }

    pub fn with_col<T: Into<usize>>(&self, col: T) -> CursorPosition {
        CursorPosition {
            line: self.line,
            col: col.into(),
        }
    }

    pub fn end_of_line(&self, buffer: &Box<dyn Buffer>) -> CursorPosition {
        let line_width = buffer.get(self.line).width();
        CursorPosition {
            line: self.line,
            col: line_width,
        }
    }
}

impl ops::Add<(usize, usize)> for CursorPosition {
    type Output = CursorPosition;

    fn add(self, rhs: (usize, usize)) -> CursorPosition {
        let (lines, cols) = rhs;
        Self {
            line: self.line + lines,
            col: self.col + cols,
        }
    }
}

impl Default for CursorPosition {
    fn default() -> Self {
        Self { line: 0, col: 0 }
    }
}

impl From<(usize, usize)> for CursorPosition {
    fn from((line, col): (usize, usize)) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cursor {
    None,
    Block(u16, u16),
    Line(u16, u16),
}

impl ops::Sub<(u16, u16)> for Cursor {
    type Output = Cursor;

    fn sub(self, rhs: (u16, u16)) -> Self::Output {
        let (dx, dy) = rhs;
        match self {
            Cursor::None => Cursor::None,
            Cursor::Block(x, y) => Cursor::Block(
                x.checked_sub(dx).unwrap_or(0),
                y.checked_sub(dy).unwrap_or(0),
            ),
            Cursor::Line(x, y) => Cursor::Line(
                x.checked_sub(dx).unwrap_or(0),
                y.checked_sub(dy).unwrap_or(0),
            ),
        }
    }
}

pub type Id = usize;

pub trait HasId {
    fn id(&self) -> Id;
}

pub trait Resizable {
    fn resize(&mut self, new_size: Size);
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusDirection {
    Up,
    Right,
    Left,
    Down,
}
