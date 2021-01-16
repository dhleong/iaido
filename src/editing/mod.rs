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

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct CursorPosition {
    // FIXME line probably needs to be usize, since this is an absolute
    // line number and not a visual one
    pub line: u16,
    pub col: u16,
}

impl CursorPosition {
    pub fn start_of_line(&self) -> CursorPosition {
        CursorPosition {
            line: self.line,
            col: 0,
        }
    }

    pub fn end_of_line(&self, buffer: &Box<dyn Buffer>) -> CursorPosition {
        let line_width = buffer.get(self.line as usize).width();
        CursorPosition {
            line: self.line,
            col: (line_width - 1) as u16,
        }
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
