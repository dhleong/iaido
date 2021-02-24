use tui::widgets::{Block, BorderType, Borders, Widget};

use crate::{
    editing::layout::{LayoutDirection, LinearLayout},
    tui::{RenderContext, Renderable},
};

use crate::tui::LayoutContext;

impl Renderable for LinearLayout {
    fn layout(&mut self, ctx: &LayoutContext) {
        for entry in &mut self.entries {
            entry.layout(ctx)
        }
    }

    fn render(&self, context: &mut RenderContext) {
        match self.direction {
            LayoutDirection::Horizontal => render_horizontal(self, context),
            LayoutDirection::Vertical => render_vertical(self, context),
        };
    }
}

fn render_vertical(layout: &LinearLayout, context: &mut RenderContext) {
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

        layout_area.height = entry.size().h;
        entry.render(&mut context.with_area(layout_area));
        layout_area.y += entry.size().h;
    }
}

fn render_horizontal(layout: &LinearLayout, context: &mut RenderContext) {
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

        layout_area.width = entry.size().w;
        entry.render(&mut context.with_area(layout_area));
        layout_area.x += entry.size().w;
    }
}
