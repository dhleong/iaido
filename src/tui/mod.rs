use crate::editing::Size;

pub mod tabpage;
pub mod tabpages;
pub mod window;
pub use tui::text;

pub struct Display<'a> {
    pub size: Size,
    pub lines: Vec<text::Spans<'a>>,
}

impl<'a> Display<'a> {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            lines: Vec::new(),
        }
    }
}

impl<'a> std::fmt::Display for Display<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Display({:?})", self.size)?;

        for line in &self.lines {
            write!(f, "\n  {:?}", line)?;
        }

        write!(f, "]")
    }
}

pub trait Renderable {
    fn render<'a>(&self, app: &'a crate::App, display: &mut Display<'a>);
}
