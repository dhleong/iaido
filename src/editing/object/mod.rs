use super::motion::{Motion, MotionContext, MotionFlags, MotionRange};

pub mod pair;
mod util;
pub mod word;

pub trait TextObject {
    fn object_range<C: MotionContext>(&self, context: &C) -> MotionRange;
}

impl<T: TextObject> Motion for T {
    fn destination<C: MotionContext>(&self, context: &C) -> super::CursorPosition {
        self.object_range(context).1
    }

    fn range<C: MotionContext>(&self, context: &C) -> MotionRange {
        let MotionRange(start, mut end, flags) = self.object_range(context);
        let inclusive = !flags.contains(MotionFlags::EXCLUSIVE);

        if inclusive {
            end.col += 1;
        }

        MotionRange(start, end, flags)
    }
}
