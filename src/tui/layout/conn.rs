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
        let mut layout_area = ctx.area.clone();
        layout_area.height = self.output.size.h;
        self.output.render(&mut ctx.with_area(layout_area));

        layout_area.y += self.output.size.h;
        layout_area.height = self.input.size.h;
        self.input.render(&mut ctx.with_area(layout_area));
    }
}
