use tui::layout::Rect;

use super::Renderable;
use crate::editing::window::Window;

impl Renderable for Window {
    fn render<'a>(&self, app: &'a crate::App, display: &mut super::Display<'a>, area: Rect) {
        let buf = match app.buffers.by_id(self.buffer) {
            None => return,
            Some(buf) => buf,
        };
        for i in 0..buf.lines_count() {
            let line = buf.get(i).clone();
            display.lines.push(line);
        }
    }
}
