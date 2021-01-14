use super::Renderable;
use crate::editing::tabpage::Tabpage;

impl Renderable for Tabpage {
    fn render(&self, app: &mut crate::tui::RenderContext) {
        self.layout.render(app);
    }
}
