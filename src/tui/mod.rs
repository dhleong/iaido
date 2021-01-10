use crate::editing::{Resizable, Size};

use std::io;
pub use tui::text;
use tui::Terminal;
use tui::{backend::CrosstermBackend, layout::Rect};

pub mod layout;
pub mod tabpage;
pub mod tabpages;
pub mod window;

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
    fn render<'a>(&self, app: &'a crate::App, display: &mut Display<'a>, area: Rect);
}

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl Tui {
    pub fn render(&mut self, app: &mut crate::App) -> Result<(), io::Error> {
        let size = self.terminal.size()?;
        let mut display = Display::new(Size {
            w: size.width,
            h: size.height,
        });

        app.resize(display.size);
        app.tabpages.render(&app, &mut display, size);

        self.render_display(display)
    }

    fn render_display(&mut self, display: Display) -> Result<(), io::Error> {
        self.terminal.draw(|f| {
            let p = tui::widgets::Paragraph::new(display.lines);
            let rect = f.size();
            f.render_widget(p, rect);
        })
    }
}

pub fn create_ui() -> Result<Tui, io::Error> {
    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    Ok(Tui { terminal })
}
