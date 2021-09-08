use std::{cmp::min, ops::Range};

use tui::text::{Span, Spans, Text};

pub type TextLine = Spans<'static>;
pub type TextLines = Text<'static>;

pub trait EditableLine {
    fn append(&mut self, other: &mut TextLine);
    fn position<P: Fn(char) -> bool>(
        &self,
        search_range: Range<usize>,
        predicate: P,
    ) -> Option<usize>;
    fn subs(&self, start: usize, end: usize) -> Self;
    fn starts_with(&self, s: &str) -> bool;
    fn ends_with(&self, s: &str) -> bool;
    fn to_string(&self) -> String;
}

impl EditableLine for TextLine {
    fn append(&mut self, other: &mut TextLine) {
        self.0.append(&mut other.0);
    }

    fn position<P: Fn(char) -> bool>(
        &self,
        search_range: Range<usize>,
        predicate: P,
    ) -> Option<usize> {
        // accepting RangeBounds and using assert_len would be nice
        // here, but it's unstable...
        let Range { start, end } = search_range;
        let mut index = 0;
        for span in &self.0 {
            if index < start && index + span.content.len() < start {
                continue;
            }

            for (i, ch) in span.content.chars().enumerate() {
                let desti = index + i;
                if desti < start {
                    continue;
                }
                if desti >= end {
                    return None;
                }

                if predicate(ch) {
                    return Some(desti);
                }
            }

            index += span.content.len();
        }

        None
    }

    fn starts_with(&self, s: &str) -> bool {
        let mut index = 0;
        for span in &self.0 {
            for i in 0..span.content.len() {
                let desti = index + i;
                if desti >= s.len() {
                    return true;
                }

                if span.content[i..i + 1] != s[desti..desti + 1] {
                    return false;
                }
            }

            index += span.content.len();
        }

        return true;
    }

    fn ends_with(&self, s: &str) -> bool {
        if self.0.is_empty() {
            return false;
        }

        let mut index = s.len();
        for span in self.0.iter().rev() {
            if span.content.is_empty() {
                continue;
            }

            let span_width = span.content.len();
            for i in (0..span_width).rev() {
                if i > index {
                    return true;
                }

                let desti = index + i - span_width;
                if span.content[i..i + 1] != s[desti..desti + 1] {
                    return false;
                }
            }

            index -= span.content.len();
        }

        return true;
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

    #[cfg(test)]
    mod starts_with {
        use super::*;

        #[test]
        fn single_span() {
            let s: String = "serenity".into();
            let part1: TextLine = s.clone().into();
            assert_eq!(part1.starts_with(&s), true);
        }

        #[test]
        fn multi_span_equal_len() {
            let expected: String = "serenity".into();
            let mut line: TextLine = "sere".into();
            let mut part2: TextLine = "nity".into();
            line.append(&mut part2);
            assert_eq!(line.starts_with(&expected), true);
        }

        #[test]
        fn multi_span_shorter() {
            let expected: String = "se".into();
            let mut line: TextLine = "sere".into();
            let mut part2: TextLine = "nity".into();
            line.append(&mut part2);
            assert_eq!(line.starts_with(&expected), true);
        }
    }

    #[cfg(test)]
    mod ends_with {
        use super::*;

        #[test]
        fn single_span() {
            let s = "serenity";
            let part1: TextLine = s.clone().into();
            assert_eq!(part1.ends_with(&s), true);
        }

        #[test]
        fn multi_span_equal_len() {
            let expected = "serenity";
            let mut line: TextLine = "sere".into();
            let mut part2: TextLine = "nity".into();
            line.append(&mut part2);
            assert_eq!(line.ends_with(&expected), true);
        }

        #[test]
        fn multi_span_shorter() {
            let mut line: TextLine = "sere".into();
            let mut part2: TextLine = "nity".into();
            line.append(&mut part2);
            assert_eq!(line.ends_with("ty"), true);
            assert_eq!(line.ends_with("enity"), true);
        }

        #[test]
        fn multi_span_longer() {
            let mut line: TextLine = "sere".into();
            let mut part2: TextLine = "nity".into();
            line.append(&mut part2);
            assert_eq!(line.ends_with("trinity"), false);
            assert_eq!(line.ends_with("serenitude"), false);
        }
    }
}
