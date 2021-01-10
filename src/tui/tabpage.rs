use super::Renderable;
use crate::editing::tabpage::Tabpage;
use tui::layout::Rect;

impl Renderable for Tabpage {
    fn render(&self, app: &crate::App, display: &mut super::Display, area: Rect) {
        self.layout.render(app, display, area);
    }
}
