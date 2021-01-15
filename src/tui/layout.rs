use tui::widgets::{Block, BorderType, Borders, Widget};

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
    let mut layout_area = context.area.clone();
    for entry in &layout.entries {
        // TODO better borders? what about corners, for example?
        if layout_area.y > 0 {
            let border = Block::default()
                .borders(Borders::TOP)
                .border_type(BorderType::Rounded);
            border.render(layout_area, &mut context.display.buffer);
            layout_area.y += 1;
        }

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
