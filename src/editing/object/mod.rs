use super::motion::{Motion, MotionContext, MotionFlags, MotionRange};

pub mod word;

pub trait TextObject {
    fn object_range<C: MotionContext>(&self, context: &C) -> MotionRange;
}

impl<T: TextObject> Motion for T {
    fn destination<C: MotionContext>(&self, context: &C) -> super::CursorPosition {
        self.object_range(context).1
    }

    fn range<C: MotionContext>(&self, context: &C) -> MotionRange {
        let inclusive = !self.flags().contains(MotionFlags::EXCLUSIVE);
        let MotionRange(start, mut end, flags) = self.object_range(context);

        if inclusive {
            end.col += 1;
        }

        MotionRange(start, end, flags)
    }
}
