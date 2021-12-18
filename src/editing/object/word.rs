use crate::editing::motion::{
    char::CharMotion, end::EndOfWordMotion, linewise::LineCrossing, word::WordMotion, Motion,
    MotionContext, MotionRange,
};

use super::TextObject;

pub struct WordObject<T>
where
    T: Fn(&str) -> bool,
{
    inner: bool,
    is_word_boundary: T,
}

impl<T> WordObject<T>
where
    T: Fn(&str) -> bool,
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
    T: Fn(&str) -> bool,
{
    fn object_range<C: MotionContext>(&self, context: &C) -> MotionRange {
        if context.buffer().lines_count() == 0 {
            let c = context.cursor();
            return (c, c).into();
        }

        let previous_word_end = EndOfWordMotion::backward_until(|ctx| (self.is_word_boundary)(ctx))
            .destination(context);
        let start = if self.inner {
            WordMotion::forward_until(|ctx| (self.is_word_boundary)(ctx))
                .destination(&context.with_cursor(previous_word_end))
        } else if previous_word_end.col > 0 {
            LineCrossing::new(CharMotion::Forward(1))
                .destination(&context.with_cursor(previous_word_end))
        } else {
            previous_word_end
        };

        let end = EndOfWordMotion::forward_until(|ctx| (self.is_word_boundary)(ctx))
            .destination(&context.with_cursor(start));

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
    }
}
