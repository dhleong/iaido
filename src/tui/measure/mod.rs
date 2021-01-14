use tui::{
    buffer, buffer::Buffer, layout::Alignment, layout::Rect, text::Text, widgets::Paragraph,
    widgets::Widget, widgets::Wrap,
};

use crate::editing::text::TextLine;

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
        let perfect_height = self.width() / (width as usize);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_wrapping() {
        let text = TextLine::from("Take my love, take my land");

        assert_eq!(text.measure_height(text.width() as u16), 1, "Exact width");
        assert_eq!(text.measure_height(4), 7, "Tight wrap");
    }
}
