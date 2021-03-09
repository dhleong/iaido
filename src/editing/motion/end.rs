use crate::editing::CursorPosition;

use super::{
    char::CharMotion,
    linewise::LineCrossing,
    word::{find, is_not_whitespace},
    Motion,
};

pub struct EndWordMotion<T>
where
    T: Fn(&str) -> bool,
{
    step: LineCrossing<CharMotion>,
    is_word_boundary: T,
}

impl<T> EndWordMotion<T>
where
    T: Fn(&str) -> bool,
{
    pub fn backward_until(predicate: T) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Backward(1)),
            is_word_boundary: predicate,
        }
    }

    pub fn forward_until(predicate: T) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Forward(1)),
            is_word_boundary: predicate,
        }
    }

    fn is_on_boundary<C: super::MotionContext>(&self, context: &C, cursor: CursorPosition) -> bool {
        if let Some(ch) = context.buffer().get_char(cursor) {
            (self.is_word_boundary)(ch)
        } else {
            false
        }
    }
}

impl<T> Motion for EndWordMotion<T>
where
    T: Fn(&str) -> bool,
{
    fn destination<C: super::MotionContext>(&self, context: &C) -> CursorPosition {
        if context.buffer().lines_count() == 0 {
            return context.cursor();
        }

        let origin = context.cursor();
        let original_line = context.buffer().get(origin.line);
        let mut was_on_boundary =
            origin.col == original_line.width() || self.is_on_boundary(context, origin);
        let mut cursor = self.step.destination(context);

        if cursor < origin {
            // special case: skip past any whitespace
            cursor = find(context, cursor, &self.step, is_not_whitespace);
            was_on_boundary = self.is_on_boundary(context, cursor);
        }

        let now_on_boundary = self.is_on_boundary(context, cursor);
        if !was_on_boundary || now_on_boundary {
            // find the next boundary
            cursor = find(context, cursor, &self.step, &self.is_word_boundary);
        }

        if cursor > origin {
            // special case: skip until the first non-whitespace
            cursor = find(context, cursor, &self.step, is_not_whitespace);
        } else if !was_on_boundary && self.is_on_boundary(context, cursor) {
            cursor =
                LineCrossing::new(CharMotion::Forward(1)).destination(&context.with_cursor(cursor));
        }

        cursor
    }
}
