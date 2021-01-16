use std::cmp::{max, min};

use crate::tui::measure::Measurable;

use super::{buffers::Buffers, Buffer, CursorPosition, HasId, Id, Resizable, Size};

pub struct Window {
    pub id: Id,
    pub buffer: Id,
    pub size: Size,
    pub focused: bool,
    pub inserting: bool,

    pub cursor: CursorPosition,

    /// number of lines from the bottom that we've scrolled
    pub scrolled_lines: u32,
    /// the visual-line offset within the current (bottom-most) line
    pub scroll_offset: u16,
}

impl Window {
    pub fn new(id: Id, buffer_id: Id) -> Self {
        Self {
            id,
            buffer: buffer_id,
            size: Size { w: 0, h: 0 },
            focused: true,
            inserting: false,
            cursor: CursorPosition { line: 0, col: 0 },
            scrolled_lines: 0,
            scroll_offset: 0,
        }
    }

    pub fn current_buffer<'a>(&self, buffers: &'a Buffers) -> &'a Box<dyn Buffer> {
        buffers.by_id(self.buffer).unwrap()
    }

    pub fn current_buffer_mut<'a>(&self, buffers: &'a mut Buffers) -> &'a mut Box<dyn Buffer> {
        buffers.by_id_mut(self.buffer).unwrap()
    }

    pub fn set_focused(&mut self, focused: bool) {
        self.focused = focused;
    }

    pub fn set_inserting(&mut self, inserting: bool) {
        self.inserting = inserting;
    }

    /// Scroll the window "back in time" by the given number of "virtual" (visual) lines.
    /// Pass a negative value for `virtual_lines` to scroll "forward in time" (toward the bottom of
    /// the screen)
    pub fn scroll_lines(&mut self, buffers: &Buffers, virtual_lines: i32) {
        let buffer = buffers.by_id(self.buffer).expect("Window buffer missing");
        if buffer.lines_count() == 0 || self.size.w <= 0 {
            // nop
            return;
        }

        let to_scroll = virtual_lines.abs();
        let step = virtual_lines / to_scroll;
        if step > 0 {
            self.scroll_up(buffer, to_scroll as usize);
        } else {
            self.scroll_down(buffer, to_scroll as usize);
        }
    }

    fn scroll_up(&mut self, buffer: &Box<dyn Buffer>, virtual_lines: usize) {
        let end = buffer.lines_count() - 1;
        let mut to_scroll = virtual_lines;

        let window_width = self.size.w;
        for line_nr in (0..(buffer.lines_count() - self.scrolled_lines as usize)).rev() {
            let line = buffer.get(line_nr);
            let consumable = line.measure_height(window_width) - self.scroll_offset;

            self.scroll_offset += min(to_scroll, consumable as usize) as u16;
            if to_scroll < consumable as usize {
                // done!
                break;
            }

            if self.scrolled_lines as usize >= end {
                // start of buffer reached and still scrolling; cancel
                break;
            }

            to_scroll -= consumable as usize;
            self.scroll_offset = 0;

            // finish scrolling past a wrapped line
            if to_scroll == 0 {
                break;
            }
        }

        if self.scrolled_lines as usize == end {
            // last buffer line; ensure we don't offset-scroll it out of visible range
            let line = buffer.get(end);
            let rendered = line.measure_height(window_width);
            self.scroll_offset = min(max(self.scroll_offset, 0), rendered - 1);
        }
    }

    fn scroll_down(&mut self, buffer: &Box<dyn Buffer>, virtual_lines: usize) {
        todo!();
    }

    pub fn set_scroll(&mut self, lines: u32, offset: u16) {
        self.scrolled_lines = lines;
        self.scroll_offset = offset;
    }
}

impl HasId for Window {
    fn id(&self) -> Id {
        return self.id;
    }
}

impl Resizable for Window {
    fn resize(&mut self, new_size: Size) {
        self.size = new_size
    }
}

impl std::fmt::Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "[TuiWindow#{}]", self.id);
    }
}
