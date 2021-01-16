pub mod linewise;

use super::{Buffer, CursorPosition};

pub type MotionRange = (CursorPosition, CursorPosition);

pub trait MotionContext {
    fn buffer(&self) -> &Box<dyn Buffer>;
    fn cursor(&self) -> CursorPosition;
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
}
