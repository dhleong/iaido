use std::rc::Rc;

pub mod buffer;
pub mod buffers;
pub mod ids;
pub mod tabpage;
pub mod tabpages;

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

pub trait Buffer: HasId {
    fn lines_count(&self) -> usize;
}

pub trait Window: HasId {
    fn cursor(&self) -> Cursor;
    fn current_buffer(&self) -> Rc<dyn Buffer>;
    fn size(&self) -> Size;
}

pub trait WindowFactory<T: Window> {
    fn create(&self, id: Id, buffer: Rc<dyn Buffer>) -> T;
}
