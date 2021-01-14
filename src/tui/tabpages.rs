use super::Renderable;
use crate::editing::tabpages::Tabpages;

impl Renderable for Tabpages {
    fn render(&self, app: &mut crate::tui::RenderContext) {
        if self.len() == 1 {
            // TODO
            self.current_tab().render(app);
        } else {
            todo!();
        }
    }
}
