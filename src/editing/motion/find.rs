use crate::editing::CursorPosition;

use super::{char::CharMotion, Motion};

pub struct FindMotion {
    step: CharMotion,
    ch: char,
}

impl FindMotion {
    pub fn forward_to(ch: char) -> Self {
        Self {
            step: CharMotion::Forward(1),
            ch,
        }
    }
}

impl Motion for FindMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        todo!()
    }
}
