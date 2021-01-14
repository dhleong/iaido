use std::cmp;

use tui::{
    layout::Alignment, text, widgets::Block, widgets::BorderType, widgets::Borders,
    widgets::Paragraph, widgets::Widget, widgets::Wrap,
};

use super::{RenderContext, Renderable};
use crate::editing::{self, window::Window};

impl Renderable for Window {
    fn render(&self, context: &mut RenderContext) {
        let buf = match context.app.buffers.by_id(self.buffer) {
            None => return,
            Some(buf) => buf,
        };

        let count = buf.lines_count();
        let end = count - cmp::min(count, self.scrolled_lines as usize);
        let start = end - cmp::min(end, self.size.h as usize);

        let lines: Vec<text::Spans> = (start..end).map(|i| buf.get(i).clone()).collect();
        let candidate_text = text::Text::from(lines);

        let mut paragraph = Paragraph::new(candidate_text)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left)
            .scroll((0, self.scroll_offset));

        if self.focused {
            // TODO borders?
            let block = Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Rounded);
            paragraph = paragraph.block(block.clone());

            let cursor_area = block.inner(context.area);

            let cursor_x = self.cursor.col % context.area.width;
            let cursor_y_offset = self.cursor.col / context.area.width;
            let cursor_y = (self.cursor.line as usize) - start - (self.scroll_offset as usize);

            let x = cursor_area.x + cursor_x;
            let y = cursor_area.y + (cursor_y as u16) + cursor_y_offset;

            if self.inserting {
                context.display.set_cursor(editing::Cursor::Line(x, y));
            } else {
                context.display.set_cursor(editing::Cursor::Block(x, y));
            }
        }

        paragraph.render(context.area, &mut context.display.buffer);
    }
}
