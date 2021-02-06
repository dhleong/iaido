use tui::layout::Rect;

use super::Renderable;
use crate::editing::tabpages::Tabpages;

impl Renderable for Tabpages {
    fn layout(&mut self, ctx: &crate::tui::LayoutContext) {
        self.current_tab_mut().layout(&ctx);
    }

    fn render(&self, app: &mut crate::tui::RenderContext) {
        if self.len() == 1 {
            self.current_tab().render(&mut app.with_area(Rect {
                height: app.area.height,
                ..app.area
            }));
        } else {
            todo!();
        }
    }
}
