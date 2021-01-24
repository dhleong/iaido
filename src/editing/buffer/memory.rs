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

        let line = &self.content.lines[first_line];
        if first_line == last_line && first_col == 0 && last_col as usize >= line.width() - 1 {
            // delete the whole line
            self.content.lines.remove(first_line);
            return;
        }

        if first_line == last_line {
            // delete within a single line
            let mut new_line = line.subs(0, first_col as usize);
            let mut rest = line.subs(last_col as usize, line.width());
            new_line.append(&mut rest);

            self.content.lines[first_line] = new_line;
            return;
        }

        // TODO
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_visual_match<T: Buffer>(buf: T, s: &'static str) {
        let actual_lines: Vec<&TextLine> = (0..buf.lines_count()).map(|i| buf.get(i)).collect();
        let expected_lines: TextLines = s.into();

        // special case:
        if expected_lines.lines.is_empty() {
            let mut s = String::default();
            for i in 0..buf.lines_count() {
                s.push_str(actual_lines[i].to_string().as_str());
                s.push_str("\n");
            }
            assert_eq!(s, "")
        }

        for i in 0..buf.lines_count() {
            let actual = actual_lines[i].to_string();
            let expected = expected_lines.lines[i].to_string();
            assert_eq!(actual, expected);
        }
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
    }
}
