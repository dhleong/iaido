pub mod buffer;
pub mod buffers;
pub mod ids;
pub mod layout;
pub mod tabpage;
pub mod tabpages;
pub mod text;
pub mod window;

use std::fmt;

use text::{TextLine, TextLines};

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub line: u16,
    pub col: u16,
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

pub trait Buffer: HasId {
    fn lines_count(&self) -> usize;
    fn append(&mut self, text: TextLines);
    fn get(&self, line_index: usize) -> &TextLine;
}

impl fmt::Display for dyn Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffer#{}]", self.id())
    }
}
