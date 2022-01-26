use crate::editing::{
    motion::{
        char::CharMotion, linewise::LineCrossing, Motion, MotionContext, MotionFlags, MotionRange,
    },
    CursorPosition,
};

use super::{util::follow_whitespace, TextObject};

pub struct InnerPairObject {
    start: char,
    end: char,
    line_crossing: bool,
}

impl InnerPairObject {
    pub fn new(start: char, end: char) -> Self {
        Self {
            start,
            end,
            line_crossing: true,
        }
    }

    pub fn within_line(start: char, end: char) -> Self {
        Self {
            start,
            end,
            line_crossing: false,
        }
    }

    pub fn into_outer(self) -> OuterPairObject {
        OuterPairObject { inner: self }
    }

    fn step<C: MotionContext>(
        &self,
        context: &C,
        cursor: CursorPosition,
        motion: CharMotion,
    ) -> CursorPosition {
        let ctx = context.with_cursor(cursor);
        if self.line_crossing {
            LineCrossing::new(motion).destination(&ctx)
        } else {
            motion.destination(&ctx)
        }
    }
}

fn char_at<C: MotionContext>(context: &C, cursor: CursorPosition) -> char {
    if context.buffer().is_empty() {
        return '\0';
    }

    if let Some(ch) = context.buffer().get_char(cursor) {
        ch
    } else {
        '\0'
    }
}

impl TextObject for InnerPairObject {
    fn object_range<C: MotionContext>(&self, context: &C) -> MotionRange {
        let mut start = context.cursor();
        let mut end = context.cursor();
        let mut found_start = false;
        let mut found_end = false;

        while char_at(context, start) != self.start {
            let new_start = self.step(context, start, CharMotion::Backward(1));
            if new_start == start {
                break;
            }
            start = new_start;
        }

        if char_at(context, start) != self.start {
            start = context.cursor();
        } else {
            found_start = true;
            start.col += 1;
        }

        while char_at(context, end) != self.end {
            let new_end = self.step(context, end, CharMotion::Forward(1));
            if new_end == end {
                break;
            }
            end = new_end;
        }

        if char_at(context, end) != self.end {
            end = context.cursor();
        } else {
            found_end = true;
            end.col -= 1;
        }

        MotionRange(
            start,
            end,
            if found_start && found_end {
                MotionFlags::NONE
            } else {
                MotionFlags::EXCLUSIVE
            },
        )
    }
}

pub struct OuterPairObject {
    inner: InnerPairObject,
}

impl TextObject for OuterPairObject {
    fn object_range<C: MotionContext>(&self, context: &C) -> MotionRange {
        let mut range = self.inner.object_range(context);
        if range.is_empty() || range.0.col == 0 {
            return range;
        }

        // expand to include start/end
        range.0.col -= 1;
        range.1.col += 1;

        // Include trailing whitespace if it exists...
        let original_end = range.1;
        range.1 = follow_whitespace(context, range.1, CharMotion::Forward(1));

        // ... else leading
        if range.1 == original_end {
            range.0 = follow_whitespace(context, range.0, CharMotion::Backward(1));
        }

        range
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use indoc::indoc;

    #[cfg(test)]
    mod quoted_string {
        use super::*;

        #[test]
        fn non_existent() {
            let ctx = window(indoc! {"
                Al pastor qu|eso burrito
            "});
            assert_eq!(ctx.select(InnerPairObject::within_line('\'', '\'')), "");
        }

        #[test]
        fn inner() {
            let ctx = window(indoc! {"
                Al pastor 'qu|eso' burrito
            "});
            assert_eq!(
                ctx.select(InnerPairObject::within_line('\'', '\'')),
                "queso"
            );
        }

        #[test]
        fn outer() {
            let ctx = window(indoc! {"
                Al pastor 'qu|eso' burrito
            "});
            assert_eq!(
                ctx.select(InnerPairObject::within_line('\'', '\'').into_outer()),
                "'queso' "
            );
        }

        #[test]
        fn outer_from_empty() {
            let ctx = window("");
            assert_eq!(
                ctx.select(InnerPairObject::within_line('\'', '\'').into_outer()),
                ""
            );
        }

        #[test]
        fn outer_with_leading_whitespace() {
            let ctx = window(indoc! {"
                Al pastor queso 'burr|ito'
            "});
            assert_eq!(
                ctx.select(InnerPairObject::within_line('\'', '\'').into_outer()),
                " 'burrito'"
            );
        }
    }
}
