use crate::editing::motion::char::CharMotion;
use crate::editing::motion::linewise::LineCrossing;
use crate::editing::motion::Motion;
use crate::editing::motion::MotionContext;
use crate::editing::motion::MotionFlags;
use crate::editing::text::EditableLine;
use crate::editing::CursorPosition;

use super::util::search;

pub enum LoopBehavior {
    TopDown,
    BottomUp,
}

impl LoopBehavior {
    fn destination<C: MotionContext>(&self, context: &C) -> Option<CursorPosition> {
        match self {
            LoopBehavior::TopDown => Some(CursorPosition::from((0, 0))),
            LoopBehavior::BottomUp => {
                if let Some(last_line) = context.buffer().last_index() {
                    Some(CursorPosition::from((last_line, 0)).end_of_line(context.buffer()))
                } else {
                    None
                }
            }
        }
    }
}

pub struct SearchMotion {
    step: LineCrossing<CharMotion>,
    query: String,
    loop_behavior: LoopBehavior,
}

impl SearchMotion {
    pub fn backward_until<T: Into<String>>(query: T) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Backward(1)),
            query: query.into(),
            loop_behavior: LoopBehavior::BottomUp,
        }
    }

    pub fn forward_until<T: Into<String>>(query: T) -> Self {
        Self {
            step: LineCrossing::new(CharMotion::Forward(1)),
            query: query.into(),
            loop_behavior: LoopBehavior::TopDown,
        }
    }

    fn destination_from_cursor<C: MotionContext>(
        &self,
        context: &C,
        origin: CursorPosition,
    ) -> CursorPosition {
        if context.buffer().lines_count() == 0 {
            return origin;
        }

        let mut cursor = self.step.destination(&context.with_cursor(origin));
        let first_char = &self.query[0..1];

        loop {
            let (next_cursor, found) = search(context, cursor, &self.step, |c| c == first_char);
            if !found {
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

            if next_cursor == cursor {
                // No match found, and cursor unmoved; this is a sanity check:
                return origin;
            }

            cursor = self.step.destination(&context.with_cursor(cursor));
        }

        cursor
    }
}

impl Motion for SearchMotion {
    fn flags(&self) -> MotionFlags {
        MotionFlags::EXCLUSIVE
    }

    fn destination<C: MotionContext>(&self, context: &C) -> CursorPosition {
        let origin = context.cursor();
        match self.destination_from_cursor(context, origin) {
            result if result == origin => {
                // NOTE: We get one shot to loop around
                let new_origin = self.loop_behavior.destination(context).unwrap_or(origin);
                match self.destination_from_cursor(context, new_origin) {
                    result if result == new_origin => {
                        // Still nothing; return the original origin to indicate no result
                        origin
                    }

                    // Success!
                    loop_result => loop_result,
                }
            }
            result => result,
        }
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
        fn repeated_within_line() {
            let mut ctx = window(indoc! {"
                |Take my land
            "});
            ctx.motion(SearchMotion::forward_until("a"));
            ctx.assert_visual_match(indoc! {"
                T|ake my land
            "});
            ctx.motion(SearchMotion::forward_until("a"));
            ctx.assert_visual_match(indoc! {"
                Take my l|and
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

        #[test]
        fn loop_to_top() {
            let mut ctx = window(indoc! {"
                Take my |land
            "});
            ctx.motion(SearchMotion::forward_until("my"));
            ctx.assert_visual_match(indoc! {"
                Take |my land
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

        #[test]
        fn loop_to_bottom() {
            let mut ctx = window(indoc! {"
                Take |my land
            "});
            ctx.motion(SearchMotion::backward_until("lan"));
            ctx.assert_visual_match(indoc! {"
                Take my |land
            "});
        }
    }
}
