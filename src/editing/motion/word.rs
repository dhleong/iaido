use crate::editing::CursorPosition;

use super::{char::CharMotion, Motion};

pub fn is_big_word_boundary(ch: &str) -> bool {
    ch == " "
}

pub fn is_small_word_boundary(ch: &str) -> bool {
    ch.find(char::is_alphanumeric).is_none()
}

pub struct WordMotion<T>
where
    T: Fn(&str) -> bool,
{
    step: CharMotion,
    is_word_boundary: T,
}

impl<T> WordMotion<T>
where
    T: Fn(&str) -> bool,
{
    pub fn backward_until(predicate: T) -> Self {
        WordMotion {
            step: CharMotion::Backward(1),
            is_word_boundary: predicate,
        }
    }

    pub fn forward_until(predicate: T) -> Self {
        Self {
            step: CharMotion::Forward(1),
            is_word_boundary: predicate,
        }
    }
}

impl<T> Motion for WordMotion<T>
where
    T: Fn(&str) -> bool,
{
    fn destination<C: super::MotionContext>(&self, context: &C) -> CursorPosition {
        let mut cursor = context.cursor().clone();

        // first, find the first boundary
        cursor = find(context, cursor, &self.step, &self.is_word_boundary);

        // next, skip until the first non-boundary
        cursor = find(context, cursor, &self.step, |ch| {
            !(self.is_word_boundary)(ch)
        });

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

    // TODO continue across lines
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

        // // ?
        // #[test]
        // fn forward_until_symbol() {
        //     let mut ctx = window(indoc! {"
        //         |Take 'my' land
        //     "});
        //     ctx.motion(WordMotion::forward_until(is_small_word_boundary));
        //     ctx.assert_visual_match(indoc! {"
        //         Take |'my' land
        //     "});
        // }
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
    }
}
