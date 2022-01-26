use crate::editing::CursorPosition;

use super::Motion;

pub fn search<C: super::MotionContext, M: Motion, P: Fn(char) -> bool>(
    context: &C,
    start: CursorPosition,
    step: &M,
    pred: P,
) -> (CursorPosition, bool) {
    let mut cursor = start;
    let mut found = false;

    loop {
        if let Some(ch) = context.buffer().get_char(cursor) {
            if pred(ch) {
                found = true;
                break;
            } else {
                let next = step.destination(&context.with_cursor(cursor));
                if next == cursor {
                    // our step didn't move; we can't go further
                    break;
                }
                cursor = next;
            }
        } else {
            // can't go further
            return (cursor, false);
        }
    }

    (cursor, found)
}

pub fn find<C: super::MotionContext, M: Motion, P: Fn(char) -> bool>(
    context: &C,
    start: CursorPosition,
    step: &M,
    pred: P,
) -> CursorPosition {
    let (cursor, _) = search(context, start, step, pred);
    return cursor;
}
