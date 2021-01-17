pub mod char;
pub mod linewise;

use super::{window::Window, Buffer, CursorPosition};

pub type MotionRange = (CursorPosition, CursorPosition);

pub trait MotionContext {
    fn buffer(&self) -> &Box<dyn Buffer>;
    fn cursor(&self) -> CursorPosition;
    fn window(&self) -> &Box<Window>;
    fn window_mut(&mut self) -> &mut Box<Window>;
}

pub trait Motion {
    fn destination<T: MotionContext>(&self, context: &T) -> CursorPosition;
    fn is_linewise(&self) -> bool {
        false
    }

    fn range<T: MotionContext>(&self, context: &T) -> MotionRange {
        let start = context.cursor();
        let end = self.destination(context);
        let linewise = self.is_linewise();
        if linewise && end < start {
            (end.start_of_line(), start.end_of_line(context.buffer()))
        } else if linewise {
            (start.start_of_line(), end.end_of_line(context.buffer()))
        } else if end < start {
            (end, start)
        } else {
            (start, end)
        }
    }

    fn apply_cursor<T: MotionContext>(&self, context: &mut T) {
        let new_cursor = self.destination(context);
        let new_cursor = context.window().clamp_cursor(context.buffer(), new_cursor);
        context.window_mut().cursor = new_cursor;
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::{buffer::MemoryBuffer, text::TextLines, window::Window, Buffer, HasId};

    use super::*;

    pub struct TestWindow {
        pub window: Box<Window>,
        pub buffer: Box<dyn Buffer>,
    }

    impl TestWindow {
        pub fn motion<T: Motion>(&mut self, motion: T) {
            motion.apply_cursor(self);
        }

        pub fn set_inserting(&mut self, inserting: bool) {
            self.window.set_inserting(inserting);
        }

        pub fn assert_visual_match(&self, s: &'static str) {
            let win = window(s);
            assert_eq!(self.cursor(), win.cursor());
        }
    }

    impl MotionContext for TestWindow {
        fn buffer(&self) -> &Box<dyn Buffer> {
            &self.buffer
        }

        fn cursor(&self) -> crate::editing::CursorPosition {
            self.window.cursor
        }

        fn window(&self) -> &Box<Window> {
            &self.window
        }

        fn window_mut(&mut self) -> &mut Box<Window> {
            &mut self.window
        }
    }

    pub fn window(s: &'static str) -> TestWindow {
        let s: String = s.into();
        let mut cursor = CursorPosition::default();
        for (index, line) in s.split("\n").enumerate() {
            if let Some(col) = line.find("|") {
                cursor.line = index;
                cursor.col = col as u16;
                break;
            }
        }

        let mut buffer = MemoryBuffer::new(0);
        let mut window = Window::new(0, buffer.id());

        buffer.append(TextLines::raw(s.replace("|", "")));
        window.cursor = cursor;

        TestWindow {
            window: Box::new(window),
            buffer: Box::new(buffer),
        }
    }
}
