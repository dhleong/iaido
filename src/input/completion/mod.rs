use crate::{app::bufwin::BufWin, editing::text::EditableLine};

pub struct CompletionContext {
    pub text: String,
    pub cursor: usize,
}

impl CompletionContext {
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

impl From<BufWin<'_>> for CompletionContext {
    fn from(bufwin: BufWin) -> Self {
        let line = bufwin.window.cursor.line;
        Self {
            text: bufwin.buffer.get(line).to_string(),
            cursor: bufwin.window.cursor.col as usize,
        }
    }
}

pub struct Completion {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}

pub trait Completer {
    type Iter: Iterator<Item = Completion>;

    fn suggest(&mut self, context: &CompletionContext) -> Self::Iter;
}

#[cfg(test)]
mod tests {
    use crate::editing::motion::{tests::window, MotionContext};

    use super::*;

    #[test]
    fn word_extraction() {
        let ctx: CompletionContext = window("take| my love").bufwin().into();
        assert_eq!(ctx.cursor, 4);
        assert_eq!(ctx.word(), "take");
        assert_eq!(ctx.word_range(), (0, 4));
    }

    #[test]
    fn word_on_whitespace() {
        let ctx: CompletionContext = window("take |my love").bufwin().into();
        assert_eq!(ctx.word(), "");
        assert_eq!(ctx.word_range(), (5, 5));
    }
}
