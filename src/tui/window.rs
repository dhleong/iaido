use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{self, Span, Spans},
    widgets::Paragraph,
    widgets::Widget,
    widgets::Wrap,
};

use super::{measure::render_into, RenderContext, Renderable};
use crate::editing::{self, text::TextLine, window::Window};
use crate::tui::Measurable;

fn wrap_cursor(line: &TextLine, width: u16, cursor_col: usize) -> (u16, u16) {
    // FIXME: it would be way more efficient to actually do our own wrapping...
    let restyled = line
        .0
        .iter()
        .flat_map(|span| span.styled_graphemes(Style::default()))
        .enumerate()
        .map(|(i, grapheme)| {
            if i < cursor_col {
                Span::raw(grapheme.symbol)
            } else {
                Span::styled(grapheme.symbol, Style::default().fg(Color::Cyan))
            }
        });

    let restyled_line = Spans(restyled.collect());
    let mut buffer = tui::buffer::Buffer::default();
    render_into(&restyled_line, width, &mut buffer);
    let cursor_index = buffer
        .content
        .iter()
        .position(|cell| cell.style().fg == Some(Color::Cyan));

    if let Some(cursor_index) = cursor_index {
        buffer.pos_of(cursor_index)
    } else {
        // probably after the last
        let cursor_index = buffer.content.iter().position(|cell| cell.symbol == "\x00");
        if let Some(cursor_index) = cursor_index {
            buffer.pos_of(cursor_index)
        } else {
            (0, 0)
        }
    }
}

impl Renderable for Window {
    fn render(&self, context: &mut RenderContext) {
        let buf = if let Some(overridden) = context.buffer_override {
            overridden
        } else {
            match context.app.buffers.by_id(self.buffer) {
                None => return,
                Some(buf) => buf,
            }
        };

        let count = buf.lines_count();
        let end = count.checked_sub(self.scrolled_lines as usize).unwrap_or(0);
        let start = end.checked_sub(self.size.h as usize).unwrap_or(0);

        let lines: Vec<text::Spans> = (start..end).map(|i| buf.get(i).clone()).collect();
        let candidate_text = text::Text::from(lines);
        let text_height = candidate_text.measure_height(context.area.width);
        let inner_height = text_height - self.scroll_offset;

        // NOTE: each line scrolled on Paragraph is a line removed
        // from the TOP of the buffer; our scroll goes backward (IE:
        // each scroll_offset removes from the BOTTOM of the buffer)
        // so we invert the scroll_offset to achieve the same effect
        let available_height = context.area.height;
        let scroll = text_height
            .checked_sub(available_height + self.scroll_offset + 1)
            .unwrap_or(0);
        let paragraph = Paragraph::new(candidate_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll((scroll, 0));

        let mut area = context.area.clone();
        if inner_height < area.height {
            area.y = area.bottom() - inner_height;
            area.height = inner_height;
        }

        if self.focused {
            let (x, y) = if count > 0 {
                let (cursor_x, cursor_y_offset) = wrap_cursor(
                    buf.get(self.cursor.line),
                    area.width,
                    self.cursor.col as usize,
                );

                let cursor_y = self.cursor.line.checked_sub(start).unwrap_or(0);

                let x = area.x + cursor_x;
                let y = area.y + (cursor_y as u16) + cursor_y_offset;
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

    use crate::tui::rendering::display::tests::TestableDisplay;
    use crate::{app::State, tui::Display};

    use indoc::indoc;

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
                buffer_override: None,
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
        fn last_line_after_last_col_at_bottom() {
            let text = TextLines::raw("Take my land\nTake me where");
            let display = text.render((15, 10), CursorPosition { line: 1, col: 13 });
            assert_eq!(display.cursor, Cursor::Block(13, 9));
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

    #[cfg(test)]
    mod simple_integration {
        use crate::editing::motion::tests::window;

        use super::*;

        #[test]
        fn cursor_at_bottom() {
            let mut ctx = window(indoc! {"
                Take my love
                Take my land
                Take |me where
            "});

            let display = ctx.render_into_size(14, 3);
            display.assert_visual_match(indoc! {"
                Take my love
                Take my land
                Take |me where
            "});
        }

        #[test]
        fn cursor_with_single_wrap() {
            let mut ctx = window(indoc! {"
                Take me where I cannot |stand
            "});

            let display = ctx.render_into_size(14, 2);
            display.assert_visual_match(indoc! {"
                Take me where
                I cannot |stand
            "});
        }

        #[test]
        fn cursor_with_multi_wrap() {
            let mut ctx = window(indoc! {"
                Take my love, Take my land, Take me where I cannot |stand
            "});

            let display = ctx.render_into_size(14, 4);
            display.assert_visual_match(indoc! {"
                Take my love,
                Take my land,
                Take me where
                I cannot |stand
            "});
        }
    }

    #[cfg(test)]
    mod scrolled_rendering {
        use crate::editing::motion::tests::window;

        use super::*;

        #[test]
        fn one_line_scroll() {
            let mut ctx = window(indoc! {"
                Take my love
                Take |my land
                Take me where
            "});
            ctx.window.resize(Size { w: 13, h: 3 });
            ctx.scroll_lines(1);

            let display = ctx.render_at_own_size();
            display.assert_visual_match(indoc! {"

                Take my love
                Take |my land
            "});
        }

        #[test]
        fn cursor_with_single_scroll_offset() {
            let mut ctx = window(indoc! {"
                Take my land
                Take me wher|e I cannot stand
            "});
            ctx.window.resize(Size { w: 14, h: 2 });
            ctx.scroll_lines(1);

            let display = ctx.render_at_own_size();
            display.assert_visual_match(indoc! {"
                Take my land
                Take me wher|e
            "});
        }

        #[test]
        fn cursor_with_multi_scroll_offset() {
            let mut ctx = window(indoc! {"
                Take my |land Take me where I cannot stand
            "});
            ctx.window.resize(Size { w: 13, h: 2 });
            ctx.scroll_lines(3);

            let display = ctx.render_at_own_size();
            display.assert_visual_match(indoc! {"

                Take my |land
            "});
        }

        #[test]
        fn cursor_on_word_wrapped_character() {
            let mut ctx = window(indoc! {"
                Take me where |I cannot stand
            "});
            ctx.window.resize(Size { w: 9, h: 4 });

            let display = ctx.render_at_own_size();
            display.assert_visual_match(indoc! {"
                Take me
                where |I
                cannot
                stand
            "});
        }

        #[test]
        fn cursor_on_word_wrapped_whitespace() {
            let mut ctx = window(indoc! {"
                Take  my | love
            "});
            ctx.window.resize(Size { w: 8, h: 2 });

            let display = ctx.render_at_own_size();
            display.assert_visual_match(indoc! {"
                Take  my
                |love
            "});
        }
    }
}
