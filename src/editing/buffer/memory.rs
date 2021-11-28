use crate::editing::{
    motion::MotionRange,
    source::BufferSource,
    text::EditableLine,
    text::{TextLine, TextLines},
    Buffer, CursorPosition, HasId,
};

use super::{util::motion_to_line_ranges, BufferConfig, CopiedRange};

pub struct MemoryBuffer {
    id: usize,
    content: TextLines,
    config: BufferConfig,
    pub source: BufferSource,
}

impl MemoryBuffer {
    pub fn new(id: usize) -> MemoryBuffer {
        MemoryBuffer {
            id,
            config: BufferConfig::default(),
            content: TextLines::default(),
            source: BufferSource::None,
        }
    }
}

impl HasId for MemoryBuffer {
    fn id(&self) -> usize {
        return self.id;
    }
}

impl Buffer for MemoryBuffer {
    fn config(&self) -> &BufferConfig {
        &self.config
    }

    fn config_mut(&mut self) -> &mut BufferConfig {
        &mut self.config
    }

    fn source(&self) -> &BufferSource {
        &self.source
    }
    fn set_source(&mut self, source: BufferSource) {
        self.source = source;
    }

    fn lines_count(&self) -> usize {
        self.content.height()
    }

    fn clear(&mut self) {
        self.content.lines.clear();
    }

    fn get(&self, line_index: usize) -> &TextLine {
        &self.content.lines[line_index]
    }

    fn get_range(&self, range: MotionRange) -> CopiedRange {
        let (first_line, last_line) = range.lines();
        let ranges = motion_to_line_ranges(range);

        let mut copy = CopiedRange::default();
        let mut line_index = first_line;
        let last_line_index = last_line - first_line;
        for (i, range) in ranges.enumerate() {
            if range.is_whole_line(line_index, self) {
                // copy the whole line
                copy.text
                    .lines
                    .push(self.content.lines.get(line_index).unwrap().clone());
                if i == 0 {
                    copy.leading_newline = true;
                }
                if i == last_line_index {
                    copy.trailing_newline = true;
                }
            } else {
                // yank within the line
                let (start, end) = range.resolve(line_index, self);
                let line = &self.content.lines[line_index];
                copy.text.lines.push(line.subs(start, end));
            }
            line_index += 1;
        }

        return copy;
    }

    fn delete_range(&mut self, range: MotionRange) -> CopiedRange {
        let (first_line, last_line) = range.lines();
        let ranges = motion_to_line_ranges(range);

        let mut copy = CopiedRange::default();
        let mut line_index = first_line;
        let last_line_index = last_line - first_line;
        for (i, range) in ranges.enumerate() {
            if range.is_whole_line(line_index, self) {
                // delete the whole line
                copy.text.lines.push(self.content.lines.remove(line_index));
                if i == 0 {
                    copy.leading_newline = true;
                }
                if i == last_line_index {
                    copy.trailing_newline = true;
                }
            } else {
                // delete within the line
                let (start, end) = range.resolve(line_index, self);
                let line = &self.content.lines[line_index];
                copy.text.lines.push(line.subs(start, end));

                let mut new_line = line.subs(0, start);
                let mut rest = line.subs(end, line.width());
                new_line.append(&mut rest);

                self.content.lines[line_index] = new_line;
                line_index += 1;
            }
        }

        // if we did a partial delete on both the first and last lines,
        // they need to be spliced together
        if last_line > first_line && copy.is_partial() {
            let to_splice_line = &self.content.lines[first_line + 1];
            let mut to_splice = to_splice_line.subs(0, to_splice_line.width());
            self.content.lines[first_line].append(&mut to_splice);
            self.content.lines.remove(first_line + 1);
        }

        return copy;
    }

    fn insert(&mut self, cursor: CursorPosition, mut text: TextLine) {
        if cursor == (0, 0).into() && self.content.lines.is_empty() {
            self.content.lines.push(text);
            return;
        }

        let original = &self.content.lines[cursor.line];
        let mut before = original.subs(0, cursor.col);
        let mut after = original.subs(cursor.col, original.width());

        let mut new = TextLine::default();
        new.append(&mut before);
        new.append(&mut text);
        new.append(&mut after);

        self.content.lines[cursor.line] = new;
    }

    fn insert_lines(&mut self, line_index: usize, text: TextLines) {
        if line_index == self.lines_count() {
            self.content.extend(text);
        } else {
            self.content
                .lines
                .splice(line_index..line_index, text.lines);
        }
    }

    fn insert_range(&mut self, cursor: CursorPosition, mut copied: CopiedRange) {
        // NOTE: we insert in reverse order, progressively "popping" off of the end of the vector,
        // to avoid having to copy. `copied` is probably already a copy, and it's cleaner (I
        // think?) to just assume the caller has made a copy if necessary than to always have to
        // make one defensively
        let mut start = 0;
        let mut end = copied.text.lines.len();
        let multiline = end > 1;

        if copied.is_partial() && !multiline {
            // simple case: single, in-line range
            self.insert(cursor.into(), copied.text.lines.remove(0).into());
            return;
        }

        if !copied.trailing_newline {
            // We have to de-splice this line
            let end_of_line = self.get_line_width(cursor.line).unwrap_or(cursor.col);
            let range: MotionRange = (cursor, cursor.with_col(end_of_line)).into();
            let after_last_line = self.delete_range(range).text;
            self.insert_lines(cursor.line + 1, after_last_line);
            self.insert(
                (cursor.line + 1, 0).into(),
                copied.text.lines.remove(end - 1).into(),
            );

            end -= 1;
        }

        if !copied.leading_newline {
            start += 1;
        }

        if end > 0 {
            let lines: Vec<TextLine> = copied.text.lines.splice(start..end, vec![]).collect();
            self.insert_lines(cursor.line + start, TextLines::from(lines));
        }

        if !copied.leading_newline {
            self.insert(cursor, copied.text.lines.remove(0));
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use indoc::indoc;

    pub trait TestableBuffer {
        fn assert_visual_match(&self, s: &'static str);
    }

    impl TestableBuffer for String {
        fn assert_visual_match(&self, s: &'static str) {
            let expected = MemoryBuffer {
                id: 0,
                config: BufferConfig::default(),
                content: s.into(),
                source: BufferSource::None,
            }
            .get_contents();

            assert_eq!(self.clone(), expected);
        }
    }

    impl<T: Buffer> TestableBuffer for T {
        fn assert_visual_match(&self, s: &'static str) {
            let actual = self.get_contents();
            actual.assert_visual_match(s);
        }
    }

    impl TestableBuffer for Box<dyn Buffer> {
        fn assert_visual_match(&self, s: &'static str) {
            let actual = self.get_contents();
            actual.assert_visual_match(s);
        }
    }

    pub fn assert_visual_match<T: Buffer>(buf: &T, s: &'static str) {
        buf.assert_visual_match(s);
    }

    #[cfg(test)]
    mod get_char {
        use super::*;

        #[test]
        fn after_delete_range() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my love land".into());
            buf.delete_range(((0, 7), (0, 12)).into());
            assert_visual_match(&buf, "Take my land");
            assert_eq!(Some(" "), buf.get_char((0, 7).into()));
            assert_eq!(Some("l"), buf.get_char((0, 8).into()));
        }
    }

    #[cfg(test)]
    mod delete_range {
        use super::*;
        use crate::editing::motion::MotionFlags;

        #[test]
        fn from_line_start() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my land".into());
            buf.delete_range(((0, 0), (0, 4)).into());
            assert_visual_match(&buf, " my land");
        }

        #[test]
        fn full_line() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my land".into());
            buf.delete_range(MotionRange(
                (0, 0).into(),
                (0, 12).into(),
                MotionFlags::LINEWISE,
            ));
            assert_visual_match(&buf, "");
        }

        #[test]
        fn all_lines() {
            let mut buf = MemoryBuffer::new(0);
            buf.append(
                indoc! {"
                    Take my love
                    Take my land
                "}
                .into(),
            );
            buf.delete_range(((0, 0), (1, 12)).into());
            assert_visual_match(&buf, "");
        }

        #[test]
        fn across_lines() {
            let mut buf = MemoryBuffer::new(0);
            buf.append(
                indoc! {"
                    Take my love
                    Take my land
                    Take me where
                "}
                .into(),
            );
            buf.delete_range(((0, 4), (2, 4)).into());
            assert_visual_match(&buf, "Take me where");
        }
    }

    #[cfg(test)]
    mod get_range {
        use super::*;
        use crate::editing::motion::MotionFlags;

        #[test]
        fn from_line_start() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my land".into());
            let contents = buf.get_range(((0, 0), (0, 4)).into()).get_contents();
            assert_eq!(contents, "Take");
        }

        #[test]
        fn full_line() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my land".into());
            let contents = buf
                .get_range(MotionRange(
                    (0, 0).into(),
                    (0, 12).into(),
                    MotionFlags::LINEWISE,
                ))
                .get_contents();
            assert_eq!(contents, "\nTake my land\n");
        }

        #[test]
        fn all_lines() {
            let mut buf = MemoryBuffer::new(0);
            buf.append(
                indoc! {"
                    Take my love
                    Take my land
                "}
                .into(),
            );
            let contents = buf.get_range(((0, 0), (1, 12)).into()).get_contents();
            assert_eq!(contents, "\nTake my love\nTake my land\n");
        }

        #[test]
        fn across_lines() {
            let mut buf = MemoryBuffer::new(0);
            buf.append(
                indoc! {"
                    Take my love
                    Take my land
                    Take me where
                "}
                .into(),
            );
            let contents = buf.get_range(((0, 4), (2, 4)).into()).get_contents();
            assert_eq!(contents, " my love\nTake my land\nTake");
        }
    }

    #[cfg(test)]
    mod insert {
        use super::*;

        #[test]
        fn at_beginning() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("my love".into());
            buf.insert((0, 0).into(), "Take ".into());
            assert_visual_match(&buf, "Take my love");
        }

        #[test]
        fn in_middle() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take love".into());
            buf.insert((0, 4).into(), " my".into());
            assert_visual_match(&buf, "Take my love");
        }

        #[test]
        fn at_end() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my".into());
            buf.insert((0, 7).into(), " love".into());
            assert_visual_match(&buf, "Take my love");
        }

        #[test]
        fn into_empty() {
            let mut buf = MemoryBuffer::new(0);
            buf.insert((0, 0).into(), "serenity".into());
            assert_visual_match(&buf, "serenity");
        }
    }

    #[cfg(test)]
    mod append_value {
        use crate::connection::ReadValue;

        use super::*;

        #[test]
        fn into_empty() {
            let mut buf = MemoryBuffer::new(0);
            buf.append_value(ReadValue::Text("serenity".into()));
            assert_visual_match(&buf, "serenity");
        }
    }

    #[cfg(test)]
    mod insert_range {
        use super::*;

        #[test]
        fn neither_leading_nor_trailing() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my stand".into());

            buf.insert_range(
                (0, 8).into(),
                CopiedRange {
                    text: TextLines::raw("love\nTake my land\nTake me where I cannot "),
                    leading_newline: false,
                    trailing_newline: false,
                },
            );

            assert_visual_match(
                &buf,
                indoc! {"
                    Take my love
                    Take my land
                    Take me where I cannot stand
                "},
            );
        }

        #[test]
        fn lines_into_empty() {
            let mut buf = MemoryBuffer::new(0);
            buf.insert_range(
                (0, 0).into(),
                CopiedRange {
                    text: TextLines::raw("Take my love\nTake my land"),
                    leading_newline: true,
                    trailing_newline: true,
                },
            );

            assert_visual_match(
                &buf,
                indoc! {"
                    Take my love
                    Take my land
                "},
            );
        }

        #[test]
        fn append() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my love".into());

            buf.insert_range(
                (1, 0).into(),
                CopiedRange {
                    text: TextLines::raw("Take my land"),
                    leading_newline: true,
                    trailing_newline: true,
                },
            );

            assert_visual_match(
                &buf,
                indoc! {"
                    Take my love
                    Take my land
                "},
            );
        }
    }
}
