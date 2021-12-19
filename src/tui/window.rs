use std::cmp::min;

use tui::{
    layout::Alignment,
    style::{Color, Style},
    text::{self, Span, Spans},
    widgets::Paragraph,
    widgets::Widget,
    widgets::Wrap,
};

use super::{measure::render_into, LayoutContext, RenderContext, Renderable};
use crate::editing::{self, text::TextLine, window::Window, Buffer};
use crate::tui::Measurable;

#[derive(Debug, PartialEq)]
struct WrappedLineOffset {
    line: usize,

    /// number of virtual lines skipped
    visual_offset: u16,
}

struct RenderableContent<'a> {
    start: WrappedLineOffset,
    end: WrappedLineOffset,
    candidate_text: text::Text<'a>,
    gutter_width: u16,
    inner_height: u16,
    inner_width: u16,

    scroll_offset: u16,
    scrolled_lines: usize,

    /// Contains the number of virtual lines rendered per visible line.
    /// line_heights[0] is start.line
    line_heights: Vec<u16>,
}

impl<'a> RenderableContent<'a> {
    fn new(window: &Window, buf: &Box<dyn Buffer>) -> Self {
        let count = buf.lines_count();

        let gutter_width = if let Some(gutter) = window.gutter.as_ref() {
            gutter.width.into()
        } else {
            0
        };
        let available_width = window.size.w.checked_sub(gutter_width).unwrap_or(0);

        // ensure scrolled_lines isn't excessive (we should always
        // be able to render at least one line)
        let scrolled_lines = min(
            window.scrolled_lines as usize,
            count.checked_sub(1).unwrap_or(0),
        );

        let end = count.checked_sub(scrolled_lines).unwrap_or(0);
        let start = end.checked_sub(window.size.h as usize).unwrap_or(0);

        let lines: Vec<text::Spans> = (start..end).map(|i| buf.get(i).clone()).collect();
        let line_heights: Vec<u16> = lines
            .iter()
            .map(|line| line.measure_height(available_width))
            .collect();

        // make sure we aren't scrolled too far due to an "undo," etc
        let bottom_height = *line_heights.last().unwrap_or(&0);
        let scroll_offset = min(
            bottom_height.checked_sub(1).unwrap_or(0),
            window.scroll_offset,
        );

        let candidate_text = text::Text::from(lines);
        let text_height: u16 = line_heights.iter().sum();
        let inner_height = text_height - scroll_offset;

        // NOTE: each line scrolled on Paragraph is a line removed
        // from the TOP of the buffer; our scroll goes backward (IE:
        // each scroll_offset removes from the BOTTOM of the buffer)
        // so we invert the scroll_offset to achieve the same effect
        let available_height = window.size.h;
        let scroll = text_height
            .checked_sub(available_height + scroll_offset)
            .unwrap_or(0);

        // NOTE: this offset should be the last *rendered* offset;
        // bottom_height - scroll_offset is how many lines are rendered
        let end_visual_offset = bottom_height.checked_sub(scroll_offset + 1).unwrap_or(0);

        Self {
            start: WrappedLineOffset {
                line: start,
                visual_offset: scroll,
            },
            end: WrappedLineOffset {
                line: end.checked_sub(1).unwrap_or(0),
                visual_offset: end_visual_offset,
            },
            candidate_text,
            gutter_width,
            inner_height,
            inner_width: available_width,
            scroll_offset,
            scrolled_lines,
            line_heights,
        }
    }
}

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
    fn layout(&mut self, context: &LayoutContext) {
        if !self.focused {
            // don't scroll to keep cursor in view unless focused
            return;
        }

        let buf = if let Some(buf) = context.buffer(self.buffer) {
            buf
        } else {
            return;
        };

        let renderable = RenderableContent::new(self, buf);
        self.scroll_offset = renderable.scroll_offset;
        self.scrolled_lines = renderable.scrolled_lines as u32;

        // Sanity check: always clamp cursor
        self.cursor = self.clamp_cursor(buf, self.cursor);

        let cursor_line = match buf.checked_get(self.cursor.line) {
            Some(line) => line,
            None => return, // nothing we can do
        };
        let (_, cursor_y_offset) =
            wrap_cursor(cursor_line, renderable.inner_width, self.cursor.col);

        if self.cursor.line < renderable.start.line {
            self.scroll_offset = 0;
            self.scrolled_lines += (renderable.start.line - self.cursor.line) as u32;
        } else if self.cursor.line > renderable.end.line {
            self.scroll_offset = 0;
            self.scrolled_lines = self
                .scrolled_lines
                .checked_sub((self.cursor.line - renderable.end.line) as u32)
                .unwrap_or(0);
        } else {
            if self.cursor.line == renderable.start.line
                && cursor_y_offset < renderable.start.visual_offset
            {
                self.scroll_lines(
                    buf,
                    (renderable.start.visual_offset - cursor_y_offset) as i32,
                );
            } else if self.cursor.line == renderable.end.line
                && cursor_y_offset > renderable.end.visual_offset
            {
                self.scroll_lines(
                    buf,
                    (renderable.end.visual_offset as i32) - cursor_y_offset as i32,
                );
            }
        }
    }

    fn render(&self, context: &mut RenderContext) {
        let buf = if let Some(overridden) = context.buffer_override {
            overridden
        } else {
            match context.app.buffers.by_id(self.buffer) {
                None => return,
                Some(buf) => buf,
            }
        };

        let renderable = RenderableContent::new(self, buf);
        let paragraph = Paragraph::new(renderable.candidate_text)
            .wrap(Wrap { trim: false })
            .alignment(Alignment::Left)
            .scroll((renderable.start.visual_offset, 0));

        let mut area = context.area.clone();
        if renderable.inner_height < area.height {
            area.y = area.bottom() - renderable.inner_height;
            area.height = renderable.inner_height;
        }

        // Windows should never appear to have 0 height
        if area.height == 0 {
            area.height = 1;
            area.y -= 1;
        }

        let gutter_x = area.x;
        area.x += renderable.gutter_width;
        area.width -= renderable.gutter_width;

        if self.focused {
            let (x, y) = if buf.lines_count() > 0 {
                let (cursor_x, cursor_y_offset) =
                    wrap_cursor(buf.get(self.cursor.line), area.width, self.cursor.col);

                let cursor_virtual_lines = self
                    .cursor
                    .line
                    .checked_sub(renderable.start.line)
                    .unwrap_or(0);
                let cursor_y: u16 = renderable
                    .line_heights
                    .iter()
                    .take(cursor_virtual_lines)
                    .sum();

                let x = area.x + cursor_x;
                let y = (area.y + cursor_y + cursor_y_offset)
                    .checked_sub(renderable.start.visual_offset)
                    .unwrap_or(0);

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

        if let Some(gutter) = self.gutter.as_ref() {
            let width: u16 = gutter.width.into();
            for y in context.area.y..context.area.y + context.area.height {
                let line = if y >= area.y {
                    let relative: usize = (y - area.y).into();
                    Some(renderable.start.line + relative)
                } else {
                    None
                };
                let content = (gutter.get_content)(line);
                context
                    .display
                    .buffer
                    .set_spans(gutter_x, y, &content, width);
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
    use editing::motion::tests::window;

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

        #[test]
        fn cursor_after_wrapped() {
            let mut ctx = window(indoc! {"
                Take my love, Take my land, Take me where
                I cannot |stand
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

        // NOTE: After disabling trimming, this test is not correct,
        // because the intra-line whitespace is *also* preserved when
        // wrapping, it seems. I *think* the new behavior is fine, but
        // keeping this test around just in case
        #[ignore]
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

        #[test]
        fn cursor_above_window_area() {
            let mut ctx = window(indoc! {"
                |Take my love Take my land Take me where
            "});
            ctx.window.resize(Size { w: 13, h: 1 });

            let display = ctx.render_at_own_size();
            display.assert_visual_match(indoc! {"
                |Take my love
            "});
        }

        #[test]
        fn cursor_after_whitespace() {
            let mut ctx = window(":|");
            ctx.window.inserting = true;
            ctx.window.resize(Size { w: 12, h: 1 });
            let before = ctx.render_at_own_size();
            assert_eq!(before.cursor_coords(), Some((1, 0)));

            ctx.buffer.insert(ctx.window.cursor, " ".into());
            ctx.window.cursor.col += 1;

            let display = ctx.render_at_own_size();
            assert_eq!(display.cursor_coords(), Some((2, 0)));
            display.assert_visual_match(indoc! {"
                : |
            "});
        }
    }

    #[cfg(test)]
    mod scroll_cursor_adjustment {
        use super::*;

        #[test]
        fn move_cursor_upward_with_scroll() {
            let mut ctx = window(indoc! {"
                Take my love
                Take my land
                Take me where
                I cannot
                |stand
            "});
            ctx.window.resize(Size { w: 13, h: 2 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                I cannot
                |stand
            "});

            ctx.scroll_lines(3);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my love
                |Take my land
            "});
        }

        #[test]
        fn move_cursor_past_end_of_wrapped_document() {
            let mut ctx = window(indoc! {"
                Take my land Take me where
                I cannot |stand
            "});
            ctx.window.resize(Size { w: 13, h: 2 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                I cannot
                |stand
            "});

            // Should be no change:
            ctx.feed_vim("j")
                .render_at_own_size()
                .assert_visual_match(indoc! {"
                I cannot
                |stand
            "});
        }

        #[test]
        fn move_cursor_upward_with_scroll_offsets() {
            let mut ctx = window(indoc! {"
                Take my land Take me where I cannot |stand
            "});
            ctx.window.resize(Size { w: 13, h: 2 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                I cannot
                |stand
            "});

            // TODO: hugging the column might be nice, but is tricky
            // when wrapping
            ctx.scroll_lines(3);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                Take my land
                Take me wh|ere
            "});
        }

        #[test]
        fn move_cursor_downward_with_scroll() {
            let mut ctx = window(indoc! {"
                |Take my love
                Take my land
                Take me where
                I cannot
                stand
            "});
            ctx.window.resize(Size { w: 13, h: 2 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |Take my love
                Take my land
            "});

            ctx.scroll_lines(-3);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |I cannot
                stand
            "});
        }

        #[test]
        fn move_cursor_downward_with_scroll_offsets() {
            let mut ctx = window(indoc! {"
                |Take my love Take my land Take me where I cannot stand
            "});
            ctx.window.resize(Size { w: 13, h: 2 });
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |Take my love
                Take my land
            "});

            ctx.scroll_lines(-3);
            ctx.render_at_own_size().assert_visual_match(indoc! {"
                |I cannot
                stand
            "});
        }
    }

    #[cfg(test)]
    mod append_value {
        use crate::connection::ReadValue;

        use super::*;

        #[test]
        fn append_value() {
            let mut ctx = window("");
            ctx.window.resize(Size { w: 12, h: 3 });
            ctx.render_at_own_size();

            ctx.buffer
                .append_value(ReadValue::Text("Take my love".into()));

            ctx.render_at_own_size().assert_visual_match(indoc! {"


                |Take my love
            "});

            ctx.buffer.append_value(ReadValue::Newline);
            ctx.buffer
                .append_value(ReadValue::Text("Take my land".into()));

            ctx.render_at_own_size().assert_visual_match(indoc! {"

                |Take my love
                Take my land
            "});
        }
    }

    #[cfg(test)]
    mod cursor_clamping {
        use super::*;

        #[test]
        fn clamp_empty_window() {
            let mut ctx = window("Take my love|");
            ctx.buffer.clear();
            ctx.render_at_own_size();
            assert_eq!(ctx.window.cursor, (0, 0).into());
            ctx.assert_visual_match(indoc! {"
                |
            "});
        }
    }

    #[cfg(test)]
    mod scroll_clamping {
        use super::*;

        #[test]
        fn excess_offset() {
            let mut ctx = window("Take my love");
            ctx.window.scroll_offset = 2;
            ctx.assert_visual_match(indoc! {"
                |Take my love
            "});
        }

        #[test]
        fn excess_lines() {
            let mut ctx = window("Take my love");
            ctx.window.scrolled_lines = 1;
            ctx.assert_visual_match(indoc! {"
                |Take my love
            "});
        }
    }
}
