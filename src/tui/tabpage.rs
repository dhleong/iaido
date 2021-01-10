use super::Renderable;
use crate::editing::tabpage::Tabpage;
use tui::layout::Rect;

impl Renderable for Tabpage {
    fn render<'a>(&self, app: &'a crate::App, display: &mut super::Display<'a>, area: Rect) {
        self.layout.render(app, display, area);
    }
}
