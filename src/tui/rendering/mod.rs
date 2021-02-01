pub mod context;
pub mod display;
pub mod size;

pub trait Renderable {
    fn render(&self, app: &mut context::RenderContext);
}
