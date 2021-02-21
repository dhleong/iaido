use std::cmp::min;

use crate::editing::CursorPosition;

use super::Motion;

/// Motion that moves the cursor to the start of the current line
pub struct ToLineStartMotion;
impl Motion for ToLineStartMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        context.cursor().start_of_line()
    }
}

/// Motion that moves the cursor to the end of the current line
pub struct ToLineEndMotion;
impl Motion for ToLineEndMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        context.cursor().end_of_line(context.buffer())
    }
}

/// Motion that moves to the beginning of the buffer
pub struct ToStartOfBufferMotion;
impl Motion for ToStartOfBufferMotion {
    fn destination<T: super::MotionContext>(&self, _: &T) -> CursorPosition {
        // easy peasy
        CursorPosition::default()
    }
}

/// Motion that moves to the first col of the last line of the buffer
pub struct ToLastLineOfBufferMotion;
impl Motion for ToLastLineOfBufferMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        // NOTE: technically this should be "the first non-blank char"
        // on the last line...
        CursorPosition {
            line: context.buffer().lines_count().checked_sub(1).unwrap_or(0),
            col: 0,
        }
    }
}

/// Motion that selects the entire current line
pub struct FullLineMotion;
impl Motion for FullLineMotion {
    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        context.cursor().start_of_line()
    }

    fn range<T: super::MotionContext>(&self, context: &T) -> super::MotionRange {
        (
            context.cursor().start_of_line(),
            context.cursor().end_of_line(context.buffer()),
        )
    }
}

/// Motion to move down one line
pub struct DownLineMotion;
impl Motion for DownLineMotion {
    fn is_linewise(&self) -> bool {
        true
    }

    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        let start = context.cursor();
        let buffer = context.buffer();
        let end_index = match buffer.last_index() {
            None => return CursorPosition::default(), // empty buffer
            Some(idx) => idx,
        };

        if start.line == end_index && start == start.end_of_line(context.buffer()) {
            return start;
        }

        let offset_on_line = start.col;
        let next_line_index = min(end_index, start.line + 1);
        let next_line = buffer.get(next_line_index);
        let new_col = min(next_line.width() as u16, offset_on_line);

        CursorPosition {
            line: next_line_index,
            col: new_col,
        }
    }
}

/// Motion to move up one line
pub struct UpLineMotion;
impl Motion for UpLineMotion {
    fn is_linewise(&self) -> bool {
        true
    }

    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        let buffer = context.buffer();
        if buffer.is_empty() {
            return CursorPosition::default();
        }

        let start = context.cursor();

        let offset_on_line = start.col;
        let next_line_index = start.line.checked_sub(1).unwrap_or(0);
        let next_line = buffer.get(next_line_index);
        let new_col = min(next_line.width() as u16, offset_on_line);

        CursorPosition {
            line: next_line_index,
            col: new_col,
        }
    }
}

pub struct ToFirstLineMotion;
impl Motion for ToFirstLineMotion {
    fn is_linewise(&self) -> bool {
        true
    }

    fn destination<T: super::MotionContext>(&self, _: &T) -> CursorPosition {
        CursorPosition { line: 0, col: 0 }
    }
}

pub struct ToLastLineMotion;
impl Motion for ToLastLineMotion {
    fn is_linewise(&self) -> bool {
        true
    }

    fn destination<T: super::MotionContext>(&self, context: &T) -> CursorPosition {
        let buffer = context.buffer();

        CursorPosition {
            line: buffer.lines_count().checked_sub(1).unwrap_or(0),
            col: 0,
        }
    }
}

pub struct LineCrossing<T: Motion> {
    base: T,
}

impl<T: Motion> LineCrossing<T> {
    pub fn new(base: T) -> Self {
        Self { base }
    }
}

impl<T: Motion> Motion for LineCrossing<T> {
    fn destination<C: super::MotionContext>(&self, context: &C) -> CursorPosition {
        let origin = context.cursor();
        let base = self.base.destination(context);
        if origin != base {
            return base;
        }

        if origin.col == 0 && origin.line > 0 {
            CursorPosition {
                line: origin.line - 1,
                col: context.buffer().get_line_width(origin.line - 1).unwrap() as u16,
            }
        } else if origin.col > 0 && origin.line < context.buffer().lines_count() - 1 {
            CursorPosition {
                line: origin.line + 1,
                col: 0,
            }
        } else {
            base
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::editing::motion::tests::window;
    use indoc::indoc;

    mod down_line_motion {
        use super::*;

        #[test]
        fn handles_empty_lines() {
            let mut ctx = window(indoc! {"
                Take my |love

                Take
            "});

            ctx.motion(DownLineMotion);
            ctx.assert_visual_match(indoc! {"
                Take my love
                |
                Take
            "});

            // NOTE: vim would actually end on Tak|e... should we bother?
            ctx.motion(DownLineMotion);
            ctx.assert_visual_match(indoc! {"
                Take my love

                |Take
            "});
        }

        #[test]
        fn hugs_columns() {
            let mut ctx = window(indoc! {"
                Take my |love
                Take
            "});
            ctx.motion(DownLineMotion);
            ctx.assert_visual_match(indoc! {"
                Take my love
                Tak|e
            "});
        }
    }

    mod up_line_motion {
        use super::*;

        #[test]
        fn hugs_columns() {
            let mut ctx = window(indoc! {"
                Take
                Take my |land
            "});
            ctx.motion(UpLineMotion);
            ctx.assert_visual_match(indoc! {"
                Tak|e
                Take my land
            "});
        }
    }
}
