use std::cmp::min;

use crate::{
    editing::{
        buffer::MemoryBuffer, text::TextLine, window::Window, Buffer, CursorPosition, Resizable,
        Size,
    },
    tui::measure::Measurable,
};

/// The Prompt is used to render prompts, cmdline mode, search modes, etc
pub struct Prompt {
    pub buffer: Box<dyn Buffer>,
    pub window: Box<Window>,
    max_height: u16,
}

impl Default for Prompt {
    fn default() -> Self {
        let mut window = Window::new(0, 0);
        window.focused = false;

        Self {
            buffer: Box::new(MemoryBuffer::new(0)),
            window: Box::new(window),
            max_height: 10,
        }
    }
}

impl Prompt {
    pub fn activate(&mut self, prompt: TextLine) {
        self.clear();
        self.window.focused = true;
        self.window.inserting = true;
        self.window.cursor = CursorPosition {
            line: 0,
            col: prompt.width(),
        };
        self.buffer.append(prompt.into());
        self.handle_content_change();
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
        self.window.focused = false;
        self.handle_content_change();
    }

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
