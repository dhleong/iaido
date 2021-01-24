use std::cmp::min;

use tui::text::{Span, Spans, Text};

pub type TextLine = Spans<'static>;
pub type TextLines = Text<'static>;

pub trait EditableLine {
    fn append(&mut self, other: &mut TextLine);
    fn subs(&self, start: usize, end: usize) -> Self;
    fn to_string(&self) -> String;
}

impl EditableLine for TextLine {
    fn append(&mut self, other: &mut TextLine) {
        self.0.append(&mut other.0);
    }

    fn subs(&self, start: usize, end: usize) -> TextLine {
        let mut spans: Vec<Span<'static>> = Vec::new();

        let mut offset = 0;
        for span in &self.0 {
            let width = span.width();
            if start < offset + width && end > offset {
                let from = start - offset;
                let to = min(end - offset, width);
                let content = &span.content[from..to];
                let s = String::from(content);
                spans.push(Span::styled(s, span.style));
            }
            offset += span.width();

            if offset >= end {
                // early shortcut
                break;
            }
        }

        TextLine::from(spans)
    }

    fn to_string(&self) -> String {
        let mut s = String::default();

        for chunk in &self.0 {
            s.push_str(&chunk.content);
        }

        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(test)]
    mod subs {
        use super::*;

        #[test]
        fn full_subsequence() {
            let span: TextLine = "Take my love".into();
            assert_eq!(span.subs(0, 12).to_string(), "Take my love");
        }

        #[test]
        fn fully_subsumed() {
            let span: TextLine = "Take my love".into();
            assert_eq!(span.subs(5, 7).to_string(), "my");
        }

        // TODO test crossing span boundaries, etc.
    }
}
