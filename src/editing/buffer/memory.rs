use crate::editing::{
    motion::MotionRange,
    text::EditableLine,
    text::{TextLine, TextLines},
    Buffer, CursorPosition, HasId,
};

pub struct MemoryBuffer {
    id: usize,
    content: TextLines,
}

impl MemoryBuffer {
    pub fn new(id: usize) -> MemoryBuffer {
        MemoryBuffer {
            id,
            content: TextLines::default(),
        }
    }
}

impl HasId for MemoryBuffer {
    fn id(&self) -> usize {
        return self.id;
    }
}

impl Buffer for MemoryBuffer {
    fn lines_count(&self) -> usize {
        self.content.height()
    }

    fn append(&mut self, text: TextLines) {
        self.content.extend(text);
    }

    fn get(&self, line_index: usize) -> &TextLine {
        &self.content.lines[line_index]
    }

    fn delete_range(&mut self, range: MotionRange) {
        let CursorPosition {
            line: first_line,
            col: first_col,
        } = range.0;
        let CursorPosition {
            line: last_line,
            col: last_col,
        } = range.1;

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

            if start == 0 && end == line.width() {
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

    fn assert_visual_match(buf: MemoryBuffer, s: &'static str) {
        let actual = buf.to_string();
        let expected = MemoryBuffer {
            id: 0,
            content: s.into(),
        }
        .to_string();

        assert_eq!(actual, expected);
    }

    #[cfg(test)]
    mod delete_range {
        use super::*;

        #[test]
        fn from_line_start() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my land".into());
            buf.delete_range(((0, 0).into(), (0, 4).into()));
            assert_visual_match(buf, " my land");
        }

        #[test]
        fn full_line() {
            let mut buf = MemoryBuffer::new(0);
            buf.append("Take my land".into());
            buf.delete_range(((0, 0).into(), (0, 12).into()));
            assert_visual_match(buf, "");
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
            buf.delete_range(((0, 0).into(), (1, 12).into()));
            assert_visual_match(buf, "");
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
            buf.delete_range(((0, 4).into(), (2, 4).into()));
            assert_visual_match(buf, "Take me where");
        }
    }
}
