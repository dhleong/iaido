use crate::editing::CursorPosition;

use super::{char::CharMotion, Motion, MotionFlags};
use super::{util::search, DirectionalMotion};

pub struct FindMotion {
    step: CharMotion,
    after: Option<CharMotion>,
    ch: char,
}

impl FindMotion {
    pub fn forward_to(ch: char) -> Self {
        Self {
            step: CharMotion::Forward(1),
            after: None,
            ch,
        }
    }

    pub fn forward_until(ch: char) -> Self {
        Self {
            step: CharMotion::Forward(1),
            after: Some(CharMotion::Backward(1)),
            ch,
        }
    }

    pub fn backward_to(ch: char) -> Self {
        Self {
            step: CharMotion::Backward(1),
            after: None,
            ch,
        }
    }

    pub fn backward_until(ch: char) -> Self {
        Self {
            step: CharMotion::Backward(1),
            after: Some(CharMotion::Forward(1)),
            ch,
        }
    }
}

impl Motion for FindMotion {
    fn flags(&self) -> MotionFlags {
        if self.step.is_forward() {
            MotionFlags::NONE
        } else {
            MotionFlags::EXCLUSIVE
        }
    }

    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        let (cursor, found) = search(context, self.step.destination(context), &self.step, |c| {
            c == self.ch
        });
        match (found, &self.after) {
            (false, _) => context.cursor(),
            (true, None) => cursor,
            (true, Some(after)) => after.destination(&context.with_cursor(cursor)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::window;

    use super::*;

    #[test]
    fn forward_to_char() {
        let mut ctx = window("|Take my love");

        ctx.motion(FindMotion::forward_to('l'));
        ctx.assert_visual_match("Take my |love");
    }

    #[test]
    fn forward_to_same_char() {
        let mut ctx = window("Tak|e my love");

        ctx.motion(FindMotion::forward_to('e'));
        ctx.assert_visual_match("Take my lov|e");
    }

    #[test]
    fn forward_to_same_char_adversary() {
        let mut ctx = window("e|eeeee");

        ctx.motion(FindMotion::forward_to('e'));
        ctx.assert_visual_match("ee|eeee");
    }

    #[test]
    fn backward_to_char() {
        let mut ctx = window("Take my |love");

        ctx.motion(FindMotion::backward_to('m'));
        ctx.assert_visual_match("Take |my love");
    }

    #[test]
    fn backward_to_same_char() {
        let mut ctx = window("Take my lov|e");

        ctx.motion(FindMotion::backward_to('e'));
        ctx.assert_visual_match("Tak|e my love");
    }

    #[test]
    fn backward_to_same_char_adversary() {
        let mut ctx = window("eeee|ee");

        ctx.motion(FindMotion::backward_to('e'));
        ctx.assert_visual_match("eee|eee");
    }

    #[test]
    fn forward_until_char() {
        let mut ctx = window("|Take my love");

        ctx.motion(FindMotion::forward_until('l'));
        ctx.assert_visual_match("Take my| love");
    }

    #[test]
    fn forward_until_same_char() {
        let mut ctx = window("Tak|e my love");

        ctx.motion(FindMotion::forward_until('e'));
        ctx.assert_visual_match("Take my lo|ve");
    }

    #[test]
    fn backward_until_char() {
        let mut ctx = window("Take my |love");

        ctx.motion(FindMotion::backward_until('m'));
        ctx.assert_visual_match("Take m|y love");
    }

    #[test]
    fn backward_until_same_char() {
        let mut ctx = window("Take my lov|e");

        ctx.motion(FindMotion::backward_until('e'));
        ctx.assert_visual_match("Take| my love");
    }
}
