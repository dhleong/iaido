use crate::editing::motion::{
    char::CharMotion, end::EndOfWordMotion, word::WordMotion, Motion, MotionContext, MotionRange,
};

use super::{util::follow_whitespace, TextObject};

pub struct WordObject<T>
where
    T: Fn(char) -> bool,
{
    inner: bool,
    is_word_boundary: T,
}

impl<T> WordObject<T>
where
    T: Fn(char) -> bool,
{
    pub fn inner(predicate: T) -> Self {
        WordObject {
            inner: true,
            is_word_boundary: predicate,
        }
    }

    pub fn outer(predicate: T) -> Self {
        WordObject {
            inner: false,
            is_word_boundary: predicate,
        }
    }
}

impl<T> TextObject for WordObject<T>
where
    T: Fn(char) -> bool,
{
    fn object_range<C: MotionContext>(&self, context: &C) -> MotionRange {
        if context.buffer().lines_count() == 0 {
            let c = context.cursor();
            return (c, c).into();
        }

        let word_end = EndOfWordMotion::forward_until(|ctx| (self.is_word_boundary)(ctx))
            .destination(&context.with_cursor(CharMotion::Backward(1).destination(context)));
        let end = if !self.inner {
            follow_whitespace(context, word_end, CharMotion::Forward(1))
        } else {
            word_end
        };

        let previous_word_end = EndOfWordMotion::backward_until(|ctx| (self.is_word_boundary)(ctx))
            .destination(context);
        let start = if self.inner || end > word_end {
            // For inner word, and if there was trailing whitespace,
            // we start at the start of the current word
            WordMotion::forward_until(|ctx| (self.is_word_boundary)(ctx))
                .destination(&context.with_cursor(previous_word_end))
        } else if previous_word_end.col > 0 {
            // Outer word without trailing whitespace; use leading whitespace
            CharMotion::Forward(1).destination(&context.with_cursor(previous_word_end))
        } else {
            previous_word_end
        };

        (start, end).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use indoc::indoc;

    mod small_word {
        use super::*;
        use crate::editing::motion::word::is_small_word_boundary;

        #[test]
        fn inner() {
            let ctx = window(indoc! {"
                Al pastor qu|eso burrito
            "});
            assert_eq!(
                ctx.select(WordObject::inner(is_small_word_boundary)),
                "queso"
            );
        }

        #[test]
        fn inner_single() {
            let ctx = window(indoc! {"
                Al pastor |a burrito
            "});
            assert_eq!(ctx.select(WordObject::inner(is_small_word_boundary)), "a");
        }

        #[test]
        fn outer() {
            let ctx = window(indoc! {"
                Al pastor qu|eso burrito
            "});
            assert_eq!(
                ctx.select(WordObject::outer(is_small_word_boundary)),
                "queso "
            );
        }

        #[test]
        fn outer_single() {
            let ctx = window(indoc! {"
                Al pastor |a burrito
            "});
            assert_eq!(ctx.select(WordObject::outer(is_small_word_boundary)), "a ");
        }
    }

    mod big_word {
        use super::*;
        use crate::editing::motion::word::is_big_word_boundary;

        #[test]
        fn inner() {
            let ctx = window(indoc! {"
                Al pastor qu|e'so burrito
            "});
            assert_eq!(
                ctx.select(WordObject::inner(is_big_word_boundary)),
                "que'so"
            );
        }

        #[test]
        fn outer() {
            let ctx = window(indoc! {"
                Al pastor qu|e'so burrito
            "});
            assert_eq!(
                ctx.select(WordObject::outer(is_big_word_boundary)),
                "que'so "
            );
        }
    }
}
