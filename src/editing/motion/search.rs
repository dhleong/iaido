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
    pub fn backward_until<T: Into<String>>(query: T) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Backward(1)),
            query: query.into(),
        }
    }

    pub fn forward_until<T: Into<String>>(query: T) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Forward(1)),
            query: query.into(),
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
            if !found || next_cursor == cursor {
                return origin;
            }

            cursor = next_cursor;
            let line = context.buffer().get(cursor.line);
            let end = cursor.col + self.query.len();
            if line.width() < end {
                // Cannot possibly be a match
                continue;
            }

            let candidate = line.subs(cursor.col, end);
            if candidate.starts_with(&self.query) {
                // Found it! Return the cursor
                break;
            }
        }

        cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use indoc::indoc;

    mod forward_search {
        use super::*;

        #[test]
        fn within_line() {
            let mut ctx = window(indoc! {"
                |Take my land
            "});
            ctx.motion(SearchMotion::forward_until("my"));
            ctx.assert_visual_match(indoc! {"
                Take |my land
            "});
        }

        #[test]
        fn dont_move_without_match() {
            let mut ctx = window(indoc! {"
                |Take my land
            "});
            ctx.motion(SearchMotion::forward_until("alright"));
            ctx.assert_visual_match(indoc! {"
                |Take my land
            "});
        }
    }

    mod backward_search {
        use super::*;

        #[test]
        fn within_line() {
            let mut ctx = window(indoc! {"
                Take my |land
            "});
            ctx.motion(SearchMotion::backward_until("my"));
            ctx.assert_visual_match(indoc! {"
                Take |my land
            "});
        }
    }
}
