use crate::editing::CursorPosition;

use super::{
    char::CharMotion,
    linewise::LineCrossing,
    word::{find, is_not_whitespace},
    {DirectionalMotion, Motion},
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

        let mut cursor = self.step.destination(context);

        let original_line = context.buffer().get(cursor.line);
        let was_on_boundary = cursor.col == 0
            || cursor.col == original_line.width()
            || self.is_on_boundary(context, cursor);

        if !was_on_boundary && !self.step.is_forward() {
            // skip past current word
            cursor = find(context, cursor, &self.step, &self.is_word_boundary);
            // while !self.is_on_boundary(context, cursor) {
            //     let next = self.step.destination(&context.with_cursor(cursor));
            //     if next == cursor {
            //         // cannot go further
            //         break;
            //     }
            //     cursor = next;
            // }
        }

        // skip past any whitespace
        cursor = find(context, cursor, &self.step, is_not_whitespace);

        if self.step.is_forward() && !self.is_on_boundary(context, cursor) {
            // continue to the end
            cursor = find(context, cursor, &self.step, &self.is_word_boundary);

            if self.is_on_boundary(context, cursor) {
                // step back onto the end of the word
                cursor.col = cursor.col.checked_sub(1).unwrap_or(0);
            }
        }

        cursor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use crate::editing::motion::word::is_small_word_boundary;
    use indoc::indoc;

    mod small_word {
        use super::*;

        #[test]
        fn forward_to_end() {
            let mut ctx = window(indoc! {"
                |Take my land
            "});
            ctx.motion(EndWordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Tak|e my land
            "});
        }

        #[test]
        fn forward_from_end_to_next_end() {
            let mut ctx = window(indoc! {"
                Tak|e my land
            "});
            ctx.motion(EndWordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take m|y land
            "});
        }

        #[test]
        fn back_from_space() {
            let mut ctx = window(indoc! {"
                Take my land    |
            "});
            ctx.motion(EndWordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my lan|d
            "});
        }

        #[test]
        fn back_to_previous_end() {
            let mut ctx = window(indoc! {"
                Take my lan|d
            "});
            ctx.motion(EndWordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take m|y land
            "});
        }
    }

    #[cfg(test)]
    mod line_crossing {
        use super::*;

        #[test]
        fn back_to_previous_end() {
            let mut ctx = window(indoc! {"
                Take my love
                Tak|e my land
            "});
            ctx.motion(EndWordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my lov|e
                Take my land
            "});
        }

        #[test]
        fn forward_to_next_end() {
            let mut ctx = window(indoc! {"
                Take my lov|e
                Take my land
            "});
            ctx.motion(EndWordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my love
                Tak|e my land
            "});
        }
    }
}
