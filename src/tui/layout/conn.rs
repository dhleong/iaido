use crate::{
    editing::layout::conn::ConnLayout,
    tui::{LayoutContext, RenderContext, Renderable},
};

impl Renderable for ConnLayout {
    fn layout(&mut self, ctx: &LayoutContext) {
        // TODO stretch input to fit content; shrink output to fit input
        self.output.layout(ctx);
        self.input.layout(ctx);
    }

    fn render(&self, ctx: &mut RenderContext) {
        todo!()
    }
}
