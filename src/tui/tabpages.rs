use super::Renderable;
use crate::editing::tabpages::Tabpages;

impl Renderable for Tabpages {
    fn render(&self, display: &mut super::Display) {
        if self.len() == 1 {
            // TODO
            self.current_tab().render(display);
        } else {
            todo!();
        }
    }
}
