use crate::editing::{
    motion::{char::CharMotion, Motion, MotionContext},
    CursorPosition,
};

fn is_whitespace(end: Option<char>) -> bool {
    if let Some(s) = end {
        s.is_whitespace()
    } else {
        false
    }
}

pub fn follow_whitespace<C: MotionContext>(
    context: &C,
    start: CursorPosition,
    step: CharMotion,
) -> CursorPosition {
    let buffer = context.buffer();
    let mut end = start;

    let line_width = match buffer.get_line_width(end.line) {
        Some(width) => width,
        _ => return end,
    };

    loop {
        let new_end = step.destination(&context.with_cursor(end));
        if new_end.col >= line_width || new_end == end || !is_whitespace(buffer.get_char(new_end)) {
            break;
        }
        end = new_end;
    }

    end
}
