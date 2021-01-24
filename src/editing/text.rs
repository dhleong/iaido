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
                let from = start.checked_sub(offset).unwrap_or(0);
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

        #[test]
        fn overlapping_span() {
            let mut part1: TextLine = "Take ".into();
            let mut part2: TextLine = "my love".into();
            let mut span = TextLine::default();
            span.append(&mut part1);
            span.append(&mut part2);
            assert_eq!(span.subs(3, 8).to_string(), "e my ");
        }

        #[test]
        fn multiple_span_overlap() {
            let mut part1: TextLine = "Take ".into();
            let mut part2: TextLine = "my ".into();
            let mut part3: TextLine = "love".into();
            let mut span = TextLine::default();
            span.append(&mut part1);
            span.append(&mut part2);
            span.append(&mut part3);
            assert_eq!(span.subs(3, 10).to_string(), "e my lo");
        }
    }
}
