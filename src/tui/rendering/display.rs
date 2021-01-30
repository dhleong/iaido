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
