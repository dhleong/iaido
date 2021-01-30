use crate::{
    editing::{self, Resizable, Size},
    ui::UI,
};

use crossterm::terminal;
use editing::window::Window;
use std::{cmp::min, io};
pub use tui::text;
use tui::{backend::Backend, Terminal};
use tui::{backend::CrosstermBackend, layout::Rect};

pub mod cursor;
pub mod events;
pub mod layout;
pub mod measure;
pub mod rendering;
pub mod tabpage;
pub mod tabpages;
pub mod window;

use cursor::CursorRenderer;
use measure::Measurable;

pub use rendering::context::RenderContext;
pub use rendering::display::Display;
pub use rendering::size;
pub use rendering::Renderable;

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

        if let Err(e) = terminal::disable_raw_mode() {
            return Err(io::Error::new(io::ErrorKind::Other, e));
        }

        // restore normal cursor
        self.cursor.reset()?;

        // ensure raw mode gets cleanly reset
        backend.flush()
    }

    pub fn render(&mut self, app: &mut crate::app::State) -> Result<(), io::Error> {
        self.terminal.autoresize()?;

        let size = self.terminal.size()?;
        let mut display = Display::new(Size {
            w: size.width,
            h: size.height,
        });

        app.resize(display.size);

        // echo line:
        self.render_echo(app, &mut display);

        // main UI:
        let mut context = RenderContext {
            app: &app,
            display: &mut display,
            area: size,
            buffer_override: None,
        };
        app.tabpages.render(&mut context);

        // prompt
        self.render_prompt(app, &mut display);

        self.render_display(display)
    }

    fn render_echo(&mut self, app: &mut crate::app::State, display: &mut Display) {
        // NOTE: doesn't allow for word wrapping:
        let echo_height = app.echo_buffer.lines_count() as u16;
        if echo_height == 0 {
            return;
        }

        let area = Rect {
            x: 0,
            y: display.size.h - echo_height,
            width: display.size.w,
            height: echo_height,
        };
        let mut context = RenderContext {
            app: &app,
            display,
            area,
            buffer_override: Some(&app.echo_buffer),
        };
        let mut win = Window::new(0, app.echo_buffer.id());
        win.set_focused(false);
        win.resize(Size {
            w: area.width,
            h: echo_height,
        });
        win.render(&mut context);
    }

    fn render_prompt(&mut self, app: &mut crate::app::State, display: &mut Display) {
        let mut prompt_display = Display::new(display.size);
        app.prompt.window.render(
            &mut RenderContext::new(app, &mut prompt_display).with_buffer(&app.prompt.buffer),
        );
        let prompt_height = min(
            display.size.h,
            app.prompt.buffer.measure_height(display.size.w),
        );
        if prompt_height == 0 {
            // nop
            return;
        }

        display.shift_up(prompt_height.checked_sub(1).unwrap_or(0));
        display.merge_at_y(display.size.h - prompt_height, prompt_display);
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

impl Drop for Tui {
    fn drop(&mut self) {
        if let Err(e) = self.close() {
            println!("Error closing Tui: {}", e);
        }
    }
}

impl UI for Tui {
    fn measure_text_height(&self, line: editing::text::TextLine, width: u16) -> u16 {
        line.measure_height(width)
    }

    fn render_app(&mut self, app: &mut crate::app::State) {
        if let Err(e) = self.render(app) {
            // ?
            panic!("Error rendering app: {}", e);
        }
    }
}

pub fn create_ui() -> Result<Tui, io::Error> {
    let backend = CrosstermBackend::new(io::stdout());
    let mut terminal = Terminal::new(backend)?;

    terminal.clear()?;

    if let Err(e) = terminal::enable_raw_mode() {
        return Err(io::Error::new(io::ErrorKind::Other, e));
    }

    Ok(Tui {
        cursor: CursorRenderer::default(),
        terminal,
    })
}
