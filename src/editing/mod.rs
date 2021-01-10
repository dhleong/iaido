pub mod buffer;
pub mod buffers;
pub mod ids;
pub mod layout;
pub mod tabpage;
pub mod tabpages;
pub mod window;

use std::fmt;

use tui::text;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    pub x: u16,
    pub y: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

pub enum Cursor {
    None,
    // TODO:
    // Block(Position),
    // Line(Position),
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
    fn append(&mut self, text: text::Text<'static>);
    fn get(&self, line_index: usize) -> &text::Spans<'static>;
}

impl fmt::Display for dyn Buffer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[Buffer#{}]", self.id())
    }
}
