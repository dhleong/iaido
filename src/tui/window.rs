use std::cmp;

use tui::{
    layout::Alignment, layout::Rect, text, widgets::Paragraph, widgets::Widget, widgets::Wrap,
};

use super::Renderable;
use crate::editing::window::Window;

impl Renderable for Window {
    fn render(&self, app: &crate::App, display: &mut super::Display, area: Rect) {
        let buf = match app.buffers.by_id(self.buffer) {
            None => return,
            Some(buf) => buf,
        };

        let count = buf.lines_count();
        let end = count - cmp::min(count, self.scrolled_lines as usize);
        let start = end - cmp::min(end, self.size.h as usize);

        let lines: Vec<text::Spans> = (start..end).map(|i| buf.get(i).clone()).collect();
        let candidate_text = text::Text::from(lines);

        // TODO borders?
        let paragraph = Paragraph::new(candidate_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll((0, self.scroll_offset));

        paragraph.render(area, &mut display.buffer);
    }
}
