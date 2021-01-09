use super::Renderable;
use crate::editing::tabpages::Tabpages;

impl Renderable for Tabpages {
    fn render<'a>(&self, app: &'a crate::App, display: &mut super::Display<'a>) {
        if self.len() == 1 {
            // TODO
            self.current_tab().render(app, display);
        } else {
            todo!();
        }
    }
}
