use std::cmp::min;

use crate::{
    editing::{buffer::MemoryBuffer, text::TextLine, window::Window, Buffer, Resizable, Size},
    tui::measure::Measurable,
};

/// The Prompt is used to render prompts, cmdline mode, search modes, etc
pub struct Prompt {
    pub buffer: Box<dyn Buffer>,
    pub window: Window,
    max_height: u16,
}

impl Default for Prompt {
    fn default() -> Self {
        Self {
            buffer: Box::new(MemoryBuffer::new(0)),
            window: Window::new(0, 0),
            max_height: 10,
        }
    }
}

impl Prompt {
    pub fn handle_content_change(&mut self) {
        self.resize(Size {
            w: self.window.size.w,
            h: self.max_height,
        });
    }
}

impl Resizable for Prompt {
    fn resize(&mut self, new_size: crate::editing::Size) {
        let height = self.buffer.measure_height(new_size.w);
        self.max_height = new_size.h;
        self.window.resize(Size {
            w: new_size.w,
            h: min(height, new_size.h),
        });
    }
}
