use tui::layout::Rect;

use super::Renderable;
use crate::editing::window::Window;

impl Renderable for Window {
    fn render(&self, app: &crate::App, display: &mut super::Display, area: Rect) {
        let buf = match app.buffers.by_id(self.buffer) {
            None => return,
            Some(buf) => buf,
        };

        // TODO
        let y_offset = if (buf.lines_count() as u16) < area.height {
            area.height - (buf.lines_count() as u16)
        } else {
            0
        };

        for i in 0..buf.lines_count() - self.scrolled_lines as usize {
            let y = area.y + self.scrolled_lines as u16 + y_offset + (i as u16);
            display.buffer.set_spans(area.x, y, buf.get(i), area.width);
        }
    }
}
