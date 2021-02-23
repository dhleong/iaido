use crate::{editing::layout::win::WinLayout, tui::Renderable};

impl Renderable for WinLayout {
    fn layout(&mut self, ctx: &crate::tui::LayoutContext) {
        self.window.layout(ctx)
    }
    fn render(&self, ctx: &mut crate::tui::RenderContext) {
        self.window.render(ctx)
    }
}
