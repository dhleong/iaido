use tui::layout::Rect;

use super::Renderable;
use crate::editing::tabpages::Tabpages;

impl Renderable for Tabpages {
    fn render(&self, app: &crate::app::State, display: &mut super::Display, area: Rect) {
        if self.len() == 1 {
            // TODO
            self.current_tab().render(app, display, area);
        } else {
            todo!();
        }
    }
}
