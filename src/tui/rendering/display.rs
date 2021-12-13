use tui::{buffer::Cell, layout::Rect};

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

    pub fn clear(&mut self, area: Rect) {
        let width: usize = area.width.into();
        for y in area.y..area.y + area.height {
            let start = self.buffer.index_of(area.x, y);
            for x in start..start + width {
                self.buffer.content[x] = Cell::default();
            }
        }
    }

    pub fn merge_at_y(&mut self, y: u16, other: Display) {
        if other.size != self.size {
            panic!(
                "other.size({:?}) does not match self.size({:?})",
                other.size, self.size
            );
        }

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
        self.cursor = self.cursor - (0, lines);
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
pub mod tests {
    use crate::editing::motion::tests::window;
    use indoc::indoc;
    use tui::style::Style;

    use super::*;

    pub trait TestableDisplay {
        fn of_string(s: &'static str) -> Display;
        fn of_sized_string<S: Into<Size>>(size: S, s: &'static str, inserting: bool) -> Display;
        fn cursor_coords(&self) -> Option<(u16, u16)>;
        fn to_visual_string(&self) -> String;
        fn assert_visual_match(&self, s: &'static str);
        fn assert_visual_equals(&self, s: &'static str);
    }

    impl TestableDisplay for Display {
        fn of_string(s: &'static str) -> Display {
            let width = s
                .split('\n')
                .map(|l| l.replace('|', "").len())
                .max()
                .unwrap_or(s.len());
            let height = s.chars().filter(|ch| *ch == '\n').count();

            return Display::of_sized_string(
                Size {
                    w: width as u16,
                    h: height as u16,
                },
                s,
                false,
            );
        }

        fn of_sized_string<S: Into<Size>>(size: S, s: &'static str, inserting: bool) -> Display {
            let mut display = Display::new(size.into());
            let mut win = window(s);
            if inserting {
                win.set_inserting(true);
            }
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
            let expected_display = Display::of_sized_string(
                self.size,
                s,
                match self.cursor {
                    editing::Cursor::Line(_, _) => true,
                    _ => false,
                },
            );
            assert_eq!(self.size, expected_display.size);
            assert_eq!(self.to_visual_string(), expected_display.to_visual_string());
        }

        fn assert_visual_equals(&self, s: &'static str) {
            let mut expected_display = Display::new(self.size);
            for (i, line) in s.split("\n").enumerate() {
                if i as u16 == self.size.h && line.is_empty() {
                    break;
                }

                expected_display
                    .buffer
                    .set_string(0, i as u16, line, Style::default());
            }

            // TODO support cursor?
            assert_eq!(
                self.to_visual_string().replace("|", ""),
                expected_display.to_visual_string()
            );
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
            |Take my land
        "});
        display.shift_up(1);

        display.assert_visual_match(indoc! {"
            |Take my land

        "});
    }

    #[test]
    fn merge_at_y_works() {
        let mut display = Display::of_string(indoc! {"
            Take my love

        "});

        let to_merge = Display::of_string(indoc! {"
            _
            Take my land
        "});
        display.merge_at_y(1, to_merge);

        display.assert_visual_match(indoc! {"
            Take my love
            Take my land
        "});
    }
}
