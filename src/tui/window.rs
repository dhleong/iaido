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
        let end = count.checked_sub(self.scrolled_lines as usize).unwrap_or(0);
        let start = end.checked_sub(self.size.h as usize).unwrap_or(0);

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
            let (x, y) = if count > 0 {
                // FIXME this y_offset doesn't account for word-wrapping
                let cursor_x = self.cursor.col % area.width;
                let cursor_y_offset = (self.cursor.col / area.width).checked_sub(1).unwrap_or(0);

                let cursor_y_absolute = (self.cursor.line as usize).checked_sub(start).unwrap_or(0);
                let cursor_y = cursor_y_absolute
                    .checked_sub(self.scroll_offset as usize)
                    .unwrap_or(0);

                let x = area.x + cursor_x;
                let y = area.y + (cursor_y as u16) - cursor_y_offset;
                (x, y)
            } else {
                // simple case
                (area.x, area.y)
            };

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
    use editing::{text::TextLine, text::TextLines, Cursor, CursorPosition, Resizable, Size};
    use tui::layout::Rect;

    use crate::{app::State, tui::Display};

    use super::*;

    trait Testable {
        fn render<T>(&self, size: T, cursor: CursorPosition) -> Display
        where
            T: Into<Size>;
    }

    impl Testable for TextLines {
        fn render<T>(&self, size: T, cursor: CursorPosition) -> Display
        where
            T: Into<Size>,
        {
            let mut state = State::default();

            let buffer_id: usize = {
                let buffer = state.buffers.create();
                buffer.id()
            };

            {
                state
                    .buffers
                    .by_id_mut(buffer_id)
                    .unwrap()
                    .append(self.clone());
            }

            let size_struct = size.into();
            let area: Rect = size_struct.into();
            let mut display = Display::new(area.into());
            let mut context = RenderContext {
                app: &state,
                display: &mut display,
                area,
            };

            {
                let mut window = Window::new(0, buffer_id);
                window.resize(area.into());
                window.cursor = cursor;
                window.render(&mut context);
            }

            return display;
        }
    }

    impl Testable for TextLine {
        fn render<T>(&self, size: T, cursor: CursorPosition) -> Display
        where
            T: Into<Size>,
        {
            let lines = TextLines::from(self.clone());
            lines.render(size, cursor)
        }
    }

    mod cursor_rendering {
        use super::*;

        #[test]
        fn single_line_at_bottom() {
            let text = TextLine::from("Take my love");
            let display = text.render((12, 10), CursorPosition { line: 0, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 9));
        }

        #[test]
        fn wrapped_line_at_bottom() {
            let text = TextLine::from("Take my love");
            let display = text.render((4, 10), CursorPosition { line: 0, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 7));
        }

        #[test]
        fn first_line_of_multi_at_bottom() {
            let text = TextLines::raw("Take my land\nTake me where");
            let display = text.render((15, 10), CursorPosition { line: 0, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 8));
        }

        #[test]
        fn last_line_at_bottom() {
            let text = TextLines::raw("Take my land\nTake me where");
            let display = text.render((15, 10), CursorPosition { line: 1, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 9));
        }

        #[test]
        fn last_line_last_col_at_bottom() {
            let text = TextLines::raw("Take my land\nTake me where");
            let display = text.render((15, 10), CursorPosition { line: 1, col: 14 });
            assert_eq!(display.cursor, Cursor::Block(14, 9));
        }

        #[test]
        fn middle_line_at_bottom() {
            let text = TextLines::raw("Take my love\nTake my land\nTake me where");
            let display = text.render((15, 10), CursorPosition { line: 1, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 8));
        }

        #[test]
        fn middle_line_of_split_at_bottom() {
            let text = TextLines::raw("Take my love\nTake my land\nTake me where");
            let display = text.render(Rect::new(0, 5, 15, 10), CursorPosition { line: 1, col: 0 });
            assert_eq!(display.cursor, Cursor::Block(0, 8));
        }
    }
}
