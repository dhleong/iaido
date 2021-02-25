use crate::editing::{
    motion::{MotionFlags, MotionRange},
    source::BufferSource,
    text::EditableLine,
    text::{TextLine, TextLines},
    Buffer, CursorPosition, HasId,
};

pub struct MemoryBuffer {
    id: usize,
    content: TextLines,
    pub source: BufferSource,
}

impl MemoryBuffer {
    pub fn new(id: usize) -> MemoryBuffer {
        MemoryBuffer {
            id,
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
    fn source(&self) -> &BufferSource {
        &self.source
    }
    fn set_source(&mut self, source: BufferSource) {
        self.source = source;
    }

    fn lines_count(&self) -> usize {
        self.content.height()
    }

    fn append(&mut self, text: TextLines) {
        self.content.extend(text);
    }

    fn clear(&mut self) {
        self.content.lines.clear();
    }

    fn get(&self, line_index: usize) -> &TextLine {
        &self.content.lines[line_index]
    }

    fn delete_range(&mut self, range: MotionRange) {
        let MotionRange(
            CursorPosition {
                line: first_line,
                col: first_col,
            },
            CursorPosition {
                line: last_line,
                col: last_col,
            },
            flags,
        ) = range;

        let ranges = (first_line..=last_line).map(|line_index| {
            if line_index == first_line && line_index == last_line {
                // single line range:
                (first_col, Some(last_col))
            } else if line_index == first_line {
                (first_col, None)
            } else if line_index == last_line {
                (0, Some(last_col))
            } else {
                (0, None)
            }
        });

        let linewise = first_line < last_line || flags.contains(MotionFlags::LINEWISE);
        let mut consumed_first_line = false;
        let mut consumed_last_line = false;
        let mut line_index = first_line;
        let last_line_index = last_line - first_line;
        for (i, (start, optional_end)) in ranges.enumerate() {
            let line = &self.content.lines[line_index];
            let end = if let Some(v) = optional_end {
                v as usize
            } else {
                line.width()
            };

            if start == 0 && end == line.width() && linewise {
                // delete the whole line
                self.content.lines.remove(line_index);
                if i == 0 {
                    consumed_first_line = true;
                }
                if i == last_line_index {
                    consumed_last_line = true;
                }
            } else {
                // delete within the line
                let mut new_line = line.subs(0, start as usize);
                let mut rest = line.subs(end, line.width());
                new_line.append(&mut rest);

                self.content.lines[line_index] = new_line;
                line_index += 1;
            }
        }

        // if we did a partial delete on both the first and last lines,
        // they need to be spliced together
        if last_line > first_line && !consumed_first_line && !consumed_last_line {
            let to_splice_line = &self.content.lines[first_line + 1];
            let mut to_splice = to_splice_line.subs(0, to_splice_line.width());
            self.content.lines[first_line].append(&mut to_splice);
            self.content.lines.remove(first_line + 1);
        }
    }

    fn insert(&mut self, cursor: CursorPosition, mut text: TextLine) {
        if cursor == (0, 0).into() && self.content.lines.is_empty() {
            self.content.lines.push(text);
            return;
        }

        let original = &self.content.lines[cursor.line];
        let mut before = original.subs(0, cursor.col as usize);
        let mut after = original.subs(cursor.col as usize, original.width());

        let mut new = TextLine::default();
        new.append(&mut before);
        new.append(&mut text);
        new.append(&mut after);

        self.content.lines[cursor.line] = new;
    }
}

impl ToString for MemoryBuffer {
    fn to_string(&self) -> String {
        let mut s = String::default();
        for i in 0..self.lines_count() {
            s.push_str(self.get(i).to_string().as_str());
            s.push_str("\n");
        }
        return s;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indoc::indoc;

    fn assert_visual_match(buf: &MemoryBuffer, s: &'static str) {
        let actual = buf.to_string();
        let expected = MemoryBuffer {
            id: 0,
            content: s.into(),
            source: BufferSource::None,
        }
        .to_string();

        assert_eq!(actual, expected);
    }

    #[cfg(test)]
    mod get_char {
        use super::*;

        #[test]
        fn after_delete_range() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my love land".into());
            buf.delete_range(((0, 7).into(), (0, 12).into()).into());
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
            buf.delete_range(((0, 0).into(), (0, 4).into()).into());
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
            buf.delete_range(((0, 0).into(), (1, 12).into()).into());
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
            buf.delete_range(((0, 4).into(), (2, 4).into()).into());
            assert_visual_match(&buf, "Take me where");
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
}
