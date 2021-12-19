use std::cmp::{max, min};

use super::{DirectionalMotion, Motion, MotionContext, MotionFlags, MotionRange};
use crate::editing::CursorPosition;

pub struct RepeatedMotion<T: Motion> {
    motion: T,
    count: u32,
}

impl<T: Motion> RepeatedMotion<T> {
    pub fn with_count(motion: T, count: u32) -> Self {
        Self { motion, count }
    }
}

impl<T: Motion + DirectionalMotion> DirectionalMotion for RepeatedMotion<T> {
    fn is_forward(&self) -> bool {
        self.motion.is_forward()
    }
}

impl<T: Motion> Motion for RepeatedMotion<T> {
    fn flags(&self) -> MotionFlags {
        self.motion.flags()
    }

    fn destination<C: MotionContext>(&self, context: &C) -> CursorPosition {
        let mut cursor = context.cursor();
        for _ in 0..self.count {
            let with_cursor = context.with_cursor(cursor);
            cursor = self.motion.destination(&with_cursor);
        }
        return cursor;
    }

    fn range<C: MotionContext>(&self, context: &C) -> MotionRange {
        let mut start = context.cursor();
        let mut end = context.cursor();
        let mut flags: MotionFlags = MotionFlags::NONE;
        let destination = self.destination(context);
        let forward = destination >= end;

        for _ in 0..self.count {
            let with_cursor = context.with_cursor(if forward { end } else { start });
            let MotionRange(new_start, new_end, new_flags) = self.motion.range(&with_cursor);
            start = min(start, new_start);
            end = max(end, new_end);
            flags = new_flags;
        }
        MotionRange(start, end, flags)
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;
    use crate::editing::motion::word::{is_small_word_boundary, WordMotion};
    use indoc::indoc;

    use super::*;

    #[test]
    fn repeat_forward_motion() {
        let mut ctx = window(indoc! {"
            |'Take' my land
        "});
        ctx.motion(RepeatedMotion::with_count(
            WordMotion::forward_until(is_small_word_boundary),
            3,
        ));
        ctx.assert_visual_match(indoc! {"
            'Take' |my land
        "});
    }

    #[test]
    fn repeat_backward_motion() {
        let mut ctx = window(indoc! {"
            'Take' my |land
        "});
        ctx.motion(RepeatedMotion::with_count(
            WordMotion::backward_until(is_small_word_boundary),
            3,
        ));
        ctx.assert_visual_match(indoc! {"
            '|Take' my land
        "});
    }
}
