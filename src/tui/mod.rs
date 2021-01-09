use crate::editing::Size;

pub mod tabpage;
pub mod tabpages;
pub mod window;
pub use tui::text;

pub struct Display {
    pub size: Size,
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Display({:?})", self.size)
    }
}

pub trait Renderable {
    fn render(&self, display: &mut Display);
}
