pub mod commands;
pub mod state;

use crate::{
    app::bufwin::BufWin,
    editing::{text::EditableLine, CursorPosition},
};

use super::commands::registry::CommandRegistry;

/// A mockable view onto App State
pub trait CompletableContext {
    fn bufwin(&mut self) -> BufWin;
    fn commands(&self) -> &CommandRegistry;
}

pub struct CompletionContext<'a, T: CompletableContext> {
    pub context: &'a T,
    pub text: String,
    pub cursor: usize,
    line_index: usize,
}

impl<T: CompletableContext> CompletionContext<'_, T> {
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
            line_index: self.line_index,
            start,
            end,
            replacement,
        }
    }
}

impl<'a, T: CompletableContext> From<&'a mut T> for CompletionContext<'a, T> {
    fn from(context: &'a mut T) -> Self {
        let bufwin = context.bufwin();
        let line = bufwin.window.cursor.line;
        let text = bufwin.buffer.get(line).to_string();
        let cursor = bufwin.window.cursor.col as usize;
        Self {
            line_index: line,
            context,
            text,
            cursor,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Completion {
    pub line_index: usize,
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

impl Completion {
    pub fn range(&self) -> (CursorPosition, CursorPosition) {
        (
            self.start(),
            CursorPosition {
                line: self.line_index,
                col: self.end as u16,
            },
        )
    }

    pub fn start(&self) -> CursorPosition {
        CursorPosition {
            line: self.line_index,
            col: self.start as u16,
        }
    }
}

pub trait Completer<T: CompletableContext> {
    type Iter: Iterator<Item = Completion>;

    fn suggest(&self, context: &CompletionContext<T>) -> Self::Iter;
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::tests::{window, TestWindow};

    use super::*;

    #[test]
    fn word_extraction() {
        let mut w = window("take| my love");
        let ctx: CompletionContext<TestWindow> = (&mut w).into();
        assert_eq!(ctx.cursor, 4);
        assert_eq!(ctx.word(), "take");
        assert_eq!(ctx.word_range(), (0, 4));
    }

    #[test]
    fn word_on_whitespace() {
        let mut w = window("take |my love");
        let ctx: CompletionContext<TestWindow> = (&mut w).into();
        assert_eq!(ctx.word(), "");
        assert_eq!(ctx.word_range(), (5, 5));
    }
}
