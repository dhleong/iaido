use std::cmp::max;

use tui::{
    buffer::Buffer,
    layout::Alignment,
    layout::Rect,
    text::{Spans, Text},
    widgets::Paragraph,
    widgets::Widget,
    widgets::Wrap,
};

use crate::editing::text::{TextLine, TextLines};

pub fn render_into(line: &Spans, width: u16, mut buffer: &mut Buffer) -> Rect {
    // TODO: this whole thing is HACKS; just do the wrapping, please

    // NOTE: in order to avoid wildly excessive allocations,
    // we use some simple heuristics to guess how much space
    // we might need, then double that guess
    let perfect_height = max(1, line.width() / (width as usize));
    let available_height = (perfect_height * 2) as u16;

    let size = Rect {
        x: 0,
        y: 0,
        width,
        height: available_height,
    };
    let expected_len = size.area();
    if expected_len as usize > buffer.content.len() {
        // make space
        buffer.resize(size);
    }

    // clear:
    for cell in &mut buffer.content {
        cell.set_symbol("\x00");
    }

    // TODO: this is HACKS; just do the wrapping, please
    let mut to_wrap = line.clone();
    if !to_wrap.0.is_empty() {
        let last_index = to_wrap.0.len() - 1;
        let old = &to_wrap.0[last_index].content;

        // replace trailing whitespace with nbsp so the wrapping
        // doesn't eat it
        let last_non_space = old.rfind(|c| c != ' ');
        let first_whitespace = last_non_space
            .map(|off| {
                old.char_indices()
                    .map(|(i, _)| i)
                    .skip_while(|i| *i <= off)
                    .next()
                    .unwrap_or(old.len() + 1)
            })
            .unwrap_or(0);
        if first_whitespace < old.len() {
            let spaces: String = vec!['\u{00A0}'; old.len() - first_whitespace]
                .into_iter()
                .collect();
            let mut new_content = old[0..first_whitespace].to_string();
            new_content.push_str(&spaces.into_boxed_str());
            to_wrap.0[last_index].content = new_content.into();
        }
    }
    let text = Text::from(vec![to_wrap]);
    let p = Paragraph::new(text)
        .wrap(Wrap { trim: false }) // NOTE: may become a pref?
        .alignment(Alignment::Left);
    p.render(size, &mut buffer);
    return size;
}

pub trait Measurable {
    fn measure_height(&self, width: u16) -> u16;
}

impl Measurable for TextLine {
    fn measure_height(&self, width: u16) -> u16 {
        if width <= 0 {
            return 0;
        }

        let mut buffer = Buffer::default();
        let size = render_into(self, width, &mut buffer);

        // NOTE: it's always at least 1-height...
        for i in 1..size.height {
            if buffer.get(0, i).symbol == "\x00" {
                return i;
            }
        }

        return size.height;
    }
}

impl Measurable for TextLines {
    fn measure_height(&self, width: u16) -> u16 {
        self.lines
            .iter()
            .map(|line| line.measure_height(width))
            .sum()
    }
}

impl Measurable for Vec<&TextLine> {
    fn measure_height(&self, width: u16) -> u16 {
        self.iter().map(|line| line.measure_height(width)).sum()
    }
}

impl Measurable for dyn crate::editing::buffer::Buffer {
    fn measure_height(&self, width: u16) -> u16 {
        let lines: Vec<&TextLine> = (0..self.lines_count()).map(|i| self.get(i)).collect();
        lines.measure_height(width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_exact() {
        let text = TextLine::from("love");
        assert_eq!(text.measure_height(4), 1, "Tight wrap");
    }

    #[test]
    fn measure_short() {
        let text = TextLine::from("my");
        assert_eq!(text.measure_height(4), 1, "Tight wrap");
    }

    #[test]
    fn measure_wrapping() {
        let text = TextLine::from("Take my love, take my land");

        assert_eq!(text.measure_height(text.width() as u16), 1, "Exact width");
        assert_eq!(text.measure_height(4), 7, "Tight wrap");
    }

    #[test]
    fn measure_wrapping_lines() {
        let text = TextLines::from(vec![
            TextLine::from("Take my love,"),
            TextLine::from("take my land"),
        ]);
        assert_eq!(text.measure_height(4), 7, "Tight wrap");
    }

    #[test]
    fn measure_python_syntax_error() {
        let text = TextLines::from(vec![TextLine::from("foo"), TextLine::from("↑↑↑")]);
        assert_eq!(text.measure_height(20), 2, "Syntax error wrap");
    }
}
