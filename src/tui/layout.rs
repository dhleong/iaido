use super::Renderable;
use crate::editing::layout::{Layout, LayoutDirection, LayoutEntry};
use tui::layout::Rect;

impl Renderable for Layout {
    fn render(&self, app: &crate::app::State, display: &mut super::Display, area: Rect) {
        match self.direction {
            LayoutDirection::Horizontal => todo!(), //render_horizontal(self, app, display, area),
            LayoutDirection::Vertical => render_vertical(self, app, display, area),
        };
    }
}

fn render_vertical(
    layout: &Layout,
    app: &crate::app::State,
    display: &mut super::Display,
    area: Rect,
) {
    let mut layout_area = area;
    for entry in &layout.entries {
        match entry {
            LayoutEntry::Window(win) => {
                layout_area.height = win.size.h;
                win.render(app, display, layout_area);
                layout_area.y += win.size.h;
            }

            _ => { /* todo */ }
        }
    }
}
