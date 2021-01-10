use crate::editing::{Resizable, Size};

use std::io;
pub use tui::text;
use tui::Terminal;
use tui::{backend::CrosstermBackend, layout::Rect};

pub mod layout;
pub mod tabpage;
pub mod tabpages;
pub mod window;

pub struct Display {
    pub size: Size,
    pub buffer: tui::buffer::Buffer,
}

impl Display {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            buffer: tui::buffer::Buffer::empty(Rect::new(0, 0, size.w, size.h)),
        }
    }
}

impl tui::widgets::Widget for Display {
    fn render(self, _area: Rect, buf: &mut tui::buffer::Buffer) {
        buf.merge(&self.buffer);
    }
}

impl std::fmt::Display for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Display({:?})", self.size)?;

        // TODO copy content
        // for line in &self.lines {
        //     write!(f, "\n  {:?}", line)?;
        // }

        write!(f, "]")
    }
}

pub trait Renderable {
    fn render(&self, app: &crate::App, display: &mut Display, area: Rect);
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
            f.render_widget(display, f.size());
            // buf.merge(&display.buffer);
            // let p = tui::widgets::Paragraph::new(display.lines);
            // let rect = f.size();
            // f.render_widget(p, rect);
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
