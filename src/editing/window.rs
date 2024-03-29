use bitflags::bitflags;

use std::cmp::{max, min};

use crate::editing::gutter::Gutter;
use crate::input::maps::KeyResult;
use crate::input::KeyError;
use crate::{
    input::completion::{state::CompletionState, Completion},
    tui::measure::Measurable,
};

use super::{buffers::Buffers, Buffer, CursorPosition, HasId, Id, Resizable, Size};

bitflags! {
    pub struct WindowFlags: u8 {
        const NONE = 0;

        /// A PROTECTED window is not normally closeable, but the contextually
        /// it may be closed depending on its content. For example, the
        /// "main" window of a Connection is PROTECTED but may be closed
        /// if the connection is not active
        const PROTECTED = 0b01;

        /// A LOCKED_BUFFER window may not have its buffer changed
        const LOCKED_BUFFER = 0b10;
    }
}

pub struct Window {
    pub id: Id,
    pub buffer: Id,
    pub size: Size,
    pub focused: bool,
    pub inserting: bool,
    pub flags: WindowFlags,
    pub gutter: Option<Gutter>,

    pub cursor: CursorPosition,

    /// number of lines from the bottom that we've scrolled
    pub scrolled_lines: u32,
    /// the visual-line offset within the current (bottom-most) line
    pub scroll_offset: u16,

    pub completion_state: Option<CompletionState>,
}

impl Window {
    pub fn new(id: Id, buffer_id: Id) -> Self {
        Window::with_focused(id, buffer_id, true)
    }

    pub fn with_focused(id: Id, buffer_id: Id, focused: bool) -> Self {
        Self {
            id,
            buffer: buffer_id,
            size: Size { w: 0, h: 0 },
            focused,
            flags: WindowFlags::NONE,
            gutter: None,
            inserting: false,
            cursor: CursorPosition { line: 0, col: 0 },
            scrolled_lines: 0,
            scroll_offset: 0,
            completion_state: None,
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
        self.completion_state = None; // reset on mode change
    }

    pub fn apply_completion(&mut self, new: &Completion) {
        self.cursor = new.replacement_end();
    }

    /// Scroll the window "back in time" by the given number of "virtual" (visual) lines.
    /// Pass a negative value for `virtual_lines` to scroll "forward in time" (toward the bottom of
    /// the screen)
    pub fn scroll_lines(&mut self, buffer: &Box<dyn Buffer>, virtual_lines: i32) {
        if buffer.is_empty() || self.size.w <= 0 || virtual_lines == 0 {
            // nop
            return;
        }

        let to_scroll = virtual_lines.abs();
        if virtual_lines > 0 {
            self.scroll_up(buffer, to_scroll as usize);
        } else {
            self.scroll_down(buffer, to_scroll as usize);
        }
    }

    fn scroll_up(&mut self, buffer: &Box<dyn Buffer>, virtual_lines: usize) {
        let end = buffer.last_index().expect("Empty buffer somewhow?");
        let mut to_scroll = virtual_lines;

        let window_width = self.size.w;
        for line_nr in (0..(buffer.lines_count() - self.scrolled_lines as usize)).rev() {
            let line = buffer.get(line_nr);
            let consumable = line.measure_height(window_width) - self.scroll_offset;
            let last_visible_line = end - (self.scrolled_lines as usize);

            self.scroll_offset += min(to_scroll, consumable as usize) as u16;

            // when scrolling through offsets, ensure the cursor col isn't
            // out of view
            if self.cursor.line >= last_visible_line {
                let last_visible_offset = consumable.checked_sub(self.scroll_offset).unwrap_or(0);
                let rows_delta = (self.cursor.col / (window_width as usize))
                    .checked_sub(last_visible_offset as usize)
                    .unwrap_or(0);
                self.cursor.col = self
                    .cursor
                    .col
                    .checked_sub(rows_delta * window_width as usize)
                    .unwrap_or(0);
            }

            if to_scroll < consumable as usize {
                // done!
                break;
            }

            // when scrolling through lines, ensure the cursor line isn't
            // out of view
            if self.cursor.line >= last_visible_line {
                self.cursor.line = self.cursor.line.checked_sub(1).unwrap_or(0);
            }

            if self.scrolled_lines as usize >= end {
                // start of buffer reached and still scrolling; cancel
                break;
            }

            to_scroll -= consumable as usize;
            self.scroll_offset = 0;
            self.scrolled_lines += 1;

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
        let end = buffer.lines_count();
        let mut to_scroll = virtual_lines;

        let window_width = self.size.w;
        for _ in (end - self.scrolled_lines as usize)..=end {
            let initial_offset = self.scroll_offset as usize;
            // NOTE: there's always at least one:
            let consumable = initial_offset + 1;
            self.scroll_offset = (self.scroll_offset as usize)
                .checked_sub(to_scroll)
                .unwrap_or(0) as u16;

            let first_visible_line = (end - self.scrolled_lines as usize)
                .checked_sub(self.size.h as usize)
                .unwrap_or(0);

            // when scrolling through offsets, ensure the cursor col isn't
            // out of view
            if self.cursor.line <= first_visible_line {
                let first_visible_offset = initial_offset
                    .checked_sub(self.scroll_offset as usize)
                    .unwrap_or(0);
                let rows_delta = (first_visible_offset as usize)
                    .checked_sub(self.cursor.col / (window_width as usize))
                    .unwrap_or(0);
                self.cursor.col += rows_delta * window_width as usize;
            }

            if to_scroll < consumable {
                // done!
                break;
            }

            if self.scrolled_lines == 0 {
                // no further to go
                break;
            }

            // when scrolling through lines, ensure the cursor line isn't
            // out of view
            if self.cursor.line <= first_visible_line {
                self.cursor.line += 1;
            }

            to_scroll = to_scroll - consumable;
            self.scrolled_lines -= 1;

            if self.scrolled_lines == 0 {
                break;
            }

            let line = buffer.get(end - self.scrolled_lines as usize);
            self.scroll_offset = line.measure_height(window_width) - 1;

            if to_scroll == 0 {
                break;
            }
        }
    }

    /// Given a CursorPosition meant to replace the one currently set on this Window, return a new
    /// CursorPosition that is guaranteed to be valid for this window, taking into account insert
    /// mode, buffer line width, etc
    pub fn clamp_cursor(&self, buffer: &Box<dyn Buffer>, cursor: CursorPosition) -> CursorPosition {
        if let Some(width) = buffer.get_line_width(cursor.line) {
            let max_index = if self.inserting {
                width
            } else if width > 0 {
                width - 1
            } else {
                0
            };

            CursorPosition {
                line: cursor.line,
                col: min(max_index, cursor.col),
            }
        } else {
            CursorPosition::default()
        }
    }

    pub fn check_buffer_change(&self) -> KeyResult {
        if self.flags.contains(WindowFlags::LOCKED_BUFFER) {
            Err(KeyError::NotPermitted(
                "Cannot change buffer for window".to_string(),
            ))
        } else {
            Ok(())
        }
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
