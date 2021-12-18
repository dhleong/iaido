use crate::editing::{
    motion::{Motion, MotionContext, MotionFlags},
    CursorPosition,
};

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

impl<T> Motion for WordObject<T>
where
    T: Fn(&str) -> bool,
{
    fn flags(&self) -> MotionFlags {
        MotionFlags::EXCLUSIVE
    }

    fn destination<C: MotionContext>(&self, context: &C) -> CursorPosition {
        // if context.buffer().lines_count() == 0 {
        return context.cursor();
        // }
    }
}
