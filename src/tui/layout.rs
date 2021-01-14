use super::Renderable;
use crate::editing::layout::{Layout, LayoutDirection, LayoutEntry};

impl Renderable for Layout {
    fn render(&self, context: &mut crate::tui::RenderContext) {
        match self.direction {
            LayoutDirection::Horizontal => todo!(), //render_horizontal(self, app, display, area),
            LayoutDirection::Vertical => render_vertical(self, context),
        };
    }
}

fn render_vertical(layout: &Layout, context: &mut crate::tui::RenderContext) {
    let mut layout_area = context.area;
    for entry in &layout.entries {
        match entry {
            LayoutEntry::Window(win) => {
                layout_area.height = win.size.h;
                win.render(&mut context.with_area(layout_area));
                layout_area.y += win.size.h;
            }

            _ => { /* todo */ }
        }
    }
}
