use crate::editing::{self, Resizable, Size};

use std::io;
pub use tui::text;
use tui::{backend::Backend, Terminal};
use tui::{backend::CrosstermBackend, layout::Rect};

pub mod cursor;
pub mod layout;
pub mod tabpage;
pub mod tabpages;
pub mod window;

use cursor::CursorRenderer;

pub struct Display {
    pub size: Size,
    pub buffer: tui::buffer::Buffer,
    pub cursor: editing::Cursor,
}

impl Display {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            buffer: tui::buffer::Buffer::empty(Rect::new(0, 0, size.w, size.h)),
            cursor: editing::Cursor::None,
        }
    }

    pub fn set_cursor(&mut self, cursor: editing::Cursor) {
        self.cursor = cursor;
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
    cursor: CursorRenderer,
}

impl Tui {
    pub fn close(&mut self) -> Result<(), io::Error> {
        // move the cursor to the bottom of the screen before leaving
        // so the terminal perserves output (hopefully)
        let size = self.terminal.size()?;
        let backend = &mut self.terminal.backend_mut();
        backend.set_cursor(0, size.height)?;

        // restore normal cursor
        self.cursor.reset()
    }

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
        let cursor = display.cursor.clone();
        self.terminal.draw(|f| {
            f.render_widget(display, f.size());

            match cursor {
                editing::Cursor::None => { /* nop */ }
                editing::Cursor::Block(x, y) => {
                    f.set_cursor(x, y);
                }
                editing::Cursor::Line(x, y) => {
                    f.set_cursor(x, y);
                }
            }
        })?;

        self.cursor.render(cursor)
    }
}

pub fn create_ui() -> Result<Tui, io::Error> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    Ok(Tui {
        cursor: CursorRenderer::new(),
        terminal,
    })
}
