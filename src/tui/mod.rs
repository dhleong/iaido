use crate::{
    app::{popup::PopupMenu, widgets::Widget},
    editing::{self, Resizable, Size},
    ui::UI,
};

use crossterm::terminal;
use editing::window::Window;
use std::{cmp::min, convert::TryInto, io};
use tui::{
    backend::Backend,
    style::{Color, Style},
    text::Span,
    widgets::{Block, ListState},
    Terminal,
};
use tui::{backend::CrosstermBackend, layout::Rect};
pub use tui::{
    text,
    widgets::{List, ListItem},
};

pub mod cursor;
pub mod events;
pub mod layout;
pub mod measure;
pub mod rendering;
mod splash;
pub mod tabpage;
pub mod tabpages;
pub mod window;

use cursor::CursorRenderer;
use measure::Measurable;

pub use rendering::context::LayoutContext;
pub use rendering::context::RenderContext;
pub use rendering::display::Display;
pub use rendering::size;
pub use rendering::Renderable;

pub struct Tui {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    cursor: CursorRenderer,
}

impl Tui {
    pub fn size(&self) -> io::Result<Size> {
        let size = self.terminal.size()?;
        Ok(Size {
            w: size.width,
            h: size.height,
        })
    }

    pub fn close(&mut self) -> io::Result<()> {
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

        // main UI:
        app.tabpages.layout(&LayoutContext::new(&app.buffers));
        app.tabpages
            .render(&mut RenderContext::new(&app, &mut display).with_area(size));

        // echo line(s):
        self.render_echo(app, &mut display);

        // prompt
        self.render_prompt(app, &mut display);

        // popup menu
        if let Some(pum) = app.pum.as_ref() {
            match display.cursor.clone() {
                editing::Cursor::Line(x, y) => Tui::render_pum(pum, x, y, &mut display),
                editing::Cursor::Block(x, y) => Tui::render_pum(pum, x, y, &mut display),
                _ => {} // nop
            }
        }

        // render any active keymap widget
        if let Some(w) = &app.keymap_widget {
            self.render_widget(
                w,
                Rect {
                    x: 0,
                    y: display.buffer.area.height - 1,
                    width: display.buffer.area.width,
                    height: 1,
                },
                &mut display,
            );
        }

        if app.showing_splash {
            splash::render(&mut display);
        }

        self.render_display(display)
    }

    fn render_echo(&mut self, app: &mut crate::app::State, display: &mut Display) {
        // NOTE: doesn't allow for word wrapping:
        let echo_height = app.echo_buffer.lines_count() as u16;
        if echo_height == 0 {
            return;
        }

        // make room
        display.shift_up(echo_height - 1);

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

    fn render_widget(&mut self, widget: &Widget, area: Rect, display: &mut Display) {
        match widget {
            &Widget::Space => {}
            &Widget::Spread(ref children) => {
                if !children.is_empty() {
                    let each_width = area.width / (children.len() as u16);
                    let mut child_area = Rect {
                        x: 0,
                        width: each_width,
                        ..area
                    };
                    for child in children {
                        self.render_widget(child, child_area, display);
                        child_area.x += each_width;
                    }
                }
            }

            &Widget::Literal(ref text) => {
                display.buffer.set_spans(area.x, area.y, text, area.width);
            }
        }
    }

    fn render_prompt(&mut self, app: &mut crate::app::State, display: &mut Display) {
        let prompt_height = min(
            display.size.h,
            app.prompt.buffer.measure_height(display.size.w),
        );
        if prompt_height == 0 {
            // nop
            return;
        }

        app.prompt
            .window
            .layout(&LayoutContext::with_buffer(&app.prompt.buffer));

        let mut prompt_display = Display::new(display.size);
        app.prompt.window.render(
            &mut RenderContext::new(app, &mut prompt_display).with_buffer(&app.prompt.buffer),
        );

        if app.prompt.window.focused {
            display.cursor = prompt_display.cursor.clone();
        }

        display.shift_up(prompt_height - 1);
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

    fn render_pum(pum: &PopupMenu, x: u16, y: u16, display: &mut Display) {
        let list = List::new(
            pum.contents
                .iter()
                .map(|item| ListItem::new(Span::raw(item)))
                .collect::<Vec<ListItem>>(),
        )
        .highlight_style(Style::default().bg(Color::LightBlue))
        .block(Block::default().style(Style::default().bg(Color::Blue)));

        let mut list_state = ListState::default();
        list_state.select(pum.cursor);

        let Size { w, h } = pum.measure(display.size);

        let requested_x = x
            .checked_sub(pum.horizontal_offset.try_into().unwrap_or(0))
            .unwrap_or(0);
        let x = if requested_x + w > display.size.w {
            display.size.w - w
        } else {
            requested_x
        };

        let y = if y + 1 + h > display.size.h / 2 {
            y.checked_sub(h).unwrap_or(1u16)
        } else {
            y + 1
        };

        let area = Rect::new(x, y, w, h);
        display.clear(area);

        tui::widgets::StatefulWidget::render(list, area, &mut display.buffer, &mut list_state);
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
