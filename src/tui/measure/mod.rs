use std::cmp::max;

use tui::{
    buffer, buffer::Buffer, layout::Alignment, layout::Rect, text::Text, widgets::Paragraph,
    widgets::Widget, widgets::Wrap,
};

use crate::editing::text::{TextLine, TextLines};

pub trait Measurable {
    fn measure_height(&self, width: u16) -> u16;
}

impl Measurable for TextLine {
    fn measure_height(&self, width: u16) -> u16 {
        let text = Text::from(vec![self.clone()]);
        let p = Paragraph::new(text)
            .wrap(Wrap { trim: true }) // NOTE: may become a pref?
            .alignment(Alignment::Left);

        // TODO: this is HACKS; just do the wrapping, please

        // NOTE: in order to avoid wildly excessive allocations,
        // we use some simple heuristics to guess how much space
        // we might need, then double that guess
        let perfect_height = max(1, self.width() / (width as usize));
        let available_height = (perfect_height * 2) as u16;

        let size = Rect {
            x: 0,
            y: 0,
            width,
            height: available_height,
        };
        let mut empty_cell = buffer::Cell::default();
        empty_cell.set_symbol("\x00");
        let mut buffer = Buffer::filled(size, &empty_cell);
        p.render(size, &mut buffer);

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
}
