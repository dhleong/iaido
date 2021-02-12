pub mod commands;
pub mod state;

use crate::editing::{motion::MotionContext, text::EditableLine};

pub struct CompletionContext<'a, T: MotionContext> {
    pub context: &'a T,
    pub text: String,
    pub cursor: usize,
}

impl<T: MotionContext> CompletionContext<'_, T> {
    pub fn word_range(&self) -> (usize, usize) {
        for i in (1..self.cursor).rev() {
            let is_whitespace = self.text[i..i + 1].find(char::is_whitespace);
            if is_whitespace.is_some() {
                return (i + 1, self.cursor);
            }
        }
        return (0, self.cursor);
    }

    pub fn word(&self) -> &str {
        let (start, end) = self.word_range();
        return &self.text[start..end];
    }

    pub fn create_completion(&self, replacement: String) -> Completion {
        let (start, end) = self.word_range();
        Completion {
            start,
            end,
            replacement,
        }
    }
}

impl<'a, T: MotionContext> From<&'a mut T> for CompletionContext<'a, T> {
    fn from(context: &'a mut T) -> Self {
        let bufwin = context.bufwin();
        let line = bufwin.window.cursor.line;
        let text = bufwin.buffer.get(line).to_string();
        let cursor = bufwin.window.cursor.col as usize;
        Self {
            context,
            text,
            cursor,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

pub trait Completer<T: MotionContext> {
    type Iter: Iterator<Item = Completion>;

    fn suggest(&self, context: &T) -> Self::Iter;
}

#[cfg(test)]
mod tests {
    // use crate::editing::motion::{tests::window, MotionContext};
    //
    // use super::*;
    //
    // #[test]
    // fn word_extraction() {
    //     let w = window("take| my love");
    //     let ctx: CompletionContext = &w.into();
    //     assert_eq!(ctx.cursor, 4);
    //     assert_eq!(ctx.word(), "take");
    //     assert_eq!(ctx.word_range(), (0, 4));
    // }
    //
    // #[test]
    // fn word_on_whitespace() {
    //     let ctx: CompletionContext = window("take |my love").into();
    //     assert_eq!(ctx.word(), "");
    //     assert_eq!(ctx.word_range(), (5, 5));
    // }
}
