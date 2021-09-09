use crate::editing::motion::char::CharMotion;
use crate::editing::motion::linewise::LineCrossing;
use crate::editing::motion::Motion;
use crate::editing::motion::MotionFlags;
use crate::editing::text::EditableLine;
use crate::editing::CursorPosition;

use super::util::search;

pub struct SearchMotion {
    step: LineCrossing<CharMotion>,
    query: String,
}

impl SearchMotion {
    pub fn backward_until(query: String) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Backward(1)),
            query,
        }
    }

    pub fn forward_until(query: String) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Forward(1)),
            query,
        }
    }
}

impl Motion for SearchMotion {
    fn flags(&self) -> MotionFlags {
        MotionFlags::EXCLUSIVE
    }

    fn destination<C: super::MotionContext>(&self, context: &C) -> CursorPosition {
        if context.buffer().lines_count() == 0 {
            return context.cursor();
        }

        let origin = context.cursor();
        let mut cursor = self.step.destination(context);
        let first_char = &self.query[0..1];

        loop {
            let (next_cursor, found) = search(context, cursor, &self.step, |c| c == first_char);
            if !found {
                return origin;
            }

            cursor = next_cursor;
            let line = context.buffer().get(cursor.line);
            if line.width() < cursor.col + self.query.len() {
                // Cannot possibly be a match
                continue;
            }

            let candidate = line.subs(cursor.col, self.query.len());
            if candidate.starts_with(&self.query) {
                // Found it! Return the cursor
                break;
            }
        }

        cursor
    }
}
