use tui::widgets::{Block, BorderType, Borders, Widget};

use super::Renderable;
use crate::editing::layout::{Layout, LayoutDirection, LayoutEntry};

impl Renderable for Layout {
    fn layout(&mut self, ctx: &super::LayoutContext) {
        for entry in &mut self.entries {
            match entry {
                LayoutEntry::Window(win) => win.layout(ctx),
                LayoutEntry::Layout(lyt) => lyt.layout(ctx),
            };
        }
    }

    fn render(&self, context: &mut crate::tui::RenderContext) {
        match self.direction {
            LayoutDirection::Horizontal => render_horizontal(self, context),
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

        match &entry {
            &LayoutEntry::Window(ref win) => {
                layout_area.height = win.size.h;
                win.render(&mut context.with_area(layout_area));
                layout_area.y += win.size.h;
            }

            &LayoutEntry::Layout(ref lyt) => {
                layout_area.height = lyt.size().h;
                lyt.render(&mut context.with_area(layout_area));
                layout_area.y += lyt.size().h;
            }
        }
    }
}

fn render_horizontal(layout: &Layout, context: &mut crate::tui::RenderContext) {
    let mut layout_area = context.area.clone();
    for entry in &layout.entries {
        // TODO better borders? what about corners, for example?
        if layout_area.x > 0 {
            let border = Block::default()
                .borders(Borders::LEFT)
                .border_type(BorderType::Rounded);
            border.render(layout_area, &mut context.display.buffer);
            layout_area.x += 1;
        }

        match &entry {
            &LayoutEntry::Window(ref win) => {
                layout_area.width = win.size.w;
                win.render(&mut context.with_area(layout_area));
                layout_area.x += win.size.w;
            }

            &LayoutEntry::Layout(ref lyt) => {
                layout_area.width = lyt.size().w;
                lyt.render(&mut context.with_area(layout_area));
                layout_area.x += lyt.size().w;
            }
        }
    }
}
