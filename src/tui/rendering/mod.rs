pub mod context;
pub mod display;
pub mod size;

pub trait Renderable {
    fn layout(&mut self, _ctx: &context::LayoutContext) {}
    fn render(&self, ctx: &mut context::RenderContext);
}
