pub mod commands;
mod empty;
pub mod file;
mod impl_macro;
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

pub struct CompletionContext {
    pub text: String,
    pub cursor: usize,
    line_index: usize,
}

impl CompletionContext {
    pub fn word_range(&self) -> (usize, usize) {
        for i in (0..self.cursor).rev() {
            let is_end_of_word = self.text[i..i + 1].find(|c| !self.is_keyword(c));
            if is_end_of_word.is_some() {
                return (i + 1, self.cursor);
            }
        }
        return (0, self.cursor);
    }

    pub fn word(&self) -> &str {
        let (start, end) = self.word_range();
        return &self.text[start..end];
    }

    pub fn nth_word(&self, n: usize) -> Option<&str> {
        let search = if let Some(idx) = self.text.find(|c| self.is_keyword(c)) {
            &self.text[idx..]
        } else {
            &self.text[0..]
        };

        // NOTE: if we want eg the 0'th word, we need at least 2 splits;
        // 1 split means actually "don't split anything"
        search.splitn(n + 2, " ").nth(n)
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

    pub fn is_keyword(&self, c: char) -> bool {
        // TODO we might want this to be configurable, like vim;
        // the config could be copied into this struct
        return char::is_alphanumeric(c);
    }
}

impl<'a, T: CompletableContext> From<&'a mut T> for CompletionContext {
    fn from(context: &'a mut T) -> Self {
        let bufwin = context.bufwin();
        let line = bufwin.window.cursor.line;
        let text = bufwin.buffer.get(line).to_string();
        let cursor = bufwin.window.cursor.col;
        Self {
            line_index: line,
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
    pub fn replacement_range(&self) -> (CursorPosition, CursorPosition) {
        (self.start(), self.replacement_end())
    }

    pub fn replacement_end(&self) -> CursorPosition {
        CursorPosition {
            line: self.line_index,
            col: self.start + self.replacement.len(),
        }
    }

    pub fn start(&self) -> CursorPosition {
        CursorPosition {
            line: self.line_index,
            col: self.start,
        }
    }
}

pub type BoxedSuggestions = Box<dyn Iterator<Item = Completion>>;

pub trait Completer {
    fn suggest(
        &self,
        app: Box<&dyn CompletableContext>,
        context: CompletionContext,
    ) -> BoxedSuggestions;
}

#[cfg(test)]
pub mod tests {
    use crate::editing::motion::tests::{window, TestWindow};

    use super::*;

    pub struct StaticCompleter {
        items: Vec<String>,
    }
    impl StaticCompleter {
        pub fn new(items: Vec<String>) -> Self {
            Self { items }
        }
    }
    crate::impl_simple_completer!(StaticCompleter (&self, _app, _context) {
        self.items.clone()
    });

    pub fn complete<T: Completer>(completer: &T, app: &mut TestWindow) -> Vec<String> {
        let ctx: CompletionContext = app.into();
        completer
            .suggest(Box::new(app), ctx)
            .map(|v| v.replacement)
            .collect()
    }

    #[test]
    fn word_extraction() {
        let mut w = window("take| my love");
        let ctx: CompletionContext = (&mut w).into();
        assert_eq!(ctx.cursor, 4);
        assert_eq!(ctx.word(), "take");
        assert_eq!(ctx.word_range(), (0, 4));
    }

    #[test]
    fn word_on_whitespace() {
        let mut w = window("take |my love");
        let ctx: CompletionContext = (&mut w).into();
        assert_eq!(ctx.word(), "");
        assert_eq!(ctx.word_range(), (5, 5));
    }

    #[test]
    fn word_on_symbol() {
        let mut w = window(":|");
        let ctx: CompletionContext = (&mut w).into();
        assert_eq!(ctx.cursor, 1);
        assert_eq!(ctx.word(), "");
        assert_eq!(ctx.word_range(), (1, 1));
    }

    #[test]
    fn empty_arg_to_command() {
        let mut w = window(":e |");
        let ctx: CompletionContext = (&mut w).into();
        assert_eq!(ctx.cursor, 3);
        assert_eq!(ctx.word(), "");
        assert_eq!(ctx.word_range(), (3, 3));
    }

    #[test]
    fn single_letter_arg_to_command() {
        let mut w = window(":e s|");
        let ctx: CompletionContext = (&mut w).into();
        assert_eq!(ctx.cursor, 4);
        assert_eq!(ctx.word(), "s");
        assert_eq!(ctx.word_range(), (3, 4));
    }

    #[test]
    fn nth_word_0_skips_non_keyword() {
        let mut w = window(":e s|");
        let ctx: CompletionContext = (&mut w).into();
        assert_eq!(Some("e"), ctx.nth_word(0));
    }
}
