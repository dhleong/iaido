use crate::editing::CursorPosition;

use super::{char::CharMotion, linewise::LineCrossing, Motion};

pub fn is_big_word_boundary(ch: &str) -> bool {
    ch == " "
}

pub fn is_small_word_boundary(ch: &str) -> bool {
    ch.find(char::is_alphanumeric).is_none()
}

pub fn is_not_whitespace(ch: &str) -> bool {
    ch.find(char::is_whitespace).is_none()
}

pub struct WordMotion<T>
where
    T: Fn(&str) -> bool,
{
    step: LineCrossing<CharMotion>,
    is_word_boundary: T,
}

impl<T> WordMotion<T>
where
    T: Fn(&str) -> bool,
{
    pub fn backward_until(predicate: T) -> Self {
        WordMotion {
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

impl<T> Motion for WordMotion<T>
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
            origin.col as usize == original_line.width() || self.is_on_boundary(context, origin);
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

fn find<C: super::MotionContext, M: Motion, P: Fn(&str) -> bool>(
    context: &C,
    start: CursorPosition,
    step: &M,
    pred: P,
) -> CursorPosition {
    let mut cursor = start;

    loop {
        if let Some(ch) = context.buffer().get_char(cursor) {
            if pred(ch) {
                break;
            } else {
                let next = step.destination(&context.with_cursor(cursor));
                if next == cursor {
                    // our step didn't move; we can't go further
                    break;
                }
                cursor = next;
            }
        } else {
            // can't go further
            return cursor;
        }
    }

    cursor
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use indoc::indoc;

    mod small_word {
        use super::*;

        #[test]
        fn forward_past_whitespace() {
            let mut ctx = window(indoc! {"
                |Take my land
            "});
            ctx.motion(WordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take |my land
            "});
        }

        #[test]
        fn forward_past_symbol() {
            let mut ctx = window(indoc! {"
                |'Take' my land
            "});
            ctx.motion(WordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                '|Take' my land
            "});
        }

        #[test]
        fn forward_until_symbol() {
            let mut ctx = window(indoc! {"
                |Take 'my' land
            "});
            ctx.motion(WordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take |'my' land
            "});
        }

        #[test]
        fn backward_past_whitespace() {
            let mut ctx = window(indoc! {"
                Take |my land
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                |Take my land
            "});
        }

        #[test]
        fn backward_past_whitespace2() {
            let mut ctx = window(indoc! {"
                Take my |land
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take |my land
            "});
        }

        #[test]
        fn backward_past_span() {
            let mut ctx = window(indoc! {"
                Take my love |land
            "});

            // split up the span by deleting a range:
            ctx.buffer
                .delete_range(((0, 7).into(), (0, 12).into()).into());
            ctx.window.cursor = (0, 8).into();
            ctx.assert_visual_match(indoc! {"
                Take my |land
            "});

            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take |my land
            "});
        }

        #[test]
        fn backward_to_start() {
            let mut ctx = window(indoc! {"
                Take my lan|d
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my |land
            "});
        }

        #[test]
        fn backward_from_end() {
            let mut ctx = window(indoc! {"
                Take my land|
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my |land
            "});
        }

        #[test]
        fn backward_to_start_of_second_line() {
            let mut ctx = window(indoc! {"
                Take my love
                Take |my land
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my love
                |Take my land
            "});
        }

        #[test]
        fn backward_to_start_of_first_line() {
            let mut ctx = window(indoc! {"
                Take |my love
                Take my land
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                |Take my love
                Take my land
            "});
        }
    }

    mod big_word {
        use super::*;

        #[test]
        fn forward_past_symbols() {
            let mut ctx = window(indoc! {"
                |'Take' my land
            "});
            ctx.motion(WordMotion::forward_until(is_big_word_boundary));
            ctx.assert_visual_match(indoc! {"
                'Take' |my land
            "});
        }

        #[test]
        fn backward_past_symbols() {
            let mut ctx = window(indoc! {"
                Take 'my|' land
            "});
            ctx.motion(WordMotion::backward_until(is_big_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take |'my' land
            "});
        }

        #[test]
        fn backward_past_symbols_and_space() {
            let mut ctx = window(indoc! {"
                Take 'my' |land
            "});
            ctx.motion(WordMotion::backward_until(is_big_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take |'my' land
            "});
        }
    }

    mod across_lines {
        use super::*;

        #[test]
        fn direct_backwards_test() {
            let mut ctx = window(indoc! {"
                Take my love
                |Take my land
            "});
            ctx.motion(WordMotion::backward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my |love
                Take my land
            "});
        }

        #[test]
        fn direct_forward_test() {
            let mut ctx = window(indoc! {"
                Take my |love
                Take my land
            "});
            ctx.motion(WordMotion::forward_until(is_small_word_boundary));
            ctx.assert_visual_match(indoc! {"
                Take my love
                |Take my land
            "});
        }
    }
}
