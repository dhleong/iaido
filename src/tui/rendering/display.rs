use tui::layout::Rect;

use crate::editing::{self, Size};

pub struct Display {
    pub size: Size,
    pub buffer: tui::buffer::Buffer,
    pub cursor: editing::Cursor,
}

impl Display {
    pub fn new(size: Size) -> Self {
        Self {
            size,
            buffer: tui::buffer::Buffer::empty(size.into()),
            cursor: editing::Cursor::None,
        }
    }

    pub fn merge_at_y(&mut self, y: u16, other: Display) {
        let to_merge_height = self.size.h - y;
        let cells_start = (y * self.size.w) as usize;
        let cells_count = (to_merge_height * self.size.w) as usize;
        let mut cells = other
            .buffer
            .content()
            .iter()
            .skip(cells_start)
            .take(cells_count);

        let start = self.buffer.index_of(0, y);
        for i in start..self.buffer.content.len() {
            if let Some(cell) = cells.next() {
                self.buffer.content[i] = cell.to_owned();
            } else {
                // no more cells to merge
                break;
            }
        }
    }

    pub fn shift_up(&mut self, lines: u16) {
        if lines == 0 {
            return; // nop
        }

        self.buffer.content.drain(0..(lines * self.size.w) as usize);
        self.buffer.resize(self.size.into());
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

#[cfg(test)]
mod tests {
    use crate::editing::{motion::tests::window, Resizable};
    use indoc::indoc;

    use super::*;

    trait TestableDisplay {
        fn of_string(s: &'static str) -> Display;
        fn cursor_coords(&self) -> Option<(u16, u16)>;
        fn to_visual_string(&self) -> String;
        fn assert_visual_match(&self, s: &'static str);
    }

    impl TestableDisplay for Display {
        fn of_string(s: &'static str) -> Display {
            let width = s.find('\n').unwrap_or(s.len());
            let height = s.chars().filter(|ch| *ch == '\n').count();

            let mut display = Display::new(Size {
                w: width as u16,
                h: height as u16,
            });
            let mut win = window(s);
            win.window.resize(display.size);
            win.render(&mut display);

            display
        }

        fn cursor_coords(&self) -> Option<(u16, u16)> {
            match self.cursor {
                editing::Cursor::Block(x, y) => Some((x, y)),
                editing::Cursor::Line(x, y) => Some((x, y)),
                editing::Cursor::None => None,
            }
        }

        fn to_visual_string(&self) -> String {
            let mut s = String::default();

            for y in 0..self.size.h {
                s.push_str("\n");

                for x in 0..self.size.w {
                    if let Some(cursor) = self.cursor_coords() {
                        if (x, y) == cursor {
                            s.push_str("|");
                        }
                    }

                    let content = &self.buffer.get(x, y).symbol;
                    if content.is_empty() {
                        s.push_str("_");
                    } else {
                        s.push(content.chars().next().unwrap());
                    }
                }
            }

            s
        }

        fn assert_visual_match(&self, s: &'static str) {
            let expected_display = Display::of_string(s);
            assert_eq!(self.to_visual_string(), expected_display.to_visual_string());
        }
    }

    #[test]
    fn visual_match_test() {
        let display = Display::of_string(indoc! {"
            Take my love
            Take my land
        "});

        display.assert_visual_match(indoc! {"
            Take my love
            Take my land
        "});
    }

    #[test]
    fn shift_up_test() {
        let mut display = Display::of_string(indoc! {"
            Take my love
            Take my land
        "});
        display.shift_up(1);

        display.assert_visual_match(indoc! {"
            Take my land

        "});
    }
}
