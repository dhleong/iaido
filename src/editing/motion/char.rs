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
            &CharMotion::Forward(step) => from.with_col(
                from.col
                    .checked_add(step)
                    .unwrap_or(from.end_of_line(context.buffer()).col),
            ),

            &CharMotion::Backward(step) => from.with_col(from.col.checked_sub(step).unwrap_or(0)),
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
