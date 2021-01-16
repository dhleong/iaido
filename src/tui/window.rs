use std::cmp;

use tui::{layout::Alignment, text, widgets::Paragraph, widgets::Widget, widgets::Wrap};

use super::{RenderContext, Renderable};
use crate::editing::{self, window::Window};
use crate::tui::Measurable;

impl Renderable for Window {
    fn render(&self, context: &mut RenderContext) {
        let buf = match context.app.buffers.by_id(self.buffer) {
            None => return,
            Some(buf) => buf,
        };

        let count = buf.lines_count();
        let end = count - cmp::min(count, self.scrolled_lines as usize);
        let start = end - cmp::min(end, self.size.h as usize);

        let lines: Vec<text::Spans> = (start..end).map(|i| buf.get(i).clone()).collect();
        let candidate_text = text::Text::from(lines);
        let inner_height = candidate_text.measure_height(context.area.width) - self.scroll_offset;

        let paragraph = Paragraph::new(candidate_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll((0, self.scroll_offset));

        let mut area = context.area.clone();
        if inner_height < area.height {
            area.y = area.bottom() - inner_height;
            area.height = inner_height;
        }

        if self.focused {
            // FIXME this y_offset doesn't account for word-wrapping
            let cursor_x = self.cursor.col % area.width;
            let cursor_y_offset = self.cursor.col / area.width;

            let cursor_y_absolute = (self.cursor.line as usize) - start;
            let cursor_y = cursor_y_absolute
                .checked_sub(self.scroll_offset as usize)
                .unwrap_or(0);

            let x = area.x + cursor_x;
            let y = area.y + (cursor_y as u16) + cursor_y_offset;

            if self.inserting {
                context.display.set_cursor(editing::Cursor::Line(x, y));
            } else {
                context.display.set_cursor(editing::Cursor::Block(x, y));
            }
        }

        paragraph.render(area, &mut context.display.buffer);
    }
}

#[cfg(test)]
mod tests {
    use editing::{text::TextLine, text::TextLines, Cursor, CursorPosition, Size};

    use crate::{app::State, tui::Display};

    use super::*;

    trait Testable {
        fn render(&self, size: (u16, u16), cursor: CursorPosition) -> Display;
    }

    impl Testable for TextLines {
        fn render(&self, size: (u16, u16), cursor: CursorPosition) -> Display {
            let mut state = State::default();

            let buffer_id: usize = {
                let buffer = state.buffers.create();
                buffer.id()
            };

            {
                state.current_buffer_mut().append(self.clone());
            }

            let size = Size {
                w: size.0,
                h: size.1,
            };
            let mut display = Display::new(size);
            let mut context = RenderContext {
                app: &state,
                display: &mut display,
                area: size.into(),
            };

            {
                let mut window = Window::new(0, buffer_id);
                window.cursor = cursor;
                window.render(&mut context);
            }

            return display;
        }
    }

    impl Testable for TextLine {
        fn render(&self, size: (u16, u16), cursor: CursorPosition) -> Display {
            let lines = TextLines::from(self.clone());
            lines.render(size, cursor)
        }
    }

    mod cursor_rendering {
        use super::*;

        #[test]
        fn single_line_at_bottom() {
            let text = TextLine::from("Take my love");
            let display = text.render((10, 10), CursorPosition { line: 0, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 10));
        }
    }
}
