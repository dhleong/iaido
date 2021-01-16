use super::Motion;

pub struct ToLineStartMotion {}

impl Motion for ToLineStartMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> crate::editing::CursorPosition {
        context.cursor().start_of_line()
    }
}

pub struct ToLineEndMotion {}

impl Motion for ToLineEndMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> crate::editing::CursorPosition {
        context.cursor().end_of_line(context.buffer())
    }
}
