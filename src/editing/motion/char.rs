use std::cmp::min;

use crate::editing::CursorPosition;

use super::Motion;

/// Character-wise column motion
pub enum CharMotion {
    Forward(u16),
    Backward(u16),
}

impl Motion for CharMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        let from = context.cursor();
        match self {
            &CharMotion::Forward(step) => {
                let end = context
                    .buffer()
                    .get_line_width(from.line)
                    .expect("Invalid line");
                from.with_col(min(end, from.col.checked_add(step as usize).unwrap_or(end)))
            }

            &CharMotion::Backward(step) => {
                from.with_col(from.col.checked_sub(step as usize).unwrap_or(0))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;

    use super::*;

    #[test]
    fn forward_is_clamped() {
        let mut ctx = window("Take my lov|e");

        ctx.motion(CharMotion::Forward(1));
        ctx.assert_visual_match("Take my lov|e");
    }

    #[test]
    fn forward_is_clamped_after_end() {
        let ctx = window("Take my love|");
        let origin = ctx.window.cursor;

        let destination = CharMotion::Forward(1).destination(&ctx);
        assert_eq!(origin, destination);
    }

    #[test]
    fn forward_can_pass_end_in_insert() {
        let mut ctx = window("Take my lov|e");

        ctx.set_inserting(true);
        ctx.motion(CharMotion::Forward(1));
        ctx.assert_visual_match("Take my love|");
    }

    #[test]
    fn backward_is_clamped() {
        let mut ctx = window("|Take my love");

        ctx.motion(CharMotion::Backward(1));
        ctx.assert_visual_match("|Take my love");
    }
}
