use super::Renderable;
use crate::editing::tabpage::Tabpage;

impl Renderable for Tabpage {
    fn layout(&mut self, ctx: &super::LayoutContext) {
        self.layout.layout(ctx);
    }

    fn render(&self, app: &mut crate::tui::RenderContext) {
        self.layout.render(app);
    }
}
